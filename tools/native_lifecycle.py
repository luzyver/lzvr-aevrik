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


def exit_from_tray(process):
    """Exit through the real tray menu when no workspace WebView is attached."""
    from pywinauto import Application

    app = Application(backend='win32').connect(process=process.pid, timeout=10)
    hwnd = window_handle(process, class_name="tray_icon_app")
    # Open this process's native menu with tray-icon's WM_RBUTTONUP callback.
    assert ctypes.windll.user32.PostMessageW(hwnd, 6002, 0, 0x0205)
    popup = app.window(class_name='#32768')
    popup.wait('visible', timeout=10)
    popup.menu().get_menu_path('Exit and stop services', exact=True)[0].click_input()
