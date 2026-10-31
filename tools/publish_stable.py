"""Publish a verified master build; never replace an existing stable tag/release."""
import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path

from release_policy import stable_tags, version_tuple


def gh(*args):
    return subprocess.run(['gh', *args], check=True, capture_output=True,
                          text=True, encoding='utf-8').stdout.strip()


def publish(directory, repo, sha, tag, run=gh):
    if not tag.startswith('v'):
        raise ValueError('Stable tags must start with v.')
    version = tag[1:]
    version_tuple(version)
    info = json.loads((directory/'build-info.json').read_text(encoding='utf-8'))
    if (info['version'], info['commit'], info['channel']) != (version, sha, 'versioned'):
        raise ValueError('Build metadata does not match the stable release.')
    names = {f'Aevrik-{version}-windows-x64.exe', f'Aevrik-{version}-windows-x64.zip',
             f'Aevrik-{version}-windows-x64-setup.exe'}
    if len(info['artifacts']) != 3 or {item['name'] for item in info['artifacts']} != names:
        raise ValueError('Unexpected stable artifacts.')
    checksums = []
    for item in info['artifacts']:
        path = directory/item['name']
        with path.open('rb') as stream:
            digest = hashlib.file_digest(stream, 'sha256').hexdigest()
        if digest != item['sha256'] or path.stat().st_size != item['bytes']:
            raise ValueError('Artifact checksum or size mismatch.')
        checksums.append(f"{digest}  {item['name']}")
    if (directory/'SHA256SUMS.txt').read_text().splitlines() != checksums:
        raise ValueError('Checksum manifest does not match build metadata.')

    api = f'repos/{repo}'
    head = json.loads(run('api', api+'/git/ref/heads/master'))
    refs = json.loads(run('api', api+'/git/matching-refs/tags/'+tag))
    ref = next((item for item in refs if item['ref'] == 'refs/tags/'+tag), None)
    if ref:
        target = ref['object']
        for _ in range(8):
            if target['type'] == 'commit':
                break
            if target['type'] != 'tag':
                raise ValueError('Stable tag does not target a commit.')
            target = json.loads(run('api', api+'/git/tags/'+target['sha']))['object']
        if target['type'] != 'commit' or target['sha'] != sha:
            raise ValueError('Stable tag already belongs to another commit; refusing to move it.')
    releases = json.loads(run('api', '--paginate', '--slurp', api+'/releases?per_page=100'))
    existing = next((item for page in releases for item in page if item['tag_name'] == tag), None)
    if existing and not ref:
        raise ValueError('Existing stable release has no matching tag.')
    if existing and not existing['draft']:
        return 'Already published; stable release left unchanged.'
    if head['object']['sha'] != sha:
        if not ref:
            return 'Skipped: master has a newer commit.'
        # A tag already reserved for this exact SHA may finish publication after
        # newer pushes. Never allocate a new tag from a stale build.
        comparison = json.loads(run('api', api+'/compare/'+sha+'...'+head['object']['sha']))
        if comparison['status'] != 'ahead':
            raise ValueError('Recovery commit must remain an ancestor of master.')
    completed = stable_tags(item['tag_name'] for page in releases for item in page
                            if not item['draft'] and not item.get('prerelease', False))
    if completed and max(completed.values()) > version_tuple(version):
        raise ValueError('A newer stable release is already published; refusing to publish an older version as latest.')
    if not ref:
        run('api', '--method', 'POST', api+'/git/refs', '-f', 'ref=refs/tags/'+tag, '-f', 'sha='+sha)
    if not existing:
        run('release', 'create', tag, '--repo', repo, '--verify-tag', '--draft',
            '--title', 'Aevrik '+tag, '--notes-file', str(directory/'RELEASE-NOTES.md'))
    assets = sorted(names) + ['SHA256SUMS.txt', 'build-info.json']
    run('release', 'upload', tag, '--repo', repo, '--clobber',
        *(str(directory/name) for name in assets))
    run('release', 'edit', tag, '--repo', repo, '--draft=false', '--latest')
    return 'Published '+tag


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, default=Path('release'))
    args = parser.parse_args()
    print(publish(args.directory, os.environ['GH_REPO'], os.environ['BUILD_SHA'], os.environ['RELEASE_TAG']))
