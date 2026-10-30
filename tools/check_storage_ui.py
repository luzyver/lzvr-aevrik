"""UI contract for destination selection and explicit uninstall confirmation; IPC is mocked."""
from playwright.sync_api import sync_playwright, expect
from check_desktop_ui import MOCK, OUT

script=MOCK+r"""
const original=window.__TAURI_INTERNALS__.invoke;
window.calls=[];
window.runtime={id:'caddy',family:'caddy',name:'Caddy',version:'2.11.7',category:'Web server',description:'Test fixture',source:'official.test',status:'available',bytes:0,total:null,error:null,path:null};
window.__TAURI_INTERNALS__.invoke=async(command,args={})=>{
 if(command==='list_packages')return [structuredClone(window.runtime)];
 if(command==='package_install_root')return 'C:\\Aevrik\\runtimes';
 if(command==='pick_path')return 'D:\\Local tools\\runtimes';
 if(command==='install_package'){
  window.calls.push([command,args]);
  Object.assign(window.runtime,{status:'installed',path:args.destination+'\\caddy\\2.11.7\\caddy.exe',installDirectory:args.destination+'\\caddy\\2.11.7'});return;
 }
 if(command==='uninstall_package'){
  window.calls.push([command,args]);
  if(window.inUse)throw 'Runtime is referenced by a service';
  Object.assign(window.runtime,{status:'available',path:null,installDirectory:null});return;
 }
 return original(command,args);
};
"""
with sync_playwright() as p:
 browser=p.chromium.launch();page=browser.new_page(viewport={'width':1280,'height':820})
 errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
 page.add_init_script(script);page.goto('http://127.0.0.1:1420')
 page.locator('[data-nav="packages"]').click()
 page.locator('[data-install="caddy"]').click()
 form=page.locator('#package-install')
 expect(form).to_be_visible()
 expect(form.locator('[name="destination"]')).to_have_value('C:\\Aevrik\\runtimes')
 page.locator('[data-action="close-dialog"]').first.click()
 assert page.evaluate('window.calls')==[]
 page.locator('[data-install="caddy"]').click()
 form.locator('[data-pick="directory"]').click()
 expect(form.locator('[name="destination"]')).to_have_value('D:\\Local tools\\runtimes')
 page.screenshot(path=str(OUT/'install-location-0.1.0-beta-1.png'))
 page.set_viewport_size({'width':800,'height':600})
 expect(form.locator('[type="submit"]')).to_be_in_viewport()
 page.screenshot(path=str(OUT/'install-location-800-0.1.0-beta-1.png'))
 page.set_viewport_size({'width':1280,'height':820})
 form.locator('[type="submit"]').click()
 expect(page.locator('[data-uninstall="caddy"]')).to_be_visible()
 assert page.evaluate('window.calls[0]')==['install_package',{'id':'caddy','destination':'D:\\Local tools\\runtimes'}]
 page.locator('[data-uninstall="caddy"]').click()
 removal=page.locator('#package-uninstall')
 expect(removal).to_contain_text('D:\\Local tools\\runtimes\\caddy\\2.11.7')
 page.locator('[data-action="close-dialog"]').first.click()
 assert len(page.evaluate('window.calls'))==1
 page.evaluate('window.inUse=true')
 page.locator('[data-uninstall="caddy"]').click()
 removal.locator('[type="submit"]').click()
 expect(removal.locator('#form-error')).to_contain_text('referenced by a service')
 expect(page.locator('[data-configure="caddy"]')).to_be_visible()
 page.evaluate('window.inUse=false')
 page.screenshot(path=str(OUT/'uninstall-confirmation-0.1.0-beta-1.png'))
 removal.locator('[type="submit"]').click()
 expect(page.locator('#dialog')).not_to_be_visible()
 expect(page.locator('[data-install="caddy"]')).to_be_visible()
 # A stopped registration can be removed without a package operation.
 page.evaluate("window.fixture.services=[{id:'test-service',name:'Test runtime',kind:'custom',executable:'C:\\\\test.exe',cwd:'C:\\\\projects',args:[],port:null,state:'stopped',pid:null,exitCode:null}]")
 page.locator('[data-nav="services"]').click()
 button=page.locator('[data-action="remove-service"]')
 expect(button).to_be_visible(timeout=5000)
 button.click()
 expect(page.locator('#service-remove')).to_contain_text('will stay on disk')
 page.locator('[data-action="close-dialog"]').first.click()
 assert page.evaluate('window.fixture.services.length')==1
 button.click()
 page.locator('#service-remove [type="submit"]').click()
 expect(page.locator('#dialog')).not_to_be_visible()
 expect(page.get_by_text('Your local stack starts here',exact=True)).to_be_visible()
 assert not errors,errors
 browser.close()
print('Install location, compact dialog, cancel, browse, exact IPC destination, uninstall confirmation, in-use errors and service removal passed.')
