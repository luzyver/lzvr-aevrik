"""Stamp the disposable CI checkout; never commit generated release versions."""
import json
import re
from ci_release import validate_version
from release_policy import version_tuple


def stamp_version(root, version):
    version_tuple(version)
    old = validate_version(root)
    updates = {}
    for name in ['package.json', 'package-lock.json', 'src-tauri/tauri.conf.json']:
        data = json.loads((root/name).read_text(encoding='utf-8'))
        data['version'] = version
        if name == 'package-lock.json':
            data['packages']['']['version'] = version
        updates[name] = json.dumps(data, indent=2, ensure_ascii=False)+'\n'
    for name in ['src-tauri/Cargo.toml', 'crates/aevrik-engine/Cargo.toml']:
        content = (root/name).read_text(encoding='utf-8')
        content, count = re.subn(r'(?m)^(version\s*=\s*")[^"]+("\s*)$',
                                 lambda m: m[1]+version+m[2], content, count=1)
        if count != 1:
            raise ValueError(f'Missing package version in {name}')
        updates[name] = content
    content = (root/'Cargo.lock').read_text(encoding='utf-8')
    for package in ['aevrik', 'aevrik-engine']:
        # Match the one local package block without altering dependency versions.
        pattern = rf'(\[\[package\]\]\s+name = "{re.escape(package)}"\s+version = ")[^"]+(")'
        content, count = re.subn(pattern, lambda m: m[1]+version+m[2], content)
        if count != 1:
            raise ValueError(f'Expected one Cargo.lock entry for {package}')
    updates['Cargo.lock'] = content
    for name, label in [('desktop/index.html', f'{old} · Desktop'),
                        ('desktop/views.ts', f'Aevrik {old} · Rust engine')]:
        content = (root/name).read_text(encoding='utf-8')
        if content.count(label) != 1:
            raise ValueError(f'Missing or ambiguous version label in {name}')
        updates[name] = content.replace(label, label.replace(old, version))
    for name, content in updates.items():
        (root/name).write_text(content, encoding='utf-8', newline='\n')
    validate_version(root, 'v'+version)
