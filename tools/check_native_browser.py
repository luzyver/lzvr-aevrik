"""Exercise delayed CDP page discovery against a real Chromium process."""
import socket
import time
import json
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from types import SimpleNamespace
from concurrent.futures import ThreadPoolExecutor
from playwright.sync_api import sync_playwright, expect, TimeoutError as PlaywrightTimeoutError
from native_browser import connect_native, wait_for_debugger_exit, wait_for_page


def check_reset_connection(p):
    """A discovery response can outlive its WebSocket during WebView shutdown."""
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    host = p.chromium.launch(args=[f'--remote-debugging-port={port}'])
    import urllib.request
    with urllib.request.urlopen(f'http://127.0.0.1:{port}/json/version') as response:
        live = json.load(response)['webSocketDebuggerUrl']

    class Discovery(BaseHTTPRequestHandler):
        requests = 0

        def log_message(self, *_):
            pass

        def do_GET(self):
            if self.path == '/stale':
                # Close during the upgrade handshake, reproducing a dead endpoint.
                self.connection.shutdown(socket.SHUT_RDWR)
                self.connection.close()
                return
            type(self).requests += 1
            url = (f'ws://127.0.0.1:{self.server.server_port}/stale'
                   if self.requests == 1 else live)
            body = json.dumps({'webSocketDebuggerUrl': url}).encode()
            self.send_response(200)
            self.end_headers()
            self.wfile.write(body)

    server = ThreadingHTTPServer(('127.0.0.1', 0), Discovery)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        browser = connect_native(p, server.server_port, SimpleNamespace(poll=lambda: None, pid=123), timeout=5)
        try:
            page = browser.contexts[0].new_page()
            page.goto('data:text/html,<title>Recovered connection</title>')
            expect(page).to_have_title('Recovered connection')
            assert Discovery.requests >= 2, 'The stale endpoint was not rediscovered'
        finally:
            browser.close()
    finally:
        server.shutdown()
        server.server_close()
        thread.join()
        host.close()
    print('Native CDP: stale WebSocket is rediscovered and a real browser connects')


def check_connection_bounds(p):
    alive = SimpleNamespace(poll=lambda: None, pid=123)
    dead = SimpleNamespace(poll=lambda: 17, pid=456)
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        port = listener.getsockname()[1]
        listener.listen()
        started = time.monotonic()
        try:
            wait_for_debugger_exit(port, timeout=.2)
        except TimeoutError as error:
            assert 'still listens' in str(error)
        else:
            raise AssertionError('A lingering browser listener was treated as stopped')
        assert time.monotonic() - started < 2
    wait_for_debugger_exit(port, timeout=5)
    try:
        connect_native(p, port, dead, timeout=1)
    except RuntimeError as error:
        assert 'code 17' in str(error) and '456' in str(error)
    else:
        raise AssertionError('A crashed application was treated as connectable')
    started = time.monotonic()
    try:
        connect_native(p, port, alive, timeout=.4)
    except TimeoutError as error:
        assert 'port' in str(error) and 'attempts' in str(error)
    else:
        raise AssertionError('Unavailable debugger did not time out')
    assert time.monotonic() - started < 2
    print('Native CDP: lingering listener, application exit and bounded timeout passed')


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
        check_reset_connection(p)
        check_connection_bounds(p)
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
            except PlaywrightTimeoutError:
                pass
            else:
                raise AssertionError('Missing native page did not time out')
        finally:
            browser.close()
            host.close()
    print('Native CDP: delayed page, existing page and missing-page timeout passed')


if __name__ == '__main__':
    run()
