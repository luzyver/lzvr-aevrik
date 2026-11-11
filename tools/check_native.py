"""Smoke-test the actual Windows executable and Rust IPC with isolated test data."""
import argparse, json, os, socket, subprocess, tempfile, time
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from native_browser import connect_native, wait_for_debugger_exit, wait_for_page
from native_lifecycle import exit_application, restore_from_tray, wait_visible

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"previews"/"desktop"
OUT.mkdir(parents=True,exist_ok=True)

def free_port():
 with socket.socket() as sock:
  sock.bind(("127.0.0.1",0))
  return sock.getsockname()[1]

def run(executable=None):
 data=Path(tempfile.mkdtemp(prefix="aevrik-native-"))
 debug_port=free_port()
 env=os.environ.copy()
 env["AEVRIK_DATA_DIR"]=str(data)
 env["WEBVIEW2_USER_DATA_FOLDER"]=str(data/"webview")
 env["WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS"]=f"--remote-debugging-port={debug_port}"
 startup=subprocess.STARTUPINFO()
 startup.dwFlags|=subprocess.STARTF_USESHOWWINDOW
 startup.wShowWindow=0
 exe=Path(executable).resolve() if executable else ROOT/"target"/"release"/"aevrik.exe"
 process=None
 errors=[]
 checks=[]
 with sync_playwright() as p:
  try:
   process=subprocess.Popen([str(exe)],cwd=data,env=env,startupinfo=startup)
   browser=connect_native(p,debug_port,process)
   page=wait_for_page(browser)
   page.on("pageerror",lambda error:errors.append(str(error)))
   expect(page.locator("#engine-status")).to_have_text("Engine connected",timeout=20000)
   expect(page.get_by_text("Make room for your next project",exact=True)).to_be_visible()
   page.locator('[data-nav="services"]').click()
   expect(page.get_by_text("Your local stack starts here",exact=True)).to_be_visible()
   assert page.locator(".window-title img").evaluate("(img)=>img.complete && img.naturalWidth>0")
   checks.append("Release executable opens with a real empty Rust snapshot")
   page.screenshot(path=str(OUT/"native-empty.png"))
   page.locator('[data-action="add-service"]').first.click()
   form=page.locator("#registration")
   form.locator('[name="name"]').fill("Smoke HTTP process")
   fixture=ROOT/"target"/"debug"/"process-fixture.exe"
   port=free_port()
   form.locator('[name="executable"]').fill(str(fixture))
   form.locator('[name="cwd"]').fill(str(data))
   form.locator('[name="args"]').fill(str(port))
   form.locator('[name="port"]').fill(str(port))
   form.locator('[type="submit"]').click()
   expect(page.locator("#dialog")).not_to_be_visible()
   with socket.socket() as occupied:
    occupied.bind(("127.0.0.1",port))
    occupied.listen()
    page.locator('[data-action="start"]').click()
    expect(page.locator("#toast")).to_contain_text("unavailable")
    expect(page.locator("#service-count")).to_have_text("0 services running")
   page.locator('[data-action="start"]').click()
   expect(page.locator("#service-count")).to_have_text("1 services running")
   expect(page.locator(".log-lines")).to_contain_text("fixture ready",timeout=10000)
   snapshot=page.evaluate("window.__TAURI_INTERNALS__.invoke('snapshot')")
   assert snapshot["services"][0]["pid"]>0
   with socket.create_connection(("127.0.0.1",port),timeout=2):pass
   page.locator('[data-action="restart"]').click()
   expect(page.locator("#service-count")).to_have_text("1 services running")
   restarted=page.evaluate("window.__TAURI_INTERNALS__.invoke('snapshot')")
   assert restarted["services"][0]["pid"]!=snapshot["services"][0]["pid"]
   checks.append("Actual service registration, port conflict, start, output, PID and restart")
   page.screenshot(path=str(OUT/"native-service.png"))
   page.locator('[data-nav="projects"]').click()
   page.locator('[data-action="add-project"]').first.click()
   form.locator('[name="name"]').fill("Smoke project")
   form.locator('[name="path"]').fill(str(data))
   form.locator('[name="domain"]').fill("smoke.test")
   form.locator('[name="serviceId"]').select_option(restarted["services"][0]["id"])
   form.locator('[type="submit"]').click()
   expect(page.locator(".project-row")).to_have_count(1)
   page.locator('#project-properties [name="runtime"]').fill("Fixture 1.0")
   page.locator('#project-properties [type="submit"]').click()
   expect(page.locator("#toast")).to_have_text("Project properties saved.")
   page.locator('[data-action="theme"]').first.click()
   expect(page.locator("html")).to_have_attribute("data-theme","light")
   page.screenshot(path=str(OUT/"native-project.png"))
   # Close hides the window but preserves the engine and its owned service.
   page.locator('[data-window="close"]').click()
   time.sleep(1)
   wait_visible(process,False)
   with socket.create_connection(("127.0.0.1",port),timeout=2):pass
   restore_from_tray(process)
   restored=page.evaluate("window.__TAURI_INTERNALS__.invoke('snapshot')")
   assert restored["services"][0]["pid"]==restarted["services"][0]["pid"]
   assert restored["logs"],"Service logs disappeared after restoring the window"
   checks.append("Native Close hides the window; service stays reachable with the same PID and logs after tray restore")
   exit_application(page)
   assert process.wait(timeout=15)==0,'Explicit Exit did not shut down cleanly'
   with socket.socket() as freed:freed.bind(("127.0.0.1",port))
   browser.close()
   wait_for_debugger_exit(debug_port)
   checks.append("Explicit Exit stops Aevrik and releases the owned service port")
   debug_port=free_port()
   env["WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS"]=f"--remote-debugging-port={debug_port}"
   process=subprocess.Popen([str(exe)],cwd=data,env=env,startupinfo=startup)
   browser=connect_native(p,debug_port,process)
   page=wait_for_page(browser)
   expect(page.locator("#engine-status")).to_have_text("Engine connected",timeout=20000)
   expect(page.locator("html")).to_have_attribute("data-theme","light")
   snapshot=page.evaluate("window.__TAURI_INTERNALS__.invoke('snapshot')")
   assert len(snapshot["projects"])==1
   assert snapshot["projects"][0]["runtime"]=="Fixture 1.0"
   assert snapshot["services"][0]["state"]=="stopped"
   assert snapshot["services"][0]["pid"] is None
   checks.append("Projects, service registration and theme persist across executable restart; processes do not autostart")
   exit_application(page)
   assert process.wait(timeout=15)==0,'Explicit Exit after restart did not shut down cleanly'
   browser.close()
   assert not errors,errors
  finally:
   if process and process.poll() is None:
    process.terminate()
    process.wait(timeout=15)
 (OUT/"native-verification.json").write_text(json.dumps({"status":"passed","kind":"Real release executable + WebView2 + Rust IPC","checks":checks,"consoleErrors":errors,"isolatedTestDirectory":str(data)},indent=2))
 print(json.dumps({"status":"passed","checks":checks,"testDirectory":str(data)},indent=2))
if __name__=="__main__":
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument("--exe",help="Test a packaged executable instead of target/release/aevrik.exe")
 run(parser.parse_args().exe)
