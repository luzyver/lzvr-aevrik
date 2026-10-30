"""Check installed representative releases without starting any server."""
import argparse
import ctypes
import json
import os
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECKS = {
    'nginx-1.31.8': ['-v'],
    'python-3.15.2': ['--version'],
    'go-1.27.3': ['version'],
    'java-25.0.5-8': ['-version'],
    'bun-1.4.4': ['--version'],
    'deno-2.9.8': ['--version'],
    'mailpit-1.32.0': ['version', '--no-release-check'],
    'php-5.6.40': ['--version'],
    'mariadb-12.2.2': ['--version'],
}


def run(directory):
    if os.name == 'nt':
        # Missing legacy VC runtimes must report an exit code, not open a blocking OS dialog.
        ctypes.windll.kernel32.SetErrorMode(0x0001 | 0x0002 | 0x8000)
    catalog = json.loads((ROOT/'crates/aevrik-engine/src/packages/catalog.json').read_text(encoding='utf-8'))
    report = []
    for release_id, arguments in CHECKS.items():
        package = next(p for p in catalog if p['id'] == release_id)
        executable = directory/release_id/package['version']/package['executable']
        assert executable.is_file(), executable
        result = subprocess.run([str(executable), *arguments], cwd=executable.parent,
                                capture_output=True, text=True, timeout=30)
        output = (result.stdout + result.stderr).strip()
        assert result.returncode == 0, (release_id, result.returncode, output)
        expected = package['version'].split('+')[0]
        assert expected in output, (release_id, output)
        report.append({'id': release_id, 'path': str(executable), 'output': output, 'status': 'passed'})
        print(release_id, output.splitlines()[0], flush=True)
        if package['family'] == 'python':
            pip = subprocess.run([str(executable), '-m', 'pip', '--version'], cwd=executable.parent,
                                 capture_output=True, text=True, timeout=30)
            assert pip.returncode == 0, pip.stderr
            report[-1]['pip'] = pip.stdout.strip()
    target = ROOT/'previews/desktop/runtime-version-verification-0.3.0.json'
    target.write_text(json.dumps(report, indent=2), encoding='utf-8')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    run(parser.parse_args().directory.resolve())
