"""Verify persisted workspace recovery after terminating a test-owned process."""
import os
from pathlib import Path
import subprocess
import tempfile

from playwright.sync_api import expect, sync_playwright

from check_native import ROOT, free_port
from native_browser import connect_native, wait_for_debugger_exit, wait_for_page
from native_lifecycle import exit_application


def run():
    with tempfile.TemporaryDirectory(prefix='aevrik-restart-', ignore_cleanup_errors=True) as scratch:
        env = os.environ.copy()
        env['AEVRIK_DATA_DIR'] = scratch
        env['WEBVIEW2_USER_DATA_FOLDER'] = str(Path(scratch) / 'webview')
        startup = subprocess.STARTUPINFO()
        startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = 0
        project = {'name': 'Saved before forced restart', 'path': scratch,
                   'domain': 'restart.test', 'runtime': '', 'serviceId': None}
        original = None
        with sync_playwright() as p:
            for restarted in range(5):
                port = free_port()
                env['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS'] = f'--remote-debugging-port={port}'
                process = subprocess.Popen([str(ROOT / 'target/release/aevrik.exe')],
                                           env=env, cwd=scratch, startupinfo=startup)
                try:
                    browser = connect_native(p, port, process)
                    try:
                        page = wait_for_page(browser)
                        expect(page.locator('#engine-status')).to_have_text('Engine connected', timeout=20000)
                        info = page.evaluate("window.__TAURI_INTERNALS__.invoke('storage_info')")
                        assert Path(info['current']).resolve() == Path(scratch).resolve()
                        if not restarted:
                            page.evaluate("input=>window.__TAURI_INTERNALS__.invoke('add_project',{input})", project)
                            original = page.evaluate("window.__TAURI_INTERNALS__.invoke('snapshot')")
                            assert len(original['projects']) == 1
                            assert original['projects'][0]['name'] == project['name']
                            assert original['services'] == [], 'Forced restart must own no services'
                            assert process.poll() is None
                            process.terminate()
                            process.wait(timeout=20)
                        else:
                            recovered = page.evaluate("window.__TAURI_INTERNALS__.invoke('snapshot')")
                            assert recovered['projects'] == original['projects']
                            assert recovered['services'] == []
                            exit_application(page)
                            assert process.wait(timeout=20) == 0, 'Explicit Exit failed after recovery'
                    finally:
                        browser.close()
                    wait_for_debugger_exit(port)
                finally:
                    if process.poll() is None:
                        process.terminate()
                        process.wait(timeout=10)
    print('Five native launches: forced restart recovery, saved project, data directory, released browser locks and normal Exit passed.')


if __name__ == '__main__':
    run()
