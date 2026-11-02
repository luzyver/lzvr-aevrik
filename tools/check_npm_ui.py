"""npm setup through the project UI, with a mocked IPC boundary."""
from playwright.sync_api import sync_playwright, expect
from check_desktop_ui import MOCK, OUT

script=MOCK+r'''
window.fixture.projects=[{id:'portfolio',name:'Portfolio',path:'D:\\Projects\\My portfolio',domain:'portfolio.test',runtime:'',serviceId:null,state:'stopped'}];
window.npmCalls=[];
const original=window.__TAURI_INTERNALS__.invoke;
window.__TAURI_INTERNALS__.invoke=async(command,args={})=>{
 if(command==='project_scripts'){
  if(window.failScripts)throw 'Cannot read package.json in this project';
  if(window.slowScripts)await new Promise(r=>setTimeout(r,1000));
  return {scripts:window.emptyScripts?[]:[{name:'build',command:'astro build'},{name:'dev',command:'astro dev <example>'}],nodes:[{label:'Node.js 26',executable:'D:\\Node runtime\\node.exe'}]};
 }
 if(command==='register_npm_service'){
  window.npmCalls.push(args.input);
  if(window.failSave)throw 'The selected npm script is missing. Reload the project scripts.';
  window.fixture.services.push({id:'npm-service',name:'Portfolio · dev',cwd:'D:\\Projects\\My portfolio',executable:'D:\\Node runtime\\node.exe',args:['npm-cli.js','run','--','dev'],npmScript:'dev',state:'stopped',pid:null,exitCode:null,port:4321});
  window.fixture.projects[0].serviceId='npm-service';return;
 }
 return original(command,args);
};
'''

with sync_playwright() as p:
    browser=p.chromium.launch()
    page=browser.new_page(viewport={'width':1280,'height':820})
    errors=[]
    page.on('pageerror',lambda e:errors.append(str(e)))
    page.add_init_script(script)
    page.goto('http://127.0.0.1:1420')
    page.locator('[data-nav="projects"]').click()
    page.locator('[data-select-project="portfolio"]').click()
    page.locator('[data-action="setup-npm"]').click()
    form=page.locator('#npm-service')
    expect(form.locator('[name="script"]')).to_have_value('dev')
    expect(form.locator('[data-script-preview]')).to_have_text('astro dev <example>')
    expect(form.locator('example')).to_have_count(0)
    form.locator('[name="port"]').fill('4321')
    page.screenshot(path=str(OUT/'npm-project-setup-dark.png'))
    page.set_viewport_size({'width':800,'height':600})
    page.screenshot(path=str(OUT/'npm-project-setup-800.png'))
    assert page.evaluate('document.documentElement.scrollWidth<=innerWidth')
    page.evaluate('window.failSave=true')
    form.locator('[type="submit"]').click()
    expect(form.locator('[role="alert"]')).to_contain_text('script is missing')
    page.evaluate('window.failSave=false')
    form.locator('[type="submit"]').click()
    expect(page.locator('#dialog')).not_to_be_visible()
    expect(page.locator('[data-action="start"]')).to_be_visible()
    assert page.evaluate('window.npmCalls.at(-1)')=={'projectId':'portfolio','script':'dev','nodeExecutable':'D:\\Node runtime\\node.exe','port':4321}
    assert page.evaluate('window.fixture.services[0].state')=='stopped'
    # Reload into an unlinked project to exercise errors and an empty manifest.
    page.reload()
    page.locator('[data-nav="projects"]').click()
    page.locator('[data-select-project="portfolio"]').click()
    page.evaluate('window.failScripts=true')
    page.locator('[data-action="setup-npm"]').click()
    expect(page.locator('#npm-service [role="alert"]')).to_contain_text('package.json')
    page.evaluate('window.failScripts=false;window.emptyScripts=true')
    page.locator('#npm-service [data-npm-reload]').click()
    expect(page.locator('#npm-service')).to_contain_text('No scripts')
    expect(page.locator('#npm-service [type="submit"]')).to_be_disabled()
    page.locator('#npm-service [data-npm-close]').first.click()
    page.evaluate('window.emptyScripts=false;window.slowScripts=true')
    page.locator('[data-action="setup-npm"]').click()
    page.locator('#npm-service [data-npm-close]').first.click()
    page.wait_for_timeout(1100)
    expect(page.locator('#dialog')).not_to_be_visible()
    assert not errors,errors
    browser.close()
print('npm project setup, script preview, save/link, errors, empty and stale-dialog states passed.')
