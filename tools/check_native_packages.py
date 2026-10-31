"""Opt-in HTTPS installation and viewport regression against the actual packaged EXE."""
import argparse, ctypes, json, os, socket, subprocess, tempfile, time, urllib.request
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from check_native import ROOT, OUT, free_port, endpoint
from native_browser import wait_for_page

def resize(process, width, height):
 handles=[]
 callback=ctypes.WINFUNCTYPE(ctypes.c_bool,ctypes.c_void_p,ctypes.c_void_p)
 def visit(hwnd,_):
  pid=ctypes.c_ulong()
  ctypes.windll.user32.GetWindowThreadProcessId(ctypes.c_void_p(hwnd),ctypes.byref(pid))
  if pid.value==process.pid:handles.append(hwnd)
  return True
 ctypes.windll.user32.EnumWindows(callback(visit),0)
 assert handles,'Native window not found'
 for hwnd in handles:
  title=ctypes.create_unicode_buffer(256)
  ctypes.windll.user32.GetWindowTextW(ctypes.c_void_p(hwnd),title,256)
  if title.value=='Aevrik':
   assert ctypes.windll.user32.SetWindowPos(ctypes.c_void_p(hwnd),None,0,0,width,height,0x16)
   return
 raise AssertionError('Aevrik window not found')

