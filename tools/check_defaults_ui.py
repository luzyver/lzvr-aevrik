"""Terminal default UI contracts; mocked IPC never changes the host PATH."""
from playwright.sync_api import sync_playwright, expect
from check_desktop_ui import MOCK, OUT

SCRIPT = MOCK + r"""
window.defaultItems=[
 {id:'node-old',family:'node',name:'Node.js',version:'20.0.0',category:'Language',description:'JavaScript runtime',source:'nodejs.org',status:'installed',bytes:0,total:null,path:'D:\\Runtimes\\Node 20\\node.exe',error:null,terminalDefault:{supported:true,active:true,onPath:true}},
 {id:'node-new',family:'node',name:'Node.js',version:'22.0.0',category:'Language',description:'JavaScript runtime',source:'nodejs.org',status:'installed',bytes:0,total:null,path:'D:\\Runtimes\\Node 22\\node.exe',error:null,terminalDefault:{supported:true,active:false,onPath:false}},
 {id:'node-download',family:'node',name:'Node.js',version:'24.0.0',category:'Language',description:'JavaScript runtime',source:'nodejs.org',status:'available',bytes:0,total:null,path:null,error:null,terminalDefault:{supported:true,active:false,onPath:false}}
];
window.defaultCalls=[];
const previous=window.__TAURI_INTERNALS__.invoke;
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
 if(command==='list_packages')return structuredClone(window.defaultItems);
 if(command==='preview_runtime_default')return {scope:window.defaultScope||'machine',removing:args.removing,executable:'D:\\Runtimes\\Node 22\\node.exe',currentExecutable:'D:\\Runtimes\\Node 20\\node.exe',directories:['D:\\Runtimes\\Node 22'],fingerprint:'reviewed'};
 if(command==='apply_runtime_default'){
  window.defaultCalls.push(args);
  await new Promise(r=>setTimeout(r,300));
  if(window.failDefault)throw 'Windows denied the PATH change. Run Aevrik as Administrator.';
  window.defaultItems.forEach(p=>{p.terminalDefault.active=!args.removing&&p.id===args.id;});
  window.defaultItems.find(p=>p.id===args.id).terminalDefault.onPath=!args.removing;
  return;
 }
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
  expect(page.locator('[data-default]')).to_have_count(0) # available version
  page.locator('[data-package-version="node"]').select_option('node-old')
  expect(page.locator('[data-family="node"]')).to_contain_text('Terminal default')
  expect(page.locator('[data-uninstall="node-old"]')).to_be_disabled()
  page.locator('[data-package-version="node"]').select_option('node-new')
  page.locator('[data-default="node-new"]').click()
  expect(page.locator('#dialog')).to_contain_text('All Windows users')
  expect(page.locator('#dialog')).to_contain_text('Administrator')
  expect(page.locator('#dialog')).to_contain_text('Restart your terminal')
  page.locator('#dialog [data-action="close-dialog"]').first.click()
  assert page.evaluate('window.defaultCalls.length')==0
  page.locator('[data-default="node-new"]').click()
  page.evaluate('window.failDefault=true')
  submit=page.locator('#package-default [type="submit"]')
  submit.click();expect(submit).to_be_disabled()
  page.keyboard.press('Escape')
  expect(page.locator('#dialog')).to_be_visible()
  expect(page.locator('#form-error')).to_contain_text('Administrator')
  expect(submit).to_be_enabled()
  page.set_viewport_size({'width':800,'height':600})
  expect(submit).to_be_in_viewport()
  expect(page.locator('#form-error')).to_be_in_viewport(ratio=1)
  page.screenshot(path=str(OUT/'runtime-default-error-800.png'),animations='disabled')
  page.evaluate('window.failDefault=false');submit.click()
  expect(page.locator('#dialog')).not_to_be_visible()
  expect(page.locator('[data-family="node"] .package-status')).to_have_text('Terminal default')
  expect(page.locator('[data-uninstall="node-new"]')).to_be_disabled()
  expect(page.locator('[data-clear-default="node-new"]')).to_be_in_viewport()
  page.screenshot(path=str(OUT/'runtime-default-800.png'),animations='disabled')
  assert page.evaluate('window.defaultCalls.at(-1)')=={'id':'node-new','scope':'machine','removing':False,'fingerprint':'reviewed'}
  page.locator('[data-clear-default="node-new"]').click()
  expect(page.locator('#dialog')).to_contain_text('Remove from PATH')
  page.locator('#package-default [type="submit"]').click()
  expect(page.locator('#dialog')).not_to_be_visible()
  expect(page.locator('[data-uninstall="node-new"]')).to_be_enabled()
  assert page.evaluate('window.defaultCalls.at(-1).removing') is True
  page.set_viewport_size({'width':1280,'height':820})
  page.locator('[data-action="theme"]').first.click()
  page.evaluate("window.defaultScope='user'")
  page.locator('[data-default="node-new"]').click()
  expect(page.locator('#dialog')).to_contain_text('This Windows account')
  expect(page.locator('#dialog')).not_to_contain_text('Administrator required')
  page.screenshot(path=str(OUT/'runtime-default-light.png'),animations='disabled')
  assert not errors,errors
  browser.close()
 print('Terminal default UI contracts passed')

if __name__=='__main__':run()
