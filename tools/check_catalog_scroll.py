"""Exercise catalog scrolling while the real renderer replaces its DOM nodes."""
import json

from playwright.sync_api import expect, sync_playwright

from catalog_scroll import scroll_to_last_package
from check_desktop_ui import MOCK, ROOT


catalog = json.loads((ROOT / 'crates/aevrik-engine/src/packages/catalog.json').read_text())
packages = [{**item, 'status': 'available', 'bytes': 0, 'total': None,
             'error': None, 'path': None} for item in catalog]
script = MOCK + '\nwindow.packages=' + json.dumps(packages) + ''';
const original = window.__TAURI_INTERNALS__.invoke;
window.__TAURI_INTERNALS__.invoke = async (command, args) =>
    command === 'list_packages' ? structuredClone(window.packages) : original(command, args);
'''

with sync_playwright() as playwright:
    browser = playwright.chromium.launch()
    page = browser.new_page(viewport={'width': 800, 'height': 600})
    errors = []
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.add_init_script(script)
    page.goto('http://127.0.0.1:1420')
    page.locator('[data-nav="packages"]').click()
    expect(page.locator('.package-row')).to_have_count(len({p.get('family', p['name']) for p in catalog}))
    expect(page.locator('.package-row').last.locator('[data-install]')).not_to_be_in_viewport()
    # Refresh between animation frames, where Playwright normally checks stability.
    # Use the production renderer, including its scroll-position restoration.
    page.evaluate('''async () => {
        const {render} = await import('/views.ts');
        window.catalogFrames = 0;
        window.keepRendering = true;
        function frame() {
            if (!window.keepRendering || window.catalogFrames >= 30) return;
            render();
            window.catalogFrames++;
            requestAnimationFrame(frame);
        }
        requestAnimationFrame(frame);
    }''')
    try:
        scroll_to_last_package(page)
        page.wait_for_function('window.catalogFrames >= 30')
        expect(page.locator('.package-row').last.locator('[data-install]')).to_be_in_viewport(ratio=1)
        assert page.locator('.catalog-scroll').evaluate('(element) => element.scrollTop > 0')
        assert page.evaluate("document.scrollingElement.scrollTop === 0 && document.querySelector('#main').scrollTop === 0")
        assert not errors, errors
    finally:
        page.evaluate('window.keepRendering = false')
        browser.close()

print('Catalog reaches the final package and retains its scroll across live renders.')