def run(exe):
 data=Path(tempfile.mkdtemp(prefix='aevrik-packages-'))
 runtimes=Path(tempfile.mkdtemp(prefix='aevrik-runtime-',dir=os.environ.get('RUNNER_TEMP',tempfile.gettempdir())))
 sentinel=runtimes/'unrelated.txt';sentinel.write_text('Keep this file')
 env=os.environ.copy()
 debug=free_port()
 env.update(AEVRIK_DATA_DIR=str(data),WEBVIEW2_USER_DATA_FOLDER=str(data/'webview'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=f'--remote-debugging-port={debug}')
 startup=subprocess.STARTUPINFO();startup.dwFlags|=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
 process=None;errors=[];checks=[];metrics=[]
 with sync_playwright() as p:
  try:
   process=subprocess.Popen([str(exe)],cwd=data,env=env,startupinfo=startup)
   browser=p.chromium.connect_over_cdp(endpoint(debug,process))
   page=wait_for_page(browser)
   page.on('pageerror',lambda e:errors.append(str(e)))
   expect(page.locator('#engine-status')).to_have_text('Engine connected',timeout=20000)
   page.locator('[data-action="browse-packages"]').click()
   manifest=json.loads((ROOT/'crates/aevrik-engine/src/packages/catalog.json').read_text())
   expect(page.locator('.package-row')).to_have_count(len({p.get('family',p['name']) for p in manifest}))
   page.screenshot(path=str(OUT/'native-catalog-available.png'))
   page.locator('[data-install="caddy"]').click()
   page.locator('#package-install [name="destination"]').fill(str(runtimes))
   page.locator('#package-install [type="submit"]').click()
   # Navigation and process snapshots remain responsive while Rust downloads.
   page.locator('[data-nav="services"]').click()
   expect(page.get_by_text('Your local stack starts here',exact=True)).to_be_visible()
   page.locator('[data-nav="packages"]').click()
   expect(page.locator('[data-configure="caddy"]')).to_be_visible(timeout=180000)
   packages=page.evaluate("window.__TAURI_INTERNALS__.invoke('list_packages')")
   installed=next(x for x in packages if x['id']=='caddy')
   assert Path(installed['path']).is_file()
   release=next(p for p in manifest if p['id']=='caddy')
   expected_path=runtimes/release['id']/release['version']/release['executable']
   # Rust canonicalizes Windows paths with a \\?\ prefix. Compare file identity,
   # not lexical ancestry, and require the exact selected release location.
   assert Path(installed['path']).samefile(expected_path), {'actual':installed['path'],'expected':str(expected_path)}
   checks.append('Real Caddy download, SHA-256 verification, extraction and Installed state; navigation responsive during install')
   older=next(p for p in manifest if p.get('family')=='caddy' and p['id']!='caddy')
   page.locator('[data-package-version="caddy"]').select_option(older['id'])
   page.locator(f'[data-install="{older["id"]}"]').click()
   expect(page.locator('#package-install [name="destination"]')).to_have_value(str(runtimes))
   page.locator('#package-install [type="submit"]').click()
   expect(page.locator(f'[data-configure="{older["id"]}"]')).to_be_visible(timeout=180000)
   packages=page.evaluate("window.__TAURI_INTERNALS__.invoke('list_packages')")
   second=next(p for p in packages if p['id']==older['id'])
   assert second['path']!=installed['path'] and Path(second['path']).is_file() and Path(installed['path']).is_file()
   expect(page.locator('[data-family="caddy"]')).to_contain_text('2 installed')
   checks.append('Two Caddy versions install side by side with distinct executable paths and independent Installed states')
   installed=second
   page.locator(f'[data-configure="{older["id"]}"]').click()
   form=page.locator('#registration')
   expect(form.locator('[name="executable"]')).to_have_value(installed['path'])
   site=data/'site';site.mkdir();(site/'index.html').write_text('Aevrik installer smoke')
   port=free_port()
   form.locator('[name="cwd"]').fill(str(site))
   form.locator('[name="args"]').fill(f'file-server\n--listen\n127.0.0.1:{port}\n--root\n{site}')
   form.locator('[name="port"]').fill(str(port))
   form.locator('[type="submit"]').click()
   expect(page.locator('#dialog')).not_to_be_visible()
   page.locator('[data-nav="services"]').click()
   page.locator('[data-action="start"]').click()
   expect(page.locator('#service-count')).to_have_text('1 services running')
   deadline=time.monotonic()+15
   while True:
    try:
     with urllib.request.urlopen(f'http://127.0.0.1:{port}',timeout=2) as response:assert response.read()==b'Aevrik installer smoke'
     break
    except OSError:
     if time.monotonic()>deadline:raise
     time.sleep(.2)
   checks.append('Installed executable configured via form and started; real HTTP response served from an isolated fixture folder')
   for width,height in [(800,600),(1280,820),(1440,1000)]:
    resize(process,width,height)
    page.wait_for_timeout(200)
    for route in ['overview','services','projects','logs','packages','domains','settings']:
     page.locator(f'[data-nav="{route}"]').click()
     size=page.evaluate('({width:innerWidth,height:innerHeight})')
     page.mouse.move(size['width']-5,size['height']-50);page.mouse.wheel(0,5000);page.wait_for_timeout(100)
     value=page.evaluate("""()=>({root:document.scrollingElement.scrollTop,main:document.querySelector('#main').scrollTop,overflow:document.querySelector('#main').scrollHeight-document.querySelector('#main').clientHeight,title:document.querySelector('.titlebar').getBoundingClientRect().top,bottom:document.querySelector('.statusbar').getBoundingClientRect().bottom,height:innerHeight})""")
     metrics.append({'requestedSize':[width,height],'actualSize':size,'route':route,**value})
     assert value['root']==0 and value['main']==0 and value['overflow']<=1 and value['title']==0 and abs(value['bottom']-value['height'])<=1,metrics[-1]
   checks.append('Real native window resized at three sizes: fixed titlebar, statusbar and main across all seven routes')
   resize(process,1280,820)
   page.locator('[data-nav="packages"]').click()
   page.screenshot(path=str(OUT/'native-catalog-installed.png'))
   page.locator('[data-action="theme"]').first.click()
   expect(page.locator('html')).to_have_attribute('data-theme','light')
   resize(process,800,600)
   page.screenshot(path=str(OUT/'native-catalog-800.png'))
   # Confirm the list scrolls independently all the way to the final package.
   page.locator('.package-row').last.locator('[data-install]').scroll_into_view_if_needed()
   expect(page.locator('.package-row').last.locator('[data-install]')).to_be_in_viewport()
   assert page.locator('#main').evaluate('(e)=>e.scrollTop')==0
   page.locator('[data-window="close"]').click();process.wait(timeout=15);browser.close()
   with socket.socket() as sock:sock.bind(('127.0.0.1',port))
   process=subprocess.Popen([str(exe)],cwd=data,env=env,startupinfo=startup)
   browser=p.chromium.connect_over_cdp(endpoint(debug,process));page=wait_for_page(browser)
   expect(page.locator('#engine-status')).to_have_text('Engine connected',timeout=20000)
   page.locator('[data-nav="packages"]').click()
   expect(page.locator('[data-configure="caddy"]')).to_be_visible()
   expect(page.locator('[data-family="caddy"]')).to_contain_text('2 installed')
   page.locator('[data-package-version="caddy"]').select_option(older['id'])
   expect(page.locator(f'[data-configure="{older["id"]}"]')).to_be_visible()
   expect(page.locator('#service-count')).to_have_text('0 services running')
   checks.append('Installed package persists across restart; managed server closes and does not autostart')
   page.locator(f'[data-uninstall="{older["id"]}"]').click()
   page.locator('#package-uninstall [type="submit"]').click()
   expect(page.locator('#package-uninstall #form-error')).not_to_be_empty()
   assert Path(second['path']).is_file()
   page.locator('[data-action="close-dialog"]').first.click()
   page.locator('[data-nav="services"]').click()
   page.locator('[data-action="remove-service"]').click()
   page.locator('#service-remove [type="submit"]').click()
   expect(page.locator('#dialog')).not_to_be_visible()
   page.locator('[data-nav="packages"]').click()
   page.locator(f'[data-uninstall="{older["id"]}"]').click()
   page.locator('#package-uninstall [type="submit"]').click()
   expect(page.locator(f'[data-install="{older["id"]}"]')).to_be_visible()
   assert not Path(second['path']).exists()
   assert sentinel.read_text()=='Keep this file' and (site/'index.html').is_file()
   page.locator('[data-package-version="caddy"]').select_option('caddy')
   expect(page.locator('[data-configure="caddy"]')).to_be_visible()
   checks.append('Chosen runtime folder remembered; in-use uninstall blocked; stopped registration removed before uninstall; other version and external files retained')
   page.locator('[data-window="close"]').click();process.wait(timeout=15);browser.close()
   assert not errors,errors
  finally:
   if process and process.poll() is None:process.terminate();process.wait(timeout=15)
 report={'status':'passed','executable':str(exe),'testDirectory':str(data),'checks':checks,'viewportMetrics':metrics,'consoleErrors':errors}
 (OUT/'native-package-verification-0.1.0-beta-1.json').write_text(json.dumps(report,indent=2))
 print(json.dumps({'status':'passed','checks':checks,'testDirectory':str(data)},indent=2))

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--exe',default=str(ROOT/'target/release/aevrik.exe'))
 run(Path(parser.parse_args().exe).resolve())
