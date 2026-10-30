"""Version selection must preserve per-release progress and installed state."""
import json
from playwright.sync_api import sync_playwright, expect
from check_desktop_ui import MOCK, OUT

items=[]
for family,name,versions in [('caddy','Caddy',['2.11.7','2.11.6']),('php','PHP',['8.4.27-1','8.3.29'])]:
 for version in versions:
  items.append(dict(id=family+'-'+version,family=family,name=name,version=version,category='Language' if family=='php' else 'Web server',description='Official test fixture',source='official.test',architecture='x64',channel='Release',status='available',bytes=0,total=None,error=None,path=None))
script=MOCK+'\nwindow.packages='+json.dumps(items)+r""";
const original=window.__TAURI_INTERNALS__.invoke;
window.installs=[];
window.__TAURI_INTERNALS__.invoke=async(command,args={})=>{
 if(command==='list_packages')return structuredClone(window.packages);
 if(command==='install_package'){
  window.installs.push(args.id);
  const item=window.packages.find(p=>p.id===args.id);item.status='downloading';item.bytes=20;item.total=100;
  await new Promise(resolve=>setTimeout(resolve,1800));
  item.status='installed';item.path='C:\\Aevrik\\'+args.id+'\\caddy.exe';return;
 }
 return original(command,args);
};
"""
with sync_playwright() as p:
 browser=p.chromium.launch();page=browser.new_page(viewport={'width':1280,'height':820})
 errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
 page.add_init_script(script);page.goto('http://127.0.0.1:1420')
 page.locator('[data-nav="packages"]').click()
 expect(page.locator('.package-row')).to_have_count(2)
 selector=page.locator('[data-package-version="caddy"]')
 expect(selector.locator('option')).to_have_count(2)
 selector.select_option('caddy-2.11.6')
 page.locator('[data-install="caddy-2.11.6"]').click()
 page.locator('#package-install [type="submit"]').click()
 expect(page.locator('[data-family="caddy"]')).to_contain_text('Downloading')
 selector.select_option('caddy-2.11.7')
 expect(page.locator('[data-install="caddy-2.11.7"]')).to_be_enabled()
 expect(page.locator('[data-family="caddy"]')).to_contain_text('1 installed',timeout=5000)
 assert page.locator('[data-configure="caddy-2.11.7"]').count()==0
 assert page.evaluate('window.installs')==['caddy-2.11.6']
 page.locator('[data-install="caddy-2.11.7"]').click()
 page.locator('#package-install [type="submit"]').click()
 expect(page.locator('[data-configure="caddy-2.11.7"]')).to_be_visible()
 expect(page.locator('[data-family="caddy"]')).to_contain_text('2 installed')
 selector.select_option('caddy-2.11.6')
 page.locator('[data-configure="caddy-2.11.6"]').click()
 expect(page.locator('#registration [name="executable"]')).to_have_value('C:\\Aevrik\\caddy-2.11.6\\caddy.exe')
 page.locator('[data-action="close-dialog"]').first.click()
 page.locator('#search').fill('8.3')
 expect(page.locator('.package-row')).to_have_count(1)
 expect(page.locator('[data-package-version="php"]')).to_have_value('php-8.3.29')
 page.locator('#search').fill('')
 page.locator('#package-category').select_option('Web server')
 expect(page.locator('.package-row')).to_have_count(1)
 assert not errors,errors
 browser.close()
print('Product grouping, exact version install, selection during download, side-by-side status, configuration and version search passed.')
