"""First-launch UI contract with mocked IPC; native startup is checked on CI."""
from playwright.sync_api import sync_playwright, expect
from check_desktop_ui import OUT

SCRIPT = r"""
window.isTauri=true;
window.calls=[];window.failInfo=true;window.failSave=true;window.cancelPicker=true;
window.__TAURI_INTERNALS__={invoke:async(command,args={})=>{
 window.calls.push({command,args});
 if(command==='setup_info'){
  if(window.failInfo)throw 'Storage settings could not be read. Try again.';
  return {preferred:'C:\\Users\\Developer\\AppData\\Roaming\\id.luzyver.aevrik.desktop',application:'D:\\Aevrik'};
 }
 if(command==='pick_path')return window.cancelPicker?null:'D:\\Development data\\Aevrik';
 if(command==='complete_setup'){
  if(window.failSave)throw 'Keep the data folder separate from the application installation folder.';
  await new Promise(resolve=>window.finishSetup=resolve);
 }
 if(command==='exit_setup')return;
 if(command==='snapshot'||command==='list_packages')throw 'Workspace started before setup';
}};
"""

with sync_playwright() as p:
    browser=p.chromium.launch()
    page=browser.new_page(viewport={'width':680,'height':560})
    errors=[]
    page.on('pageerror',lambda error:errors.append(str(error)))
    page.add_init_script(SCRIPT)
    page.goto('http://127.0.0.1:1420/setup.html')
    expect(page.get_by_role('heading',name='Choose your data folder')).to_be_visible()
    expect(page.locator('.setup-heading img')).to_have_js_property('complete',True)
    assert page.locator('.setup-heading img').evaluate('(img)=>img.naturalWidth>0'), 'Setup logo failed to load'
    expect(page.locator('#setup-error')).to_contain_text('could not be read')
    expect(page.get_by_role('button',name='Open Aevrik',exact=True)).to_be_disabled()
    page.evaluate('window.failInfo=false')
    page.get_by_role('button',name='Try again',exact=True).click()
    field=page.get_by_label('Data folder',exact=True)
    default='C:\\Users\\Developer\\AppData\\Roaming\\id.luzyver.aevrik.desktop'
    expect(field).to_have_value(default)
    page.get_by_role('button',name='Browse…',exact=True).click()
    expect(field).to_have_value(default)
    page.evaluate('window.cancelPicker=false')
    page.get_by_role('button',name='Browse…',exact=True).click()
    expect(field).to_have_value('D:\\Development data\\Aevrik')
    page.get_by_role('button',name='Open Aevrik',exact=True).click()
    expect(page.locator('#setup-error')).to_contain_text('separate')
    expect(field).to_have_value('D:\\Development data\\Aevrik')
    expect(page.get_by_role('button',name='Open Aevrik',exact=True)).to_be_enabled()
    page.screenshot(path=str(OUT/'first-launch-error.png'))
    page.get_by_role('button',name='Use default folder',exact=True).click()
    expect(field).to_have_value(default)
    field.fill('D:\\AevrikData')
    page.evaluate("document.documentElement.dataset.theme='light'")
    page.screenshot(path=str(OUT/'first-launch-light.png'))
    page.set_viewport_size({'width':520,'height':520})
    page.evaluate("document.documentElement.dataset.theme='dark'")
    expect(page.get_by_role('button',name='Open Aevrik',exact=True)).to_be_in_viewport()
    assert page.evaluate('document.documentElement.scrollWidth<=innerWidth')
    page.screenshot(path=str(OUT/'first-launch-dark-compact.png'))
    page.get_by_role('button',name='Exit',exact=True).click()
    assert any(c['command']=='exit_setup' for c in page.evaluate('window.calls'))
    page.evaluate('window.failSave=false')
    page.get_by_role('button',name='Open Aevrik',exact=True).click()
    expect(page.get_by_role('button',name='Opening…',exact=True)).to_be_disabled()
    expect(field).to_be_disabled()
    expect(page.get_by_role('button',name='Browse…',exact=True)).to_be_disabled()
    expect(page.get_by_role('button',name='Exit',exact=True)).to_be_disabled()
    submits=[c for c in page.evaluate('window.calls') if c['command']=='complete_setup']
    assert len(submits)==2 and submits[-1]['args']['directory']=='D:\\AevrikData'
    page.evaluate('window.finishSetup()')
    expect(page.locator('#setup-status')).to_contain_text('Opening your workspace')
    assert not any(c['command'] in ('snapshot','list_packages') for c in page.evaluate('window.calls'))
    assert not errors,errors
    browser.close()
print('First launch: info retry, browse/cancel, inline errors, default reset, exit, busy guard and compact themes passed.')
