"""Native lifecycle helpers scoped to the smoke test's own process."""
import ctypes
from ctypes import wintypes
import time


def window_handle(process, *, title=None, class_name=None):
    user32 = ctypes.windll.user32
    matches = []
    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)

    def visit(hwnd, _):
        hwnd = wintypes.HWND(hwnd)
        pid = wintypes.DWORD()
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
        if pid.value != process.pid:
            return True
        text = ctypes.create_unicode_buffer(256)
        user32.GetWindowTextW(hwnd, text, len(text))
        if title is not None and text.value != title:
            return True
        user32.GetClassNameW(hwnd, text, len(text))
        if class_name is not None and text.value != class_name:
            return True
        matches.append(hwnd.value)
        return True

    user32.EnumWindows(callback_type(visit), 0)
    assert len(matches) == 1, f"Expected one window, found {len(matches)}"
    return wintypes.HWND(matches[0])


def wait_visible(process, visible):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        assert process.poll() is None, "Closing the window stopped Aevrik"
        hwnd = window_handle(process, title="Aevrik")
        if bool(ctypes.windll.user32.IsWindowVisible(hwnd)) == visible:
            return
        time.sleep(.1)
    raise AssertionError(f"Window visibility did not become {visible}")


def restore_from_tray(process):
    hwnd = window_handle(process, class_name="tray_icon_app")
    # tray-icon 0.25's native callback message and WM_LBUTTONUP. Deliver to
    # this process's real tray window, exercising its registered Rust handler.
    assert ctypes.windll.user32.PostMessageW(hwnd, 6002, 0, 0x0202)
    wait_visible(process, True)
    deadline = time.monotonic() + 10
    while ctypes.windll.user32.IsIconic(window_handle(process, title="Aevrik")):
        assert time.monotonic() < deadline, "Tray restore left the window minimized"
        time.sleep(.1)


def exit_application(page):
    page.locator('[data-nav="settings"]').click()
    page.locator('[data-action="exit-app"]').click()
    # Exit asks for confirmation only while services are running.
    confirm = page.locator('#exit-confirm [type="submit"]')
    try:
        confirm.click(timeout=1500)
    except Exception:
        pass  # Nothing was running, or the application already exited.


def exit_from_tray(process, timeout=10):
    """Exit through the real tray menu when no workspace WebView is attached."""
    from pywinauto import Application
    from pywinauto.timings import TimeoutError

    app = Application(backend='win32').connect(process=process.pid, timeout=timeout)
    hwnd = window_handle(process, class_name="tray_icon_app")
    popup = app.window(class_name='#32768')
    deadline = time.monotonic() + timeout
    # tray-icon drops callbacks while Shell_NotifyIconGetRect cannot find the
    # newly registered icon. Retry opening until the real menu is visible.
    while not (popup.exists(timeout=0) and popup.is_visible()):
        assert process.poll() is None, 'Aevrik exited before its tray menu opened'
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError(f'Aevrik tray menu did not appear within {timeout}s (PID {process.pid})')
        assert ctypes.windll.user32.PostMessageW(hwnd, 6002, 0, 0x0205)
        try:
            popup.wait('visible', timeout=min(.5, remaining), retry_interval=.05)
        except TimeoutError:
            continue
    item = popup.menu().get_menu_path('Exit and stop services', exact=True)[0]
    assert item.is_enabled(), 'Tray Exit is disabled'
    command = item.item_id()
    # Dispatch the selected item's real WM_COMMAND to its tray owner. Mouse
    # coordinates can lose a race with focus/cursor changes on the CI desktop.
    assert ctypes.windll.user32.PostMessageW(hwnd, 0x001F, 0, 0)  # WM_CANCELMODE
    assert ctypes.windll.user32.PostMessageW(hwnd, 0x0111, command, 0)  # WM_COMMAND
