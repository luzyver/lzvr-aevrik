"""Stable release numbering from Conventional Commits and reachable Git tags."""
import re

STABLE = re.compile(r'(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)')
HEADER = re.compile(r'([a-z]+)(?:\([^\r\n()]+\))?(!)?:\s+\S', re.I)


def version_tuple(version):
    match = STABLE.fullmatch(version)
    if not match:
        raise ValueError(f'Invalid stable version: {version!r}')
    return tuple(map(int, match.groups()))


def stable_tags(tags):
    return {tag: version_tuple(tag[1:]) for tag in tags
            if tag.startswith('v') and STABLE.fullmatch(tag[1:])}


def change_level(messages):
    level = 0
    for message in messages:
        subject = message.split('\n', 1)[0]
        header = HEADER.match(subject)
        if (header and header.group(2)) or re.search(r'^BREAKING[ -]CHANGE:\s+\S', message, re.M):
            return 3
        if header:
            kind = header.group(1).lower()
            level = max(level, 2 if kind == 'feat' else 1 if kind in ['fix', 'perf'] else 0)
    return level


def plan_version(source_version, merged_tags, all_tags, head_tags, messages, *, published_tags):
    merged = stable_tags(merged_tags)
    all_stable = stable_tags(all_tags)
    latest = max(merged, key=merged.get) if merged else None
    if all_stable and (not latest or max(all_stable.values()) > merged[latest]):
        raise ValueError('Stable tag history is not included in this commit; restore master history before publishing.')
    pending = sorted(set(merged) - set(published_tags) - set(head_tags), key=merged.get, reverse=True)
    if pending:
        raise ValueError(
            'Stable tag(s) unpublished: '+', '.join(pending)+'. '
            'Re-run the failed publication job in the original workflow run at the tag SHA '
            '(or re-run all jobs there if artifacts expired), then re-run this workflow. '
            'Unpublished tags/drafts cannot be used as a version baseline.')
    if latest and latest in head_tags:
        return {'version': latest[1:], 'tag': latest, 'reason': 'retry', 'previous': latest}
    if latest:
        level = change_level(messages)
        if not level:
            return {'version': latest[1:], 'tag': '', 'reason': 'no release changes', 'previous': latest}
        major, minor, patch = merged[latest]
        parts = (major+1, 0, 0) if level == 3 else (major, minor+1, 0) if level == 2 else (major, minor, patch+1)
        version = '.'.join(map(str, parts))
        reason = {1: 'patch', 2: 'minor', 3: 'major'}[level]
    else:
        version = source_version.split('-', 1)[0]
        version_tuple(version)
        reason = 'first stable release'
    return {'version': version, 'tag': 'v'+version, 'reason': reason, 'previous': latest}
