"""Validate release identity and package the executable built by the Windows CI job."""
import argparse
import hashlib
import json
import os
import re
import shutil
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def validate_version(root, tag=None):
    package = json.loads((root/'package.json').read_text(encoding='utf-8'))
    lock = json.loads((root/'package-lock.json').read_text(encoding='utf-8'))
    tauri = json.loads((root/'src-tauri/tauri.conf.json').read_text(encoding='utf-8'))
    version = package['version']
    versions = [lock['version'], lock['packages']['']['version'], tauri['version']]
    for name in ['src-tauri/Cargo.toml', 'crates/aevrik-engine/Cargo.toml']:
        versions.append(tomllib.loads((root/name).read_text(encoding='utf-8'))['package']['version'])
    if not re.fullmatch(r'\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?', version) or any(v != version for v in versions):
        raise ValueError('Release versions must match in package.json, package-lock.json, both Cargo manifests and tauri.conf.json.')
    if tauri['identifier'] != 'id.luzyver.aevrik.desktop':
        raise ValueError('Unexpected application identifier.')
    if tag is not None and tag != 'v'+version:
        raise ValueError(f'Tag {tag!r} does not match v{version}.')
    return version


def package_release(root, output, tag=None, channel='versioned'):
    if channel not in ['versioned', 'dev', 'beta']:
        raise ValueError('Unknown release channel.')
    version = validate_version(root, tag)
    executable = root/'target/release/aevrik.exe'
    if not executable.is_file(): raise ValueError('Build the release executable on the Windows runner first.')
    installers = list((root/'target/release/bundle/nsis').glob('*-setup.exe'))
    if len(installers) != 1 or installers[0].name != f'Aevrik_{version}_x64-setup.exe':
        raise ValueError('Build exactly one NSIS installer matching the current version on the Windows runner first.')
    output.mkdir(parents=True, exist_ok=True)
    name = f"Aevrik-{version if channel == 'versioned' else channel}-windows-x64"
    shutil.copy2(executable, output/(name+'.exe'))
    shutil.copy2(installers[0], output/(name+'-setup.exe'))
    shutil.copy2(root/'docs/RELEASE-NOTES.md', output/'RELEASE-NOTES.md')
    with zipfile.ZipFile(output/(name+'.zip'), 'w', zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        archive.write(output/(name+'.exe'), name+'.exe')
        archive.write(root/'docs/PORTABLE-README.txt', 'README.txt')
    artifacts = []
    catalog=json.loads((root/'crates/aevrik-engine/src/packages/catalog.json').read_text(encoding='utf-8'))
    (output/'runtime-catalog.json').write_text(json.dumps({'schemaVersion':1,'packages':catalog},separators=(',',':'))+'\n',encoding='utf-8')
    for filename in [name+'.exe', name+'.zip', name+'-setup.exe', 'runtime-catalog.json']:
        path = output/filename
        with path.open('rb') as file: digest = hashlib.file_digest(file, 'sha256').hexdigest()
        artifacts.append({'name':path.name, 'bytes':path.stat().st_size, 'sha256':digest})
    result = {'version':version, 'channel':channel, 'identifier':'id.luzyver.aevrik.desktop', 'commit':os.environ.get('GITHUB_SHA'), 'runId':os.environ.get('GITHUB_RUN_ID'), 'builtAt':int(os.environ.get('AEVRIK_BUILT_AT','0')), 'artifacts':artifacts}
    (output/'build-info.json').write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
    # Windows builds publish on Linux, where CRLF can become part of filenames.
    (output/'SHA256SUMS.txt').write_text(''.join(f"{a['sha256']}  {a['name']}\n" for a in artifacts), encoding='utf-8', newline='\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--tag')
    parser.add_argument('--channel', choices=['versioned', 'dev', 'beta'], default='versioned')
    parser.add_argument('--output', type=Path, default=ROOT/'release')
    args = parser.parse_args()
    print(validate_version(ROOT, args.tag) if args.check else json.dumps(package_release(ROOT, args.output, args.tag, args.channel), indent=2))
