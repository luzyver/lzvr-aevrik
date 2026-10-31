"""Plan and stamp a stable build in CI; this script never creates commits or tags."""
import argparse
import json
import os
import subprocess
from pathlib import Path

from ci_release import ROOT, validate_version
from release_policy import plan_version, stable_tags, version_tuple
from stamp_version import stamp_version


def git(root, *args):
    return subprocess.run(['git', *args], cwd=root, check=True, capture_output=True,
                          text=True, encoding='utf-8').stdout.strip()


def published_stable_tags(repo):
    # Read access is enough: drafts hidden from this token still count as unpublished
    # because their Git tags will be absent from the published release set.
    output = subprocess.run(['gh', 'api', '--paginate', '--slurp', f'repos/{repo}/releases?per_page=100'],
                            check=True, capture_output=True, text=True, encoding='utf-8').stdout
    return [release['tag_name'] for page in json.loads(output) for release in page
            if not release['draft'] and not release['prerelease']]


def prepare(root, tag=None, dry_run=False, read_git=git, *, published_tags=()):
    source = validate_version(root)
    all_tags = read_git(root, 'tag', '--list').splitlines()
    merged = read_git(root, 'tag', '--merged', 'HEAD').splitlines()
    head_tags = read_git(root, 'tag', '--points-at', 'HEAD').splitlines()
    previous = stable_tags([item for item in merged if item not in head_tags])
    base = max(previous, key=previous.get) if previous else None
    revision = base+'..HEAD' if base else 'HEAD'
    messages = read_git(root, 'log', '--format=%B%x00', revision).split('\x00')
    messages = [message.strip() for message in messages if message.strip()]
    if tag:
        if not tag.startswith('v'):
            raise ValueError('Stable tags must start with v.')
        version_tuple(tag[1:])
        plan = {'version': tag[1:], 'tag': tag, 'reason': 'explicit stable tag', 'previous': base}
    else:
        plan = plan_version(source, merged, all_tags, head_tags, messages, published_tags=published_tags)
    if not dry_run:
        stamp_version(root, plan['version'])
        if plan['tag']:
            subjects = '\n'.join('- '+message.split('\n', 1)[0] for message in messages)
            notes = f"# Aevrik {plan['version']}\n\n{subjects or 'Rebuild of the tagged source.'}\n"
            (root/'docs/RELEASE-NOTES.md').write_text(notes, encoding='utf-8', newline='\n')
    return plan


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tag')
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args()
    published = published_stable_tags(os.environ['GH_REPO']) if not args.tag else []
    result = prepare(ROOT, args.tag, args.dry_run, published_tags=published)
    if not args.dry_run and os.environ.get('GITHUB_OUTPUT'):
        with Path(os.environ['GITHUB_OUTPUT']).open('a', encoding='utf-8', newline='\n') as output:
            output.write(f"tag={result['tag']}\nversion={result['version']}\n")
    print(json.dumps(result, indent=2))
