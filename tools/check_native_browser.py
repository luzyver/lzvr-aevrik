"""Exercise delayed CDP page discovery against a real Chromium process."""
import socket
import time
from concurrent.futures import ThreadPoolExecutor
from playwright.sync_api import sync_playwright, expect, TimeoutError
from native_browser import wait_for_page


def open_delayed_page(endpoint):
    time.sleep(.3)
    with sync_playwright() as p:
        browser = p.chromium.connect_over_cdp(endpoint)
        page = browser.contexts[0].new_page()
        page.goto('data:text/html,<title>Delayed native page</title>')


def run():
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    endpoint = f'http://127.0.0.1:{port}'
    with sync_playwright() as p:
        host = p.chromium.launch(args=[f'--remote-debugging-port={port}'])
        browser = p.chromium.connect_over_cdp(endpoint)
        try:
            assert not browser.contexts[0].pages
            with ThreadPoolExecutor(max_workers=1) as pool:
                opening = pool.submit(open_delayed_page, endpoint)
                page = wait_for_page(browser, timeout=10000)
                expect(page).to_have_title('Delayed native page')
                opening.result(timeout=10)
            assert wait_for_page(browser) == page
            page.close()
            try:
                wait_for_page(browser, timeout=100)
            except TimeoutError:
                pass
            else:
                raise AssertionError('Missing native page did not time out')
        finally:
            browser.close()
            host.close()
    print('Native CDP: delayed page, existing page and missing-page timeout passed')


if __name__ == '__main__':
    run()
