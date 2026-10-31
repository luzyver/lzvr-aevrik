"""Hosted-only test: real IPC, saved data folders and cold restarts, no mocked engine."""
import json
import os
import subprocess
import time
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from pywinauto import Application
from check_native import free_port, endpoint, OUT


def check(executable, scratch):
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        raise RuntimeError('Run this test only on an isolated GitHub Windows runner.')
    profile = Path(os.environ['APPDATA'])/'id.luzyver.aevrik.desktop'
    config = profile/'data-location.json'
    if config.exists() or (profile/'aevrik.sqlite').exists():
        raise RuntimeError('Refusing to overwrite an existing Aevrik profile.')
    source = scratch/'Original data'
    target = scratch/'Selected data'
    override = scratch/'Override data'
    source.mkdir()
    profile.mkdir(parents=True, exist_ok=True)
    # A fresh launch offers the folder decision before creating its database.
    first_env=os.environ.copy()
    first_env.pop('AEVRIK_DATA_DIR',None)
    first_env.pop('WEBVIEW2_USER_DATA_FOLDER',None)
    first=subprocess.Popen([str(executable)],env=first_env,cwd=scratch)
    try:
        native=Application(backend='win32').connect(process=first.pid,timeout=30)
        prompt=native.window(title='Choose where Aevrik stores your data')
        prompt.wait('visible',timeout=30)
        prompt.capture_as_image().save(str(OUT/'first-launch-data-folder.png'))
        prompt.child_window(control_id=2,class_name='Button').click()
        first.wait(timeout=20)
        assert not config.exists() and not (profile/'aevrik.sqlite').exists()
    finally:
        if first.poll() is None: first.terminate();first.wait(timeout=10)
    config.write_text(json.dumps({'version':1,'directory':str(source)}), encoding='utf-8')
    processes=[]
    with sync_playwright() as p:
        def launch(overridden=None):
            port=free_port()
            env=os.environ.copy()
            env.pop('AEVRIK_DATA_DIR',None)
            env.pop('WEBVIEW2_USER_DATA_FOLDER',None)
            if overridden is not None: env['AEVRIK_DATA_DIR']=str(overridden)
            env['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS']=f'--remote-debugging-port={port}'
            startup=subprocess.STARTUPINFO()
            startup.dwFlags|=subprocess.STARTF_USESHOWWINDOW
            startup.wShowWindow=0
            process=subprocess.Popen([str(executable)],env=env,cwd=scratch,startupinfo=startup)
            processes.append(process)
            browser=p.chromium.connect_over_cdp(endpoint(port,process))
            deadline=time.monotonic()+20
            while not browser.contexts[0].pages and time.monotonic()<deadline: time.sleep(.1)
            page=browser.contexts[0].pages[0]
            expect(page.locator('#engine-status')).to_have_text('Engine connected',timeout=20000)
            return process,browser,page

        def close(process,browser,page):
            page.locator('[data-window="close"]').click()
            process.wait(timeout=20)
            browser.close()

        def info(page): return page.evaluate("window.__TAURI_INTERNALS__.invoke('storage_info')")
        def same(left,right): return os.path.normcase(str(left))==os.path.normcase(str(right))
        try:
            process,browser,page=launch()
            assert same(info(page)['current'],source)
            page.evaluate("path=>window.__TAURI_INTERNALS__.invoke('add_project',{input:{name:'Preserved project',path,domain:'preserved.test',runtime:'',serviceId:null}})",str(source))
            result=page.evaluate("directory=>window.__TAURI_INTERNALS__.invoke('set_data_directory',{directory})",str(target))
            assert same(result['current'],source) and same(result['next'],target)
            close(process,browser,page)
            assert (source/'aevrik.sqlite').is_file()
            process,browser,page=launch()
            assert same(info(page)['current'],target) and info(page)['next'] is None
            assert page.evaluate("window.__TAURI_INTERNALS__.invoke('snapshot')")['projects']==[]
            assert same(page.evaluate("window.__TAURI_INTERNALS__.invoke('package_install_root')"),target/'runtimes')
            assert (target/'webview').is_dir()
            page.locator('[data-nav="settings"]').click()
            expect(page.locator('#storage-current')).to_have_text(str(target))
            page.screenshot(path=str(OUT/'native-data-folder.png'))
            page.evaluate("directory=>window.__TAURI_INTERNALS__.invoke('set_data_directory',{directory})",str(source))
            close(process,browser,page)
            process,browser,page=launch()
            assert page.evaluate("window.__TAURI_INTERNALS__.invoke('snapshot')")['projects'][0]['name']=='Preserved project'
            close(process,browser,page)
            saved=config.read_bytes()
            process,browser,page=launch(override)
            assert info(page)['overridden'] and same(info(page)['current'],override)
            error=page.evaluate("async directory=>{try{await window.__TAURI_INTERNALS__.invoke('set_data_directory',{directory});return null}catch(error){return String(error)}}",str(target))
            assert error and 'AEVRIK_DATA_DIR' in error
            assert config.read_bytes()==saved
            close(process,browser,page)
        finally:
            for process in processes:
                if process.poll() is None: process.terminate();process.wait(timeout=10)
            # Only this test's preference file; never remove a user's profile tree.
            config.unlink(missing_ok=True)
    print('Native data-folder switch, cold restarts, preserved metadata, runtime/cache roots and environment override passed.')
