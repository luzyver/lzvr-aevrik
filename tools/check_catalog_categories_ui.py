"""Verify category navigation and newly available runtimes at desktop widths."""
import json
from playwright.sync_api import sync_playwright, expect
from check_desktop_ui import MOCK, ROOT, OUT

catalog=json.loads((ROOT/'crates/aevrik-engine/src/packages/catalog.json').read_text())
script=MOCK+'\nwindow.packages='+json.dumps([
    {**p,'status':'available','bytes':0,'total':None,'error':None,'path':None}
    for p in catalog])+''';
const original=window.__TAURI_INTERNALS__.invoke;
window.__TAURI_INTERNALS__.invoke=async(command,args={})=>
 command==='list_packages'?structuredClone(window.packages):original(command,args);
'''

with sync_playwright() as p:
    browser=p.chromium.launch()
    page=browser.new_page(viewport={'width':1280,'height':820})
    errors=[]
    page.on('pageerror',lambda error:errors.append(str(error)))
    page.add_init_script(script)
    page.goto('http://127.0.0.1:1420')
    page.locator('[data-nav="packages"]').click()
    expect(page.locator('.package-row')).to_have_count(22)
    category=page.locator('#package-category')
    expect(category.locator('option')).to_have_text([
        'All','Runtimes','SDKs & compilers','Web servers','Database servers','Embedded databases','Development tools'])
    for label,families in [
        ('SDKs & compilers',{'go','java','dotnet-sdk','zig','nim'}),
        ('Database servers',{'mariadb','mongodb'}),
        ('Embedded databases',{'duckdb','sqlite'}),
        ('Development tools',{'mailpit'}),
        ('Runtimes',{'php','node','python','java-jre','bun','deno','dotnet-runtime','aspnet-runtime','luau','wasmtime'})]:
        category.select_option(label=label)
        expect(page.locator('.package-row')).to_have_count(len(families))
        assert set(page.locator('.package-row').evaluate_all('(rows)=>rows.map(r=>r.dataset.family)'))==families
    for name,family in [('Java JRE','java-jre'),('Luau','luau'),('Wasmtime','wasmtime')]:
        page.locator('#search').fill(name)
        row=page.locator('.package-row')
        expect(row).to_have_count(1)
        expect(row).to_have_attribute('data-family',family)
        expect(row.locator('[data-install]')).to_be_enabled()
        row.locator('select').select_option(index=1)
        expect(row.locator('[data-install]')).to_be_enabled()
    page.locator('#search').fill('')
    category.select_option(label='SDKs & compilers')
    page.screenshot(path=str(OUT/'catalog-categories-dark.png'))
    page.locator('[data-action="theme"]').first.click()
    page.screenshot(path=str(OUT/'catalog-categories-light.png'))
    page.set_viewport_size({'width':800,'height':600})
    page.screenshot(path=str(OUT/'catalog-categories-800.png'))
    assert page.evaluate('document.documentElement.scrollWidth<=innerWidth')
    assert page.evaluate("document.querySelector('#main').scrollHeight<=document.querySelector('#main').clientHeight+1")
    assert not errors,errors
    browser.close()
print('Category filters, new runtime selection, themes and 800px layout passed.')
