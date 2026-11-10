"""Frontend-only contract tests with a mocked IPC boundary; not native proof."""
import json
import re
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
 if(command==='edit_project')Object.assign(s.projects.find(p=>p.id===args.id),{name:args.name,path:args.path,domain:args.domain});
 if(command==='remove_project')s.projects=s.projects.filter(p=>p.id!==args.id);
 if(command==='update_service'){
  const service=s.services.find(x=>x.id===args.id);
  if(service.state==='running')throw 'Stop this service before editing it.';
  Object.assign(service,args.input);
 }
 if(command==='control_all'){
  for(const service of s.services){service.state=args.action==='start'?'running':'stopped';service.pid=args.action==='start'?7654:null;}
  return [];
 }
 if(command==='set_restore_services')s.restoreServices=args.enabled;
 if(command==='startup_enabled')return !!window.startup;
 if(command==='set_startup')return window.startup=args.enabled;
 if(command==='open_target')(window.opened??=[]).push(args.kind+':'+args.id);
 if(command==='exit_app')window.exited=true;
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
  expect(page.locator("#breadcrumb")).to_have_text("Projects")
  expect(page.get_by_text("Make room for your next project",exact=True)).to_be_visible()
  expect(page.locator(".preview-label")).to_have_text(json.loads((ROOT/"package.json").read_text())["version"]+" · Desktop")
  page.locator('[data-nav="services"]').click()
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
  # Rebuilding the page keeps keyboard focus on the control that was used.
  assert page.evaluate("document.activeElement?.dataset.action")=="stop"
  page.wait_for_timeout(1200)
  assert page.evaluate("document.activeElement?.dataset.action")=="stop"
  # A running service cannot be edited; the row offers a one-click stop and its local URL.
  expect(page.locator('[data-action="edit-service"]')).to_be_disabled()
  page.locator('.service-table [data-open="service-url"]').click()
  assert page.evaluate("window.opened")==["service-url:test-service"]
  page.locator('.service-table [data-quick="stop"]').click()
  expect(page.locator("#service-count")).to_have_text("0 services running")
  # A process that ended on its own is reported as a failure, not as a quiet stop.
  page.evaluate("Object.assign(window.fixture.services[0],{state:'exited',exitCode:3})")
  expect(page.locator(".service-table .badge.failed")).to_have_text("Exited · code 3")
  expect(page.locator(".detail-pane .notice.failure")).to_contain_text("exit code 3")
  page.evaluate("Object.assign(window.fixture.services[0],{state:'stopped',exitCode:null})")
  page.locator('[data-action="edit-service"]').click()
  edit=page.locator("#service-edit")
  expect(edit.locator('[name="executable"]')).to_have_value("C:\\Tools\\server.exe")
  edit.locator('[name="name"]').fill("Local HTTP 2")
  edit.locator('[type="submit"]').click()
  expect(page.locator("#dialog")).not_to_be_visible()
  expect(page.locator(".detail-pane h2")).to_have_text("Local HTTP 2")
  page.locator('[data-action="start-all"]').click()
  expect(page.locator("#service-count")).to_have_text("1 services running")
  expect(page.locator('[data-action="start-all"]')).to_have_count(0)
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
  page.locator('.project-row [data-open="project-folder"]').click()
  assert page.evaluate("window.opened").pop()=="project-folder:test-project"
  page.locator('[data-action="edit-project"]').click()
  page.locator('#project-edit [name="name"]').fill("Renamed project")
  page.locator('#project-edit [type="submit"]').click()
  expect(page.locator(".project-row strong")).to_have_text("Renamed project")
  # Ctrl+K finds projects, services and commands, and runs the highlighted one on Enter.
  page.keyboard.press("Control+k")
  expect(page.locator(".command-result").first).to_have_attribute("aria-selected","true")
  page.locator("#command-query").fill("output")
  expect(page.locator(".command-result")).to_have_count(1)
  page.keyboard.press("Enter")
  expect(page.locator("#dialog")).not_to_be_visible()
  expect(page.locator("#breadcrumb")).to_have_text("Output")
  expect(page.locator("#log-service option")).to_have_count(2)
  page.locator("#log-service").select_option("test-service")
  expect(page.locator(".log-lines")).to_contain_text("Fixture output")
  # Exit lists what it will stop and waits for confirmation while services run.
  page.locator('[data-nav="settings"]').click()
  expect(page.locator(".page-footnote")).to_contain_text("Aevrik "+json.loads((ROOT/"package.json").read_text())["version"])
  page.locator('[data-action="toggle-startup"]').click()
  expect(page.locator('[data-action="toggle-startup"]')).to_have_attribute("aria-checked","true")
  page.locator('[data-action="toggle-restore"]').click()
  expect(page.locator('[data-action="toggle-restore"]')).to_have_attribute("aria-checked","true")
  page.locator('[data-action="exit-app"]').click()
  expect(page.locator("#exit-confirm")).to_contain_text("Local HTTP 2")
  assert not page.evaluate("window.exited")
  page.locator('#exit-confirm [data-action="close-dialog"]').last.click()
  expect(page.locator("#dialog")).not_to_be_visible()
  # The tray's Exit reaches the same confirmation instead of stopping services unasked.
  page.evaluate("window.dispatchEvent(new Event('aevrik-exit-requested'))")
  expect(page.locator("#exit-confirm")).to_contain_text("Local HTTP 2")
  assert not page.evaluate("window.exited")
  page.locator('#exit-confirm [data-action="close-dialog"]').last.click()
  # With the palette open, the tray's Exit replaces it: Enter must not run a palette command.
  page.keyboard.press("Control+k")
  page.locator("#command-query").fill("output")
  expect(page.locator(".command-result")).to_have_count(1)
  page.evaluate("window.dispatchEvent(new Event('aevrik-exit-requested'))")
  expect(page.locator("#exit-confirm")).to_be_visible()
  expect(page.locator("#dialog")).not_to_have_class(re.compile("command-dialog"))
  page.locator('#exit-confirm [data-action="close-dialog"]').last.focus()
  page.keyboard.press("Enter")
  expect(page.locator("#dialog")).not_to_be_visible()
  expect(page.locator("#breadcrumb")).to_have_text("Settings")
  assert not page.evaluate("window.exited")
  # Another open dialog keeps its input; the confirmation follows once it closes.
  page.locator('[data-nav="projects"]').click()
  page.locator('[data-action="edit-project"]').click()
  page.evaluate("window.dispatchEvent(new Event('aevrik-exit-requested'))")
  page.wait_for_timeout(300)
  expect(page.locator("#project-edit")).to_be_visible()
  page.locator('#project-edit [data-action="close-dialog"]').last.click()
  expect(page.locator("#exit-confirm")).to_be_visible()
  page.locator('#exit-confirm [data-action="close-dialog"]').last.click()
  expect(page.locator("#dialog")).not_to_be_visible()
  # Sidebar focus survives a page change and the live refreshes that follow.
  page.locator('[data-nav="logs"]').focus()
  page.keyboard.press("Enter")
  expect(page.locator("#breadcrumb")).to_have_text("Output")
  assert page.evaluate("document.activeElement?.dataset.nav")=="logs"
  page.evaluate("window.fixture.logs.push({timestamp:Date.now(),service:'test-service',level:'INFO',message:'Later line'})")
  expect(page.locator(".log-lines")).to_contain_text("Later line")
  assert page.evaluate("document.activeElement?.dataset.nav")=="logs"
  page.locator('[data-nav="settings"]').click()
  page.locator("#theme-select").select_option("system")
  expect(page.locator("html")).to_have_attribute("data-theme","light")
  page.locator("#theme-select").select_option("dark")
  expect(page.locator("html")).to_have_attribute("data-theme","dark")
  page.locator('[data-nav="projects"]').click()
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
  # With nothing running, Exit needs no confirmation.
  page.locator('[data-nav="projects"]').click()
  page.locator('[data-action="remove-project"]').click()
  page.locator('#project-remove [type="submit"]').click()
  expect(page.locator(".project-row")).to_have_count(0)
  page.locator('[data-nav="settings"]').click()
  page.locator('[data-action="exit-app"]').click()
  assert page.evaluate("window.exited")
  assert not errors,errors
  browser.close()
 (OUT/"frontend-verification.json").write_text(json.dumps({"status":"passed","kind":"Mocked IPC frontend tests","checks":["browser without engine","empty native state","registration forms","backend error propagation","authoritative PID/status","escaped output","filtering","project properties","light theme","800px desktop"],"consoleErrors":errors},indent=2))
 print("Frontend contract and layout tests passed; no page errors.")
if __name__=="__main__":run()
