"""Wait for a native WebView without blocking Playwright's event dispatcher."""


def wait_for_page(browser, timeout=20000):
    context = browser.contexts[0]
    if context.pages:
        return context.pages[0]
    return context.wait_for_event('page', timeout=timeout)
