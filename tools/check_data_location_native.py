"""Hosted-only test: real IPC, saved data folders and cold restarts, no mocked engine."""
import json
import os
import subprocess
import time
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
from pywinauto import Application
from check_native import free_port, endpoint, OUT


def check_installed_startup(executable, expected):
    """Setup has already selected data; launch must go directly to the workspace."""
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        raise RuntimeError('Run this test only on an isolated GitHub Windows runner.')
    port=free_port()
    env=os.environ.copy()
    env.pop('AEVRIK_DATA_DIR',None)
    env.pop('WEBVIEW2_USER_DATA_FOLDER',None)
    env['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS']=f'--remote-debugging-port={port}'
    process=subprocess.Popen([str(executable)],env=env,cwd=executable.parent)
    try:
        with sync_playwright() as p:
            browser=p.chromium.connect_over_cdp(endpoint(port,process))
            try:
                deadline=time.monotonic()+20
                page=None
                while page is None and time.monotonic()<deadline:
                    page=next((page for context in browser.contexts for page in context.pages),None)
                    if page is None: time.sleep(.1)
                assert page is not None,'Installed app did not open a workspace'
                expect(page.locator('#engine-status')).to_have_text('Engine connected',timeout=20000)
                info=page.evaluate("window.__TAURI_INTERNALS__.invoke('storage_info')")
                assert Path(info['current']).resolve()==expected.resolve()
                expect(page.locator('#setup-open')).to_have_count(0)
                page.locator('[data-window="close"]').click()
                process.wait(timeout=20)
            finally:
                browser.close()
    finally:
        if process.poll() is None: process.terminate();process.wait(timeout=10)


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
    processes=[]
    with sync_playwright() as p:
        def launch(overridden=None, setup=False):
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
            page=None
            while page is None and time.monotonic()<deadline:
                page=next((page for context in browser.contexts for page in context.pages),None)
                if page is None: time.sleep(.1)
            if page is None: raise TimeoutError('Aevrik did not create a WebView page')
            if setup:
                expect(page.get_by_role('heading',name='Choose your data folder')).to_be_visible(timeout=20000)
                expect(page.locator('#setup-open')).to_be_enabled()
            else:
                expect(page.locator('#engine-status')).to_have_text('Engine connected',timeout=20000)
            return process,browser,page

        def close(process,browser,page):
            page.locator('[data-window="close"]').click()
            process.wait(timeout=20)
            browser.close()

        def info(page): return page.evaluate("window.__TAURI_INTERNALS__.invoke('storage_info')")
        def same(left,right): return os.path.normcase(str(left))==os.path.normcase(str(right))
        try:
            # A fresh launch has no engine/database until the user confirms.
            process,browser,page=launch(setup=True)
            assert not config.exists() and not (profile/'aevrik.sqlite').exists()
            assert not (profile/'runtimes').exists()
            page.screenshot(path=str(OUT/'first-launch-data-folder.png'))
            page.get_by_role('button',name='Exit',exact=True).click()
            process.wait(timeout=20)
            browser.close()
            assert not config.exists() and not (profile/'aevrik.sqlite').exists()

            process,browser,page=launch(setup=True)
            field=page.get_by_label('Data folder',exact=True)
            field.fill(str(executable.parent))
            page.get_by_role('button',name='Open Aevrik',exact=True).click()
            expect(page.locator('#setup-error')).to_contain_text('separate')
            assert not config.exists() and not (profile/'aevrik.sqlite').exists()
            field.fill(str(source))
            page.get_by_role('button',name='Open Aevrik',exact=True).click()
            # Setup and main use different WebView profiles. Inspect the native
            # handoff here; reconnect CDP after a cold restart for actual IPC.
            native=Application(backend='win32').connect(process=process.pid,timeout=30)
            main=native.window(title='Aevrik')
            main.wait('exists',timeout=30)
            native.window(title='Aevrik — Choose your data folder').wait_not('exists',timeout=30)
            assert process.poll() is None
            saved=json.loads(config.read_text(encoding='utf-8'))
            assert same(saved['directory'].removeprefix('\\\\?\\'),source)
            assert (source/'aevrik.sqlite').is_file() and (source/'runtimes').is_dir()
            assert not (profile/'aevrik.sqlite').exists()
            main.close()
            process.wait(timeout=20)
            browser.close()

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
    print('Native first-launch setup, exit, validation, selection, data-folder switch, cold restarts, preserved metadata, runtime/cache roots and environment override passed.')
