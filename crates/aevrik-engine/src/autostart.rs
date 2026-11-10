//! "Start with Windows" for the signed-in account: one value under the
//! per-user Run key. Nothing is written for other users or as a service.
use crate::{Result, error::invalid};

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE: &str = "Aevrik";

/// The command Windows runs at sign-in: the window stays hidden in the tray.
pub fn command(executable: &std::path::Path) -> String {
    format!("\"{}\" --background", executable.display())
}

pub fn enabled() -> Result<bool> {
    Ok(registry::read(RUN, VALUE)?.is_some())
}

pub fn set(executable: Option<&std::path::Path>) -> Result<()> {
    match executable {
        Some(path) if path.is_absolute() && path.is_file() => {
            registry::write(RUN, VALUE, &command(path))
        }
        Some(_) => Err(invalid("Aevrik's executable could not be located.")),
        None => registry::delete(RUN, VALUE),
    }
}

#[cfg(windows)]
mod registry {
    use crate::Result;
    use std::ptr;
    use windows_sys::Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
        System::Registry::{
            HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE,
            REG_SZ, RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegQueryValueExW,
            RegSetValueExW,
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
    fn check(code: u32) -> Result<()> {
        if code != ERROR_SUCCESS {
            return Err(std::io::Error::from_raw_os_error(code as i32).into());
        }
        Ok(())
    }
    fn open(key: &str) -> Result<Key> {
        let mut handle = ptr::null_mut();
        // SAFETY: terminated UTF-16 input, valid output pointer, handle closed by Key.
        check(unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                wide(key).as_ptr(),
                0,
                ptr::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_QUERY_VALUE | KEY_SET_VALUE,
                ptr::null(),
                &mut handle,
                ptr::null_mut(),
            )
        })?;
        Ok(Key(handle))
    }
    pub fn read(key: &str, value: &str) -> Result<Option<String>> {
        let key = open(key)?;
        let name = wide(value);
        let mut data = vec![0u16; 32768];
        let mut bytes = (data.len() * 2) as u32;
        let mut kind = 0;
        // SAFETY: the buffer size is passed in bytes as RegQueryValueExW requires.
        let code = unsafe {
            RegQueryValueExW(
                key.0,
                name.as_ptr(),
                ptr::null(),
                &mut kind,
                data.as_mut_ptr().cast(),
                &mut bytes,
            )
        };
        if code == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        check(code)?;
        if kind != REG_SZ {
            return Ok(None);
        }
        let text = &data[..bytes as usize / 2];
        let end = text.iter().position(|&c| c == 0).unwrap_or(text.len());
        Ok(Some(String::from_utf16_lossy(&text[..end])))
    }
    pub fn write(key: &str, value: &str, text: &str) -> Result<()> {
        let key = open(key)?;
        let data = wide(text);
        // SAFETY: valid handle and terminated UTF-16 input with its exact byte size.
        check(unsafe {
            RegSetValueExW(
                key.0,
                wide(value).as_ptr(),
                0,
                REG_SZ,
                data.as_ptr().cast(),
                (data.len() * 2) as u32,
            )
        })
    }
    pub fn delete(key: &str, value: &str) -> Result<()> {
        let key = open(key)?;
        // SAFETY: valid handle and terminated UTF-16 value name.
        let code = unsafe { RegDeleteValueW(key.0, wide(value).as_ptr()) };
        if code == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        check(code)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use windows_sys::Win32::System::Registry::RegDeleteKeyW;
        struct TestKey(String);
        impl Drop for TestKey {
            fn drop(&mut self) {
                unsafe {
                    RegDeleteKeyW(HKEY_CURRENT_USER, wide(&self.0).as_ptr());
                }
            }
        }
        #[test]
        fn value_round_trips_and_delete_is_idempotent_in_a_disposable_key() {
            // Never touches the real Run key: exercise the adapter against its own key.
            let key = TestKey(format!(
                "Software\\Aevrik-Autostart-Test-{}",
                uuid::Uuid::new_v4()
            ));
            assert_eq!(read(&key.0, "Aevrik").unwrap(), None);
            let command = "\"D:\\開発\\Aevrik\\aevrik.exe\" --background";
            write(&key.0, "Aevrik", command).unwrap();
            assert_eq!(read(&key.0, "Aevrik").unwrap().as_deref(), Some(command));
            delete(&key.0, "Aevrik").unwrap();
            delete(&key.0, "Aevrik").unwrap();
            assert_eq!(read(&key.0, "Aevrik").unwrap(), None);
        }
    }
}

#[cfg(not(windows))]
mod registry {
    use crate::{Result, error::invalid};
    pub fn read(_: &str, _: &str) -> Result<Option<String>> {
        Ok(None)
    }
    pub fn write(_: &str, _: &str, _: &str) -> Result<()> {
        Err(invalid("Start with Windows is only supported on Windows."))
    }
    pub fn delete(_: &str, _: &str) -> Result<()> {
        Ok(())
    }
}
