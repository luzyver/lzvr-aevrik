"""Shared viewport assertions for the live package catalog."""
from playwright.sync_api import Page, expect


def scroll_to_last_package(page: Page) -> None:
    # The live renderer replaces rows between animation frames. Resolve and
    # scroll the current pane in one browser task instead of retaining a row
    # handle while Playwright waits for layout stability.
    page.wait_for_function('''() => {
        const pane = document.querySelector('.catalog-scroll');
        if (!pane) return false;
        pane.scrollTop = pane.scrollHeight;
        return true;
    }''')
    expect(page.locator('.package-row').last.locator('[data-install]')).to_be_in_viewport(ratio=1)
    expect(page.locator('#main')).to_have_js_property('scrollTop', 0)
    assert page.evaluate('document.scrollingElement.scrollTop') == 0
