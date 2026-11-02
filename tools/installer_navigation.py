"""Bounded navigation to the NSIS application-directory page."""
import time
from pywinauto.timings import TimeoutError


def select_application_directory(window, destination, timeout=30):
    deadline = time.monotonic() + timeout
    # NSIS MUI's directory field is 1019. Custom pages can have other Edit controls.
    directory = window.child_window(control_id=1019, class_name='Edit')
    next_button = window.child_window(control_id=1, class_name='Button')
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError('Setup never offered a ready application directory picker')
        if directory.exists(timeout=0) and directory.is_visible():
            directory.wait('ready', timeout=remaining)
            directory.set_edit_text(str(destination))
            assert directory.window_text() == str(destination)
            return
        next_button.wait('ready', timeout=remaining)
        # A transition can finish during the wait. Never click past the target page.
        if directory.exists(timeout=0) and directory.is_visible():
            continue
        next_button.click()
