"""Exercise installer navigation against real delayed Win32 controls, without installing."""
import subprocess
import sys
import threading
from pathlib import Path


def fixture(mode):
    import win32con as c
    import win32gui as gui

    next_button = None
    page = 'welcome'
    advance = c.WM_APP + 1

    def later(window):
        timer = threading.Timer(.6, lambda: gui.PostMessage(window, advance, 0, 0))
        timer.daemon = True
        timer.start()

    def messages(window, message, wparam, lparam):
        nonlocal page
        if message == advance:
            if page == 'transition':
                gui.CreateWindowEx(0, 'Edit', 'Default application folder',
                                   c.WS_CHILD | c.WS_VISIBLE | c.WS_BORDER,
                                   10, 45, 300, 25, window, 1019, None, None)
                page = 'directory'
            gui.EnableWindow(next_button, True)
            return 0
        if message == c.WM_COMMAND and wparam & 0xffff == 1:
            if page == 'directory':
                gui.SetWindowText(window, 'Unexpected install click')
            else:
                page = 'transition'
                gui.EnableWindow(next_button, False)
                later(window)
            return 0
        if message == c.WM_CLOSE:
            gui.DestroyWindow(window)
            return 0
        if message == c.WM_DESTROY:
            gui.PostQuitMessage(0)
            return 0
        return gui.DefWindowProc(window, message, wparam, lparam)

    cls = gui.WNDCLASS()
    cls.lpszClassName = 'AevrikInstallerNavigationFixture'
    cls.lpfnWndProc = messages
    atom = gui.RegisterClass(cls)
    # Offscreen, non-activating test window; no installer, registry or profile writes.
    window = gui.CreateWindowEx(c.WS_EX_NOACTIVATE, atom, 'Navigation fixture',
                                c.WS_OVERLAPPEDWINDOW, -10000, -10000, 400, 200,
                                0, 0, None, None)
    if mode == 'unrelated':
        gui.CreateWindowEx(0, 'Edit', 'Unrelated input', c.WS_CHILD | c.WS_VISIBLE,
                           10, 10, 300, 25, window, 1000, None, None)
    next_button = gui.CreateWindowEx(0, 'Button', 'Next',
                                     c.WS_CHILD | c.WS_VISIBLE | c.WS_DISABLED,
                                     10, 90, 80, 25, window, 1, None, None)
    gui.ShowWindow(window, c.SW_SHOWNOACTIVATE)
    if mode != 'stuck':
        later(window)
    print(window, flush=True)
    gui.PumpMessages()


def run():
    from pywinauto import Application
    from pywinauto.timings import TimeoutError
    from installer_navigation import select_application_directory

    for mode in ['delayed', 'unrelated', 'stuck']:
        process = subprocess.Popen([sys.executable, __file__, '--fixture', mode],
                                   stdout=subprocess.PIPE, text=True,
                                   creationflags=subprocess.CREATE_NO_WINDOW)
        try:
            handle = int(process.stdout.readline())
            window = Application(backend='win32').connect(process=process.pid).window(handle=handle)
            if mode == 'stuck':
                try:
                    select_application_directory(window, Path(r'C:\Test application'), timeout=.5)
                except TimeoutError:
                    pass
                else:
                    raise AssertionError('Disabled installer must time out, not pass')
            else:
                select_application_directory(window, Path(r'C:\Test application'), timeout=10)
                assert window.child_window(control_id=1019, class_name='Edit').window_text() == r'C:\Test application'
                if mode == 'unrelated':
                    assert window.child_window(control_id=1000, class_name='Edit').window_text() == 'Unrelated input'
                assert window.window_text() == 'Navigation fixture', 'Clicked past directory page'
            window.close()
            process.wait(timeout=10)
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=10)
    print('Installer navigation: delayed buttons/pages, exact directory field and timeout passed.')


if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == '--fixture':
        fixture(sys.argv[2])
    else:
        run()
