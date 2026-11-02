"""Compile and drive harmless NSIS installers, including a slow skipped page."""
import os
import subprocess
import tempfile
from pathlib import Path

from pywinauto import Application
from installer_navigation import select_application_directory


def run():
    compiler = Path(os.environ['LOCALAPPDATA'])/'tauri/NSIS/makensis.exe'
    fixture = Path(__file__).parent/'fixtures/installer-navigation.nsi'
    assert compiler.is_file(), 'Run after Tauri has downloaded the NSIS bundler'
    for delay in (0, 1500):
        with tempfile.TemporaryDirectory(prefix='aevrik-nsis-navigation-') as temporary:
            scratch = Path(temporary)
            executable = scratch/'navigation.exe'
            subprocess.run([str(compiler), '/V2', f'/DOUTPUT={executable}',
                            f'/DTRANSITION_DELAY={delay}', str(fixture.resolve())], check=True)
            process = subprocess.Popen([str(executable)], cwd=scratch,
                                       creationflags=subprocess.CREATE_NO_WINDOW)
            try:
                app = Application(backend='win32').connect(process=process.pid, timeout=10)
                window = app.window(class_name='#32770')
                field = window.child_window(class_name='Edit')
                field.wait('ready', timeout=10)
                field.set_edit_text(str(scratch/'Chosen data'))
                destination = scratch/'Chosen application'
                select_application_directory(window, destination, timeout=10)
                assert window.child_window(control_id=1019, class_name='Edit').window_text() == str(destination)
                assert not (scratch/'unexpected-install.txt').exists(), 'Navigation started installation'
                window.close()
                process.wait(timeout=10)
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=10)
            print(f'Real NSIS navigation passed (skipped-page delay: {delay} ms).', flush=True)


if __name__ == '__main__':
    run()
