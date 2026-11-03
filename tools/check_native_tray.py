"""Exercise real tray Exit with normal, delayed and unavailable shell icons."""
import ctypes
from ctypes import wintypes
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import time

import win32gui
from playwright.sync_api import expect, sync_playwright
from pywinauto.timings import TimeoutError, wait_until

from check_native import ROOT, endpoint, free_port
from native_browser import wait_for_page
from native_lifecycle import exit_from_tray, wait_visible, window_handle


class IconIdentifier(ctypes.Structure):
    _fields_ = [('size', wintypes.DWORD), ('hwnd', wintypes.HWND),
                ('id', wintypes.UINT), ('guid', ctypes.c_byte * 16)]


def remove_test_icon(hwnd):
    """Remove only this test process's icon to reproduce shell readiness delay."""
    def registered_id():
        # tray-icon 0.25 uses small sequential IDs, including builder IDs.
        # Find the registered ID instead of assuming a builder allocation order.
        for icon_id in range(1, 64):
            identity = IconIdentifier(ctypes.sizeof(IconIdentifier), hwnd, icon_id)
            rect = wintypes.RECT()
            if ctypes.windll.shell32.Shell_NotifyIconGetRect(
                    ctypes.byref(identity), ctypes.byref(rect)) == 0:
                return icon_id
        return 0

    wait_until(10, .05, lambda: bool(registered_id()))
    icon_id = registered_id()
    assert icon_id, 'Test icon disappeared before removal'
    win32gui.Shell_NotifyIcon(win32gui.NIM_DELETE, (hwnd, icon_id))
    assert not registered_id(), 'Test icon is still registered'


def check(mode):
    with tempfile.TemporaryDirectory(prefix='aevrik-tray-', ignore_cleanup_errors=True) as scratch:
        env = os.environ.copy()
        env['AEVRIK_DATA_DIR'] = scratch
        env['WEBVIEW2_USER_DATA_FOLDER'] = str(Path(scratch) / 'webview')
        port = free_port()
        env['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS'] = f'--remote-debugging-port={port}'
        startup = subprocess.STARTUPINFO()
        startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = 0
        process = subprocess.Popen([str(ROOT / 'target/release/aevrik.exe')],
                                   env=env, cwd=scratch, startupinfo=startup)
        timer = None
        try:
            with sync_playwright() as p:
                browser = p.chromium.connect_over_cdp(endpoint(port, process))
                try:
                    page = wait_for_page(browser)
                    expect(page.locator('#engine-status')).to_have_text('Engine connected', timeout=20000)
                    if mode == 'hidden':
                        page.locator('[data-window="close"]').click()
                        wait_visible(process, False)
                    if mode in ('delayed', 'unavailable'):
                        hwnd = window_handle(process, class_name='tray_icon_app').value
                        remove_test_icon(hwnd)
                        restore = win32gui.RegisterWindowMessage('TaskbarCreated')
                        if mode == 'delayed':
                            # Deliver only to this app, never restart or alter Explorer.
                            timer = threading.Timer(1.5, lambda: win32gui.PostMessage(hwnd, restore, 0, 0))
                            timer.start()
                        else:
                            started = time.monotonic()
                            try:
                                exit_from_tray(process, timeout=1)
                            except TimeoutError as error:
                                assert 'tray menu did not appear' in str(error)
                            else:
                                raise AssertionError('Unavailable tray must time out')
                            assert time.monotonic() - started < 5, 'Tray wait exceeded its bound'
                            assert process.poll() is None, 'Missing menu must not kill the app'
                            win32gui.PostMessage(hwnd, restore, 0, 0)
                    exit_from_tray(process)
                    process.wait(timeout=20)
                    assert process.returncode == 0, 'Tray Exit did not shut down cleanly'
                finally:
                    browser.close()
        finally:
            if timer is not None:
                timer.cancel()
                timer.join()
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=10)
    print(f'Native tray Exit passed: {mode}', flush=True)


def run():
    for mode in ('visible', 'hidden', 'delayed', 'unavailable'):
        check(mode)


if __name__ == '__main__':
    run()
