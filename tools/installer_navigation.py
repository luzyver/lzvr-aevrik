"""Bounded navigation to the NSIS application-directory page."""
import time
from pywinauto.timings import TimeoutError, wait_until


def _page_identity(window):
    # NSIS keeps its outer dialog and Next button across page changes. The
    # embedded dialog is replaced. Include its children in case Windows reuses
    # a recently destroyed page handle; an empty/hidden transition isn't a page.
    pages = [page for page in window.children(class_name='#32770') if page.is_visible()]
    return tuple((page.handle, tuple(child.handle for child in page.children()))
                 for page in pages if page.children())


def select_application_directory(window, destination, timeout=30):
    deadline = time.monotonic() + timeout
    # NSIS MUI's directory field is 1019. Custom pages can have other Edit controls.
    directory = window.child_window(control_id=1019, class_name='Edit')
    next_button = window.child_window(control_id=1, class_name='Button')
    def remaining():
        seconds = deadline - time.monotonic()
        if seconds <= 0:
            raise TimeoutError('Setup never offered a ready application directory picker')
        return seconds

    while True:
        if directory.exists(timeout=0) and directory.is_visible():
            directory.wait('ready', timeout=remaining())
            directory.set_edit_text(str(destination))
            assert directory.window_text() == str(destination)
            print(f'Setup application directory selected: {destination}', flush=True)
            return
        wait_until(remaining(), .05, lambda: bool(_page_identity(window)))
        page = _page_identity(window)
        next_button.wait('ready', timeout=remaining())
        # A transition can finish during the wait. Never click past the target page.
        if directory.exists(timeout=0) and directory.is_visible():
            continue
        if _page_identity(window) != page:
            continue
        print(f'Setup advancing page using {next_button.window_text()!r}', flush=True)
        next_button.click()
        # A ready button does NOT mean NSIS finished processing the click. Its
        # script thread may still be checking/skipping the next page. A second
        # click queues another advance and can skip Directory and start install.
        wait_until(remaining(), .05,
                   lambda: bool(current := _page_identity(window)) and current != page)
