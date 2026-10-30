"""Catalog UI contract; real downloads are covered separately in the packaged EXE."""
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from check_desktop_ui import MOCK, OUT
import json

CATALOG=json.loads((Path(__file__).resolve().parents[1]/"crates/aevrik-engine/src/packages/catalog.json").read_text())
SCRIPT=MOCK+"\nwindow.packages="+json.dumps([{**p,"status":"available","bytes":0,"total":None,"error":None,"path":None} for p in CATALOG])+r""";
const original=window.__TAURI_INTERNALS__.invoke;
window.__TAURI_INTERNALS__.invoke=async(command,args={})=>{
 if(command==='list_packages')return structuredClone(window.packages);
 if(command==='install_package'){
  const p=window.packages.find(p=>p.id===args.id);
  p.status='downloading';p.bytes=500;p.total=1000;
  await new Promise(resolve=>setTimeout(resolve,1400));
  if(window.failInstall){p.status='failed';p.error='Checksum mismatch';throw p.error;}
  p.status='installed';p.error=null;p.path='C:\\Aevrik\\caddy.exe';return;
 }
 return original(command,args);
};
"""
with sync_playwright() as p:
 browser=p.chromium.launch()
 page=browser.new_page(viewport={"width":1280,"height":820})
 errors=[]
 page.on("pageerror",lambda e:errors.append(str(e)))
 page.add_init_script(SCRIPT)
 page.goto("http://127.0.0.1:1420")
 page.locator('[data-nav="packages"]').click()
 expect(page.locator('[data-install="caddy"]')).to_be_visible()
 expect(page.locator('.package-row')).to_have_count(len({p.get('family',p['name']) for p in CATALOG}))
 page.locator('#search').fill('php')
 expect(page.locator('.package-row')).to_have_count(1)
 page.locator('#search').fill('')
 page.evaluate('window.failInstall=true')
 page.locator('[data-install="caddy"]').click()
 page.locator('#package-install [type="submit"]').click()
 expect(page.locator('[data-package="caddy"]')).to_contain_text('Downloading')
 expect(page.locator('[data-package="caddy"]')).to_contain_text('Checksum mismatch')
 expect(page.locator('[data-install="caddy"]')).to_have_text('Retry')
 assert page.locator('[data-configure="caddy"]').count()==0
 page.evaluate('window.failInstall=false')
 page.locator('[data-install="caddy"]').click()
 page.locator('#package-install [type="submit"]').click()
 expect(page.locator('[data-configure="caddy"]')).to_be_visible()
 page.screenshot(path=str(OUT/'catalog-dark.png'))
 page.locator('[data-configure="caddy"]').click()
 expect(page.locator('#registration [name="executable"]')).to_have_value('C:\\Aevrik\\caddy.exe')
 page.locator('[data-action="close-dialog"]').first.click()
 page.locator('[data-action="theme"]').first.click()
 page.screenshot(path=str(OUT/'catalog-light.png'))
 page.set_viewport_size({"width":800,"height":600})
 page.screenshot(path=str(OUT/'catalog-800.png'))
 assert page.evaluate("document.querySelector('#main').scrollHeight<=document.querySelector('#main').clientHeight+1")
 assert not errors,errors
 browser.close()
print('Catalog search, install progress, failed retry, confirmed installation and configuration passed.')
