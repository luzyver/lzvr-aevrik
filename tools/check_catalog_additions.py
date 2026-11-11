"""Hosted-runner smoke: install new products and query version without starting a database."""
import json
import argparse
import os
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FAMILIES = ['mongodb', 'duckdb', 'sqlite', 'dotnet-sdk', 'dotnet-runtime', 'aspnet-runtime', 'zig', 'nim', 'java-jre', 'luau', 'wasmtime',
            'traefik', 'frankenphp', 'gleam', 'pocketbase', 'uv', 'pnpm', 'gh', 'hugo', 'typst',
            'erlang', 'perl', 'julia', 'powershell', 'nushell', 'babashka', 'gcc', 'llvm-mingw', 'dart', 'vlang', 'odin', 'haxe', 'tinygo', 'cmake', 'ninja']
# Arguments that print the version when --version does not, and the text to expect when it is not the catalog version.
ARGUMENTS = {
    'dotnet-runtime': ['--list-runtimes'], 'aspnet-runtime': ['--list-runtimes'], 'java-jre': ['-version'],
    'zig': ['version'], 'traefik': ['version'], 'frankenphp': ['version'], 'hugo': ['version'],
    'vlang': ['version'], 'odin': ['version'], 'tinygo': ['version'],
    'erlang': ['-noshell', '-eval', 'io:format("otp ~s~n",[erlang:system_info(otp_release)]),halt().'],
}
EXPECTED = {
    'erlang': lambda version: 'otp '+version.split('.')[0],
    'perl': lambda version: 'v'+'.'.join(version.split('.')[:3]),
    'gcc': lambda version: version.split('-')[0],
    'llvm-mingw': lambda version: 'clang version',
    'odin': lambda version: 'dev-'+version.replace('.', '-'),
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--families', nargs='+', default=FAMILIES)
    args = parser.parse_args()
    catalog = json.loads((ROOT/'crates/aevrik-engine/src/packages/catalog.json').read_text())
    selected = [next(p for p in catalog if p['family'] == family)
                for family in args.families]
    destination = Path(tempfile.mkdtemp(prefix='aevrik-catalog-', dir=os.environ.get('RUNNER_TEMP')))
    subprocess.run(['cargo', 'run', '--locked', '-p', 'aevrik-engine', '--example',
                    'install_smoke', '--', str(destination), *[p['id'] for p in selected]],
                   cwd=ROOT, check=True, timeout=3600)
    checks = []
    for package in selected:
        family = package['family']
        executable = destination/package['id']/package['version']/package['executable']
        arguments = ARGUMENTS.get(family, ['--version'])
        expected = EXPECTED.get(family, lambda version: version)(package['version'])
        if family == 'luau':
            script = destination/'smoke.luau'
            script.write_text('print("aevrik-luau-ok")\n')
            arguments = [str(script)]
            expected = 'aevrik-luau-ok'
        result = subprocess.run([str(executable), *arguments], cwd=destination,
                                capture_output=True, text=True, check=True, timeout=120)
        output = result.stdout + result.stderr
        assert expected in output, (package['id'], output)
        checks.append({'id':package['id'], 'executable':str(executable), 'output':output.strip()})
    report = ROOT/'previews/desktop/catalog-additions-smoke-0.1.0-beta-1.json'
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps({'status':'passed', 'directory':str(destination), 'checks':checks}, indent=2))
    print(f'New catalog product checks passed: {report}')


if __name__ == '__main__':
    main()
