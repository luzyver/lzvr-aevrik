"""Own a Vite process for browser checks, or reuse an already-running project server."""
import os, subprocess, sys, time, urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def ready():
 try:
  with urllib.request.urlopen('http://127.0.0.1:1420',timeout=1) as response:return b'Aevrik' in response.read()
 except OSError:return False
process=None
try:
 if not ready():
  process=subprocess.Popen(['node',str(ROOT/'node_modules/vite/bin/vite.js'),'--host','127.0.0.1'],cwd=ROOT,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,creationflags=subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0)
  for _ in range(100):
   if ready():break
   if process.poll() is not None:raise RuntimeError('Vite failed to start')
   time.sleep(.1)
  else:raise RuntimeError('Vite did not become ready')
 for name in sys.argv[1:] or ['check_desktop_ui.py','check_scroll.py','check_packages_ui.py','check_multiversion_ui.py','check_storage_ui.py']:
  subprocess.run([sys.executable,str(ROOT/'tools'/name)],cwd=ROOT,check=True)
finally:
 if process:process.terminate();process.wait(timeout=10)
