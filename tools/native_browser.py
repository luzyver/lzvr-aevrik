"""Wait for a native WebView without blocking Playwright's event dispatcher."""
import json
import socket
import time
import urllib.request


def connect_native(playwright, port, process, timeout=45):
    """Wait for discovery AND the WebSocket handshake within one deadline."""
    from playwright.sync_api import Error

    deadline = time.monotonic() + timeout
    last_error = None
    attempts = 0
    while time.monotonic() < deadline:
        code = process.poll()
        if code is not None:
            raise RuntimeError(f'Aevrik PID {process.pid} exited with code {code} before CDP connected')
        attempts += 1
        try:
            with urllib.request.urlopen(f'http://127.0.0.1:{port}/json/version',
                                        timeout=min(1, max(.001, deadline - time.monotonic()))) as response:
                endpoint = json.load(response)['webSocketDebuggerUrl']
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                break
            browser = playwright.chromium.connect_over_cdp(endpoint, timeout=min(5000, remaining * 1000))
            code = process.poll()
            if code is not None:
                browser.close()
                raise RuntimeError(f'Aevrik PID {process.pid} exited with code {code} during CDP connection')
            print(f'Native CDP connected: PID {process.pid}, port {port}, attempts {attempts}', flush=True)
            return browser
        except (OSError, ValueError, KeyError, Error) as error:
            last_error = error
            time.sleep(min(.2, max(0, deadline - time.monotonic())))
    raise TimeoutError(f'WebView2 CDP connection timed out after {timeout}s '
                       f'(PID {process.pid}, port {port}, attempts {attempts}): {last_error}') from last_error


def wait_for_debugger_exit(port, timeout=20):
    """Do not reuse a WebView profile while its previous browser is shutting down."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(('127.0.0.1', port),
                                          timeout=min(3, max(.001, deadline - time.monotonic()))):
                pass
        except ConnectionRefusedError:
            return
        except TimeoutError:
            pass
        time.sleep(.1)
    raise TimeoutError(f'Previous WebView2 debugger still listens on port {port} after {timeout}s')


def wait_for_page(browser, timeout=20000):
    context = browser.contexts[0]
    if context.pages:
        return context.pages[0]
    return context.wait_for_event('page', timeout=timeout)
