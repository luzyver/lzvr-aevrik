"""Frontend-only contract tests with a mocked IPC boundary; not native proof."""
import json
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"previews"/"desktop"
OUT.mkdir(parents=True,exist_ok=True)
MOCK=r"""
window.isTauri=true;
window.fixture={projects:[],services:[],logs:[],theme:'dark'};
window.__TAURI_INTERNALS__={invoke:async(command,args={})=>{
 const s=window.fixture;
 if(command==='snapshot')return structuredClone(s);
 if(command==='storage_info')return {current:'C:\\AevrikData',next:null,application:'C:\\Aevrik',overridden:false};
 if(command==='list_packages')return [];
 if(command==='package_install_root')return 'C:\\Aevrik\\runtimes';
 if(command==='register_service'){
  if(!args.input.executable.endsWith('.exe'))throw 'Choose an existing executable.';
  s.services.push({...args.input,id:'test-service',state:'stopped',pid:null,exitCode:null});
 }
 if(command==='remove_service')s.services=s.services.filter(x=>x.id!==args.id);
 if(command==='control_service'){
  if(window.rejectControl)throw 'Port 8080 is unavailable.';
  const service=s.services.find(x=>x.id===args.id);
  service.state=args.action==='stop'?'stopped':'running';service.pid=args.action==='stop'?null:7654;
  s.logs.push({timestamp:Date.now(),service:service.id,level:'INFO',message:'Fixture output <script>unsafe</script>'});
 }
 if(command==='add_project'){
  if(s.projects.some(p=>p.domain===args.input.domain))throw 'A project already uses that name or domain.';
  s.projects.push({...args.input,id:'test-project',state:'stopped'});
 }
 if(command==='update_project')Object.assign(s.projects[0],{runtime:args.runtime,serviceId:args.serviceId});
 if(command==='set_theme')s.theme=args.theme;
 if(command==='clear_logs')s.logs=[];
 if(command==='pick_path')return null;
}};
"""
def run():
 errors=[]
 with sync_playwright() as p:
  browser=p.chromium.launch()
  page=browser.new_page(viewport={"width":1280,"height":820})
  page.on("pageerror",lambda error:errors.append(str(error)))
  page.goto("http://127.0.0.1:1420")
  expect(page.get_by_text("Open Aevrik as a desktop application",exact=True)).to_be_visible()
  assert not page.locator(".badge.running").count()
  page.add_init_script(MOCK)
  page.reload()
  expect(page.get_by_text("Your local stack starts here",exact=True)).to_be_visible()
  page.locator('[data-action="add-service"]').first.click()
  form=page.locator("#registration")
  form.locator('[name="name"]').fill("Local HTTP")
  form.locator('[name="executable"]').fill("C:\\Tools\\server.exe")
  form.locator('[name="cwd"]').fill("C:\\Projects")
  form.locator('[name="port"]').fill("8080")
  form.locator('[type="submit"]').click()
  expect(page.locator("#dialog")).not_to_be_visible()
  page.evaluate("window.rejectControl=true")
  page.locator('[data-action="start"]').click()
  expect(page.locator("#toast")).to_contain_text("Port 8080 is unavailable")
  expect(page.locator("#service-count")).to_have_text("0 services running")
  page.evaluate("window.rejectControl=false")
  page.locator('[data-action="start"]').click()
  expect(page.locator("#service-count")).to_have_text("1 services running")
  expect(page.locator(".detail-pane")).to_contain_text("7654")
  expect(page.locator(".log-lines")).to_contain_text("<script>unsafe</script>")
  assert page.locator(".log-lines script").count()==0
  page.screenshot(path=str(OUT/"ui-dark.png"))
  page.locator('#search').fill("missing")
  expect(page.get_by_text("No matching services",exact=True)).to_be_visible()
  page.locator('#search').fill("")
  page.locator('[data-nav="projects"]').click()
  page.locator('[data-action="add-project"]').first.click()
  form.locator('[name="name"]').fill("Example project")
  form.locator('[name="path"]').fill("C:\\Projects\\Example")
  form.locator('[name="domain"]').fill("example.test")
  form.locator('[type="submit"]').click()
  expect(page.locator(".project-row")).to_have_count(1)
  page.locator('#project-properties [name="runtime"]').fill("PHP 8.4")
  page.locator('#project-properties [type="submit"]').click()
  expect(page.locator('#project-properties [name="runtime"]')).to_have_value("PHP 8.4")
  page.locator('[data-action="theme"]').first.click()
  expect(page.locator("html")).to_have_attribute("data-theme","light")
  page.screenshot(path=str(OUT/"ui-light-projects.png"))
  page.set_viewport_size({"width":800,"height":600})
  page.locator('[data-nav="services"]').click()
  expect(page.locator('[data-action="stop"]')).to_be_visible()
  page.screenshot(path=str(OUT/"ui-800.png"))
  assert page.evaluate("document.documentElement.scrollWidth<=innerWidth")
  page.locator('[data-action="stop"]').click()
  expect(page.locator("#service-count")).to_have_text("0 services running")
  assert not errors,errors
  browser.close()
 (OUT/"frontend-verification.json").write_text(json.dumps({"status":"passed","kind":"Mocked IPC frontend tests","checks":["browser without engine","empty native state","registration forms","backend error propagation","authoritative PID/status","escaped output","filtering","project properties","light theme","800px desktop"],"consoleErrors":errors},indent=2))
 print("Frontend contract and layout tests passed; no page errors.")
if __name__=="__main__":run()
