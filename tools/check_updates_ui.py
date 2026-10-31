"""Update interaction contracts; IPC is mocked, native networking is tested separately."""
from playwright.sync_api import sync_playwright, expect
from check_desktop_ui import MOCK, OUT

SCRIPT=MOCK+r"""
window.updateItems=[{id:'node-old',family:'node',name:'Node.js',version:'20.0.0',category:'Language',description:'JavaScript runtime',source:'nodejs.org',status:'installed',bytes:0,total:null,path:'D:\\Runtimes\\node.exe',error:null}];
window.updateCalls=[];
const previous=window.__TAURI_INTERNALS__.invoke;
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
 if(command==='list_packages')return structuredClone(window.updateItems);
 if(command==='refresh_catalog'){
  window.updateCalls.push(command);
  await new Promise(r=>setTimeout(r,200));
  if(window.failUpdate)throw 'Network unavailable. Try again.';
  window.updateItems.push({...window.updateItems[0],id:'node-new',version:'22.0.0',status:'available',path:null});
  return {added:1,total:2};
 }
 if(command==='check_app_update'){
  if(window.failApp)throw 'GitHub rate limit. Try again later.';
  return {channel:'dev',currentVersion:'0.1.0-beta-1',latestVersion:'0.1.0-beta-1',currentCommit:'1234567old',latestCommit:'abcdefgnew',tag:'dev',status:window.appStatus||'available',message:'A newer build is available.'};
 }
 if(command==='install_app_update'){
  window.updateCalls.push({command,...args});
  await new Promise(r=>setTimeout(r,500));
  if(window.failInstall)throw 'Installer checksum mismatch. Try again.';
  return;
 }
 if(command==='open_release_page'){window.updateCalls.push(args);return;}
 return previous(command,args);
};
"""

def run():
 with sync_playwright() as p:
  browser=p.chromium.launch()
  page=browser.new_page(viewport={'width':1280,'height':820})
  errors=[];page.on('pageerror',lambda error:errors.append(str(error)))
  page.add_init_script(SCRIPT);page.goto('http://127.0.0.1:1420')
  page.locator('[data-nav="packages"]').click()
  check=page.locator('[data-action="check-runtime-updates"]')
  expect(check).to_be_visible()
  page.evaluate('window.failUpdate=true');check.click()
  expect(check).to_be_disabled()
  expect(page.locator('#catalog-update-status')).to_contain_text('Network unavailable')
  expect(page.locator('[data-family="node"]')).to_contain_text('20.0.0')
  page.evaluate('window.failUpdate=false');check.click()
  expect(page.locator('#catalog-update-status')).to_contain_text('1 new')
  update=page.locator('[data-update-package="node-new"]')
  expect(update).to_have_text('Install update 22.0.0')
  page.locator('[data-package-version="node"]').select_option('node-old')
  update.click()
  expect(page.locator('#dialog')).to_contain_text('Install Node.js 22.0.0')
  expect(page.locator('#dialog [name="destination"]')).to_be_visible()
  expect(page.locator('#dialog')).to_contain_text('Existing versions stay')
  page.locator('#dialog [data-action="close-dialog"]').first.click()
  page.set_viewport_size({'width':800,'height':600})
  expect(check).to_be_in_viewport()
  expect(update).to_be_in_viewport()
  page.screenshot(path=str(OUT/'runtime-updates-800.png'))
  page.locator('[data-nav="settings"]').click()
  page.evaluate('window.failApp=true')
  page.locator('[data-action="check-app-update"]').click()
  expect(page.locator('#app-update-status')).to_contain_text('rate limit')
  page.evaluate('window.failApp=false')
  page.locator('[data-action="check-app-update"]').click()
  expect(page.locator('#app-update-status')).to_contain_text('newer build')
  page.locator('[data-action="open-app-release"]').click()
  assert page.evaluate('window.updateCalls.at(-1)')=={'tag':'dev'}
  install=page.locator('[data-action="install-app-update"]')
  expect(install).to_have_text('Download & install update')
  page.evaluate('window.failInstall=true')
  install.click()
  expect(install).to_be_disabled()
  expect(page.locator('[data-action="check-app-update"]')).to_be_disabled()
  # Even a second dispatched action must not start another IPC request.
  page.evaluate("document.querySelector('[data-action=install-app-update]').dispatchEvent(new MouseEvent('click',{bubbles:true}))")
  assert page.evaluate("window.updateCalls.filter(call=>call.command==='install_app_update').length")==1
  expect(page.locator('#app-update-status')).to_contain_text('Downloading')
  expect(page.locator('#app-update-status')).to_contain_text('checksum mismatch')
  expect(install).to_be_enabled()
  page.evaluate('window.failInstall=false')
  page.locator('[data-action="check-app-update"]').click()
  page.locator('#app-update-section').scroll_into_view_if_needed()
  page.screenshot(path=str(OUT/'app-updates-800.png'))
  page.set_viewport_size({'width':1280,'height':820})
  page.locator('[data-action="theme"]').first.click()
  page.locator('#app-update-section').scroll_into_view_if_needed()
  page.screenshot(path=str(OUT/'app-updates-light.png'))
  expect(page.locator('#app-update-status')).to_contain_text('newer build')
  for status in ['current','ahead','unknown']:
   page.evaluate('status=>window.appStatus=status',status)
   page.locator('[data-action="check-app-update"]').click()
   expect(install).to_have_count(0)
  expect(page.locator('[data-action="open-app-release"]')).to_be_visible()
  page.evaluate('window.appStatus="available"')
  page.locator('[data-action="check-app-update"]').click()
  install.click()
  expect(page.locator('#app-update-status')).to_contain_text('Opening Windows Setup')
  assert page.evaluate('window.updateCalls.at(-1)')=={'command':'install_app_update','commit':'abcdefgnew'}
  assert not errors,errors
  browser.close()
 print('Runtime/app update UI contracts passed')

if __name__=='__main__':run()
