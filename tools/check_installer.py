"""Hosted-only NSIS checks: visible directory picker, custom-path install and uninstall."""
import hashlib
import json
import os
import subprocess
import tempfile
import time
import winreg
from pathlib import Path
from pywinauto import Application
from check_native import ROOT, OUT
from check_data_location_native import check as check_storage

KEY = r'Software\Microsoft\Windows\CurrentVersion\Uninstall\Aevrik'


def run():
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        raise RuntimeError('Run this test only on an isolated GitHub Windows runner.')
    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,KEY):
            raise RuntimeError('Refusing to replace an existing Aevrik installation.')
    except FileNotFoundError:
        pass
    bundles=list((ROOT/'target/release/bundle/nsis').glob('*-setup.exe'))
    assert len(bundles)==1,'Expected exactly one Setup executable'
    installer=bundles[0]
    with tempfile.TemporaryDirectory(prefix='aevrik-setup-',dir=os.environ['RUNNER_TEMP']) as temporary:
        scratch=Path(temporary)
        destination=scratch/'Custom application folder'
        process=subprocess.Popen([str(installer)],cwd=scratch)
        try:
            app=Application(backend='win32').connect(process=process.pid,timeout=30)
            window=app.window(class_name='#32770')
            window.wait('visible',timeout=30)
            deadline=time.monotonic()+30
            while time.monotonic()<deadline:
                edits=[control for control in window.descendants(class_name='Edit') if control.is_visible()]
                if edits:
                    edit=edits[0]
                    edit.set_edit_text(str(destination))
                    assert edit.window_text()==str(destination)
                    window.capture_as_image().save(str(OUT/'setup-directory-picker.png'))
                    break
                window.child_window(control_id=1,class_name='Button').click()
                time.sleep(.4)
            else:
                raise AssertionError('Setup never offered a directory picker')
            # Cancel the interactive preview; verify installation separately with
            # the same destination and NSIS's documented /D argument.
            window.close()
            time.sleep(.3)
            if process.poll() is None:
                try:
                    confirm=app.top_window().child_window(title_re='&?Yes',class_name='Button')
                    if confirm.exists(timeout=1): confirm.click()
                except Exception:
                    if process.poll() is None: raise
            process.wait(timeout=15)
        finally:
            if process.poll() is None: process.terminate();process.wait(timeout=10)
        # NSIS requires /D to be last and its path to be unquoted, including spaces.
        subprocess.run(f'"{installer}" /S /D={destination}',check=True,timeout=180)
        installed=destination/'aevrik.exe'
        assert installed.is_file()
        assert hashlib.sha256(installed.read_bytes()).digest()==hashlib.sha256((ROOT/'target/release/aevrik.exe').read_bytes()).digest()
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,KEY) as key:
            assert 'Aevrik' in winreg.QueryValueEx(key,'DisplayName')[0]
            assert str(destination).casefold() in winreg.QueryValueEx(key,'UninstallString')[0].casefold()
        check_storage(installed,scratch)
        sentinel=scratch/'Original data/aevrik.sqlite'
        before=sentinel.read_bytes()
        subprocess.run(f'"{destination / "uninstall.exe"}" /S _?={destination}',check=True,timeout=120)
        assert not installed.exists(),'Uninstall left the application executable behind'
        assert sentinel.read_bytes()==before,'Uninstall changed external user data'
        try:
            winreg.OpenKey(winreg.HKEY_CURRENT_USER,KEY).Close()
        except FileNotFoundError:
            pass
        else:
            raise AssertionError('Uninstall entry remains after removal')
    (OUT/'installer-verification.json').write_text(json.dumps({'directoryPicker':True,'customPathInstall':True,'dataPersistence':True,'uninstallPreservesData':True},indent=2))
    print('Setup directory picker, custom install, installed application, data selection and uninstall passed.')


if __name__=='__main__': run()
