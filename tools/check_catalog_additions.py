"""Hosted-runner smoke: install new products and query version without starting a database."""
import json
import os
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    catalog = json.loads((ROOT/'crates/aevrik-engine/src/packages/catalog.json').read_text())
    selected = [next(p for p in catalog if p['family'] == family)
                for family in ['mongodb', 'duckdb', 'sqlite', 'dotnet-sdk', 'dotnet-runtime', 'aspnet-runtime', 'zig', 'nim']]
    destination = Path(tempfile.mkdtemp(prefix='aevrik-catalog-', dir=os.environ.get('RUNNER_TEMP')))
    subprocess.run(['cargo', 'run', '--locked', '-p', 'aevrik-engine', '--example',
                    'install_smoke', '--', str(destination), *[p['id'] for p in selected]],
                   cwd=ROOT, check=True, timeout=2400)
    checks = []
    for package in selected:
        executable = destination/package['id']/package['version']/package['executable']
        argument = '--list-runtimes' if package['family'] in ['dotnet-runtime','aspnet-runtime'] else 'version' if package['family']=='zig' else '--version'
        result = subprocess.run([str(executable), argument], cwd=destination,
                                capture_output=True, text=True, check=True, timeout=30)
        output = result.stdout + result.stderr
        assert package['version'] in output, (package['id'], output)
        checks.append({'id':package['id'], 'executable':str(executable), 'output':output.strip()})
    report = ROOT/'previews/desktop/catalog-additions-smoke-0.1.0-beta-1.json'
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps({'status':'passed', 'directory':str(destination), 'checks':checks}, indent=2))
    print(f'New catalog product checks passed: {report}')


if __name__ == '__main__':
    main()
