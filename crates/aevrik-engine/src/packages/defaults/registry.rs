//! Registry access is confined here. No shell, setx truncation or process-wide set_var.
use super::environment::{Environment, Value};
use super::{PackageResult, Scope, invalid};
use std::{collections::BTreeMap, ptr};
use windows_sys::Win32::{
    Foundation::{ERROR_ACCESS_DENIED, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS},
    System::Registry::{
        HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_EXPAND_SZ,
        REG_SZ, RegCloseKey, RegEnumValueW, RegOpenKeyExW, RegSetValueExW,
    },
    UI::WindowsAndMessaging::{
        HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
    },
};

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
struct Key(HKEY);
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

fn open(scope: Scope, access: u32) -> PackageResult<Key> {
    let (root, name) = match scope {
        Scope::User => (HKEY_CURRENT_USER, "Environment"),
        Scope::Machine => (
            HKEY_LOCAL_MACHINE,
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        ),
    };
    let mut handle = ptr::null_mut();
    // SAFETY: terminated UTF-16 input, valid output pointer, handle closed by Key.
    let code = unsafe { RegOpenKeyExW(root, wide(name).as_ptr(), 0, access, &mut handle) };
    if code == ERROR_ACCESS_DENIED && access & KEY_SET_VALUE != 0 {
        return Err(invalid(
            "Windows denied the PATH change. Close Aevrik, reopen it with Run as administrator, and try again. No PATH changes were made.",
        ));
    }
    if code != ERROR_SUCCESS {
        return Err(std::io::Error::from_raw_os_error(code as i32).into());
    }
    Ok(Key(handle))
}

fn values(scope: Scope) -> PackageResult<BTreeMap<String, Value>> {
    let key = open(scope, KEY_QUERY_VALUE)?;
    read_values(&key)
}

fn read_values(key: &Key) -> PackageResult<BTreeMap<String, Value>> {
    let mut result = BTreeMap::new();
    // Windows environment strings fit in 32K UTF-16 code units. Use aligned buffers.
    let mut name = vec![0u16; 32768];
    let mut data = vec![0u16; 65536];
    for index in 0..10000 {
        let mut name_len = name.len() as u32;
        let mut bytes = (data.len() * 2) as u32;
        let mut kind = 0;
        // SAFETY: buffer sizes passed in bytes/code units as required by RegEnumValueW.
        let code = unsafe {
            RegEnumValueW(
                key.0,
                index,
                name.as_mut_ptr(),
                &mut name_len,
                ptr::null(),
                &mut kind,
                data.as_mut_ptr().cast(),
                &mut bytes,
            )
        };
        if code == ERROR_NO_MORE_ITEMS {
            return Ok(result);
        }
        if code != ERROR_SUCCESS {
            return Err(std::io::Error::from_raw_os_error(code as i32).into());
        }
        let name = String::from_utf16(&name[..name_len as usize])
            .map_err(|_| invalid("Windows environment name contains invalid Unicode."))?;
        if kind != REG_SZ && kind != REG_EXPAND_SZ {
            if name.eq_ignore_ascii_case("path") {
                return Err(invalid(
                    "Windows PATH must be a string or expandable string registry value.",
                ));
            }
            continue;
        }
        if !bytes.is_multiple_of(2) {
            return Err(invalid(
                "Windows environment contains an invalid Unicode value.",
            ));
        }
        let mut text = &data[..bytes as usize / 2];
        if text.last() == Some(&0) {
            text = &text[..text.len() - 1];
        }
        if text.contains(&0) {
            return Err(invalid(
                "Windows environment contains an embedded null character.",
            ));
        }
        let text = String::from_utf16(text)
            .map_err(|_| invalid("Windows environment contains invalid Unicode."))?;
        result.insert(name.to_uppercase(), Value { text, kind });
    }
    Err(invalid("Windows environment has too many entries."))
}

pub(super) fn read() -> PackageResult<Environment> {
    Ok(Environment {
        machine: values(Scope::Machine)?,
        user: values(Scope::User)?,
    })
}

pub(super) fn write_path(scope: Scope, kind: u32, value: &str) -> PackageResult<()> {
    let key = open(scope, KEY_SET_VALUE)?;
    set_path(&key, kind, value)?;
    // A successful write must not be reported as failed if a window ignores notification.
    // Existing terminals still require a restart; we never mutate the engine's environment.
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            wide("Environment").as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            1000,
            ptr::null_mut(),
        );
    }
    Ok(())
}

fn set_path(key: &Key, kind: u32, value: &str) -> PackageResult<()> {
    let data = wide(value);
    // SAFETY: valid handle and terminated UTF-16 input with its exact byte size.
    let code = unsafe {
        RegSetValueExW(
            key.0,
            wide("Path").as_ptr(),
            0,
            kind,
            data.as_ptr().cast(),
            (data.len() * 2) as u32,
        )
    };
    if code != ERROR_SUCCESS {
        return Err(std::io::Error::from_raw_os_error(code as i32).into());
    }
    Ok(())
}

/// Serialize cooperating Aevrik instances, including separate data folders/sessions.
pub(super) struct PathLock(windows_sys::Win32::Foundation::HANDLE);
impl PathLock {
    pub fn acquire() -> PackageResult<Self> {
        use windows_sys::Win32::{
            Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0},
            System::Threading::{CreateMutexW, WaitForSingleObject},
        };
        let handle = unsafe {
            CreateMutexW(
                ptr::null(),
                0,
                wide(r"Global\Aevrik.RuntimeDefaults.v1").as_ptr(),
            )
        };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        let result = unsafe { WaitForSingleObject(handle, 10_000) };
        if result != WAIT_OBJECT_0 && result != WAIT_ABANDONED {
            unsafe {
                CloseHandle(handle);
            }
            return Err(invalid(
                "Another Aevrik instance is changing PATH. Try again after it finishes.",
            ));
        }
        Ok(Self(handle))
    }
}
impl Drop for PathLock {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::System::Threading::ReleaseMutex(self.0);
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::System::Registry::{
        REG_OPTION_NON_VOLATILE, RegCreateKeyExW, RegDeleteKeyW,
    };
    struct TestKey(Vec<u16>);
    impl Drop for TestKey {
        fn drop(&mut self) {
            unsafe {
                RegDeleteKeyW(HKEY_CURRENT_USER, self.0.as_ptr());
            }
        }
    }
    #[test]
    fn isolated_registry_preserves_unicode_long_paths_and_value_kind() {
        // Never writes Environment or broadcasts: exercise the exact adapter against a disposable key.
        let name = wide(&format!(
            "Software\\Aevrik-Path-Test-{}",
            uuid::Uuid::new_v4()
        ));
        let cleanup = TestKey(name);
        let mut handle = ptr::null_mut();
        let code = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                cleanup.0.as_ptr(),
                0,
                ptr::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE | KEY_QUERY_VALUE,
                ptr::null(),
                &mut handle,
                ptr::null_mut(),
            )
        };
        assert_eq!(code, ERROR_SUCCESS);
        let key = Key(handle);
        for kind in [REG_SZ, REG_EXPAND_SZ] {
            let text = format!(
                "D:\\開発\\node;%SystemRoot%;{}",
                "D:\\Long entry;".repeat(200)
            );
            set_path(&key, kind, &text).unwrap();
            assert_eq!(read_values(&key).unwrap()["PATH"], Value { text, kind });
        }
        set_path(&key, REG_SZ, "").unwrap();
        assert_eq!(read_values(&key).unwrap()["PATH"].text, "");
    }
}
