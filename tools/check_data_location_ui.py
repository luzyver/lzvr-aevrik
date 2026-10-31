"""Settings behavior at the IPC boundary; native persistence is tested separately."""
from playwright.sync_api import sync_playwright, expect
from check_desktop_ui import MOCK, OUT

SCRIPT = MOCK + r"""
const previousInvoke = window.__TAURI_INTERNALS__.invoke;
window.locationInfo={current:'C:\\AevrikData',next:null,application:'D:\\Aevrik',overridden:false};
window.savedLocations=[];
window.__TAURI_INTERNALS__.invoke=async(command,args={})=>{
 if(command==='storage_info'){
  if(window.failRead)throw 'Cannot read storage settings';
  return structuredClone(window.locationInfo);
 }
 if(command==='pick_path')return window.cancelPicker?null:'D:\\Development data\\Aevrik';
 if(command==='set_data_directory'){
  if(window.failSave)throw 'Access denied. Choose a writable folder.';
  window.savedLocations.push(args.directory);
  window.locationInfo.next=args.directory===window.locationInfo.current?null:args.directory;
  return structuredClone(window.locationInfo);
 }
 return previousInvoke(command,args);
};
"""

with sync_playwright() as p:
    browser=p.chromium.launch()
    page=browser.new_page(viewport={'width':1280,'height':820})
    errors=[]
    page.on('pageerror',lambda error:errors.append(str(error)))
    page.add_init_script(SCRIPT)
    page.goto('http://127.0.0.1:1420')
    expect(page.locator('#engine-status')).to_have_text('Engine connected')
    page.locator('[data-nav="settings"]').click()
    expect(page.locator('#storage-current')).to_have_text('C:\\AevrikData')
    expect(page.locator('#storage-application')).to_have_text('D:\\Aevrik')
    page.locator('[data-action="change-data-folder"]').click()
    form=page.locator('#data-location')
    expect(form).to_contain_text('Existing files stay where they are')
    page.evaluate('window.cancelPicker=true')
    form.locator('[data-pick]').click()
    expect(form.locator('[name="directory"]')).to_have_value('C:\\AevrikData')
    page.evaluate('window.cancelPicker=false')
    form.locator('[data-pick]').click()
    expect(form.locator('[name="directory"]')).to_have_value('D:\\Development data\\Aevrik')
    form.locator('[data-action="close-dialog"]').first.click()
    assert page.evaluate('window.savedLocations') == []
    page.locator('[data-action="change-data-folder"]').click()
    form.locator('[data-pick]').click()
    page.evaluate('window.failSave=true')
    form.locator('[type="submit"]').click()
    expect(form.locator('#form-error')).to_contain_text('Access denied')
    expect(form.locator('[type="submit"]')).to_be_enabled()
    page.set_viewport_size({'width':800,'height':600})
    expect(form.locator('[type="submit"]')).to_be_in_viewport()
    page.screenshot(path=str(OUT/'data-folder-dialog-800.png'))
    page.evaluate('window.failSave=false')
    form.locator('[type="submit"]').click()
    expect(page.locator('#dialog')).not_to_be_visible()
    expect(page.locator('#storage-current')).to_have_text('C:\\AevrikData')
    expect(page.locator('#storage-next')).to_contain_text('D:\\Development data\\Aevrik')
    expect(page.locator('#storage-next')).to_contain_text('Close and reopen')
    expect(page.locator('[data-action="cancel-data-change"]')).to_be_in_viewport()
    assert page.evaluate('window.savedLocations') == ['D:\\Development data\\Aevrik']
    page.screenshot(path=str(OUT/'data-folder-settings-800.png'))
    page.set_viewport_size({'width':1280,'height':820})
    page.screenshot(path=str(OUT/'data-folder-settings-dark.png'))
    page.locator('[data-action="theme"]').first.click()
    page.screenshot(path=str(OUT/'data-folder-settings-light.png'))
    page.locator('[data-action="cancel-data-change"]').click()
    expect(page.locator('#storage-next')).to_have_count(0)
    page.evaluate('window.locationInfo.overridden=true')
    page.locator('[data-nav="services"]').click()
    page.locator('[data-nav="settings"]').click()
    expect(page.locator('#storage-section')).to_contain_text('AEVRIK_DATA_DIR')
    expect(page.locator('[data-action="change-data-folder"]')).to_be_disabled()
    page.evaluate('window.failRead=true')
    page.locator('[data-nav="services"]').click()
    page.locator('[data-nav="settings"]').click()
    expect(page.locator('#storage-section')).to_contain_text('Cannot read storage settings')
    page.evaluate('window.failRead=false')
    page.locator('[data-action="reload-storage"]').click()
    expect(page.locator('#storage-current')).to_have_text('C:\\AevrikData')
    assert not errors,errors
    browser.close()
print('Data-folder Settings: cancel, browse, errors, save, pending/current distinction, undo, override and compact layout passed.')
