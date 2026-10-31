import json
import shutil
import subprocess
import tempfile
import tomllib
import unittest
from unittest.mock import patch
from pathlib import Path

from ci_release import package_release, validate_version
from prepare_stable import prepare, published_stable_tags
from publish_stable import publish
from release_policy import plan_version
from stamp_version import stamp_version

ROOT = Path(__file__).resolve().parents[1]


class VersionPolicyTests(unittest.TestCase):
    def plan(self, messages, tags=('v1.2.3',), head_tags=(), all_tags=None):
        return plan_version('0.1.0-beta-1', tags, tags if all_tags is None else all_tags, head_tags, messages,
                            published_tags=tags)

    def test_first_stable_promotes_source_base(self):
        result = self.plan(['feat!: initial application'], tags=())
        self.assertEqual(result['version'], '0.1.0')
        self.assertEqual(result['tag'], 'v0.1.0')

    def test_highest_change_wins_and_resets_lower_numbers(self):
        cases = [
            (['fix: stop crash'], '1.2.4'),
            (['perf(engine): reduce startup'], '1.2.4'),
            (['fix: stop crash', 'feat(ui): add runtime picker'], '1.3.0'),
            (['feat: new runtime', 'refactor(api)!: remove old command'], '2.0.0'),
            (['fix: update storage\n\nBREAKING CHANGE: old format removed'], '2.0.0'),
            (['chore: migrate\n\nBREAKING-CHANGE: remove compatibility'], '2.0.0'),
        ]
        for messages, version in cases:
            with self.subTest(messages=messages):
                self.assertEqual(self.plan(messages)['version'], version)

    def test_documentation_and_merge_titles_do_not_release(self):
        result = self.plan(['docs: explain installation', 'ci: cache builds', 'Merge pull request #8'])
        self.assertEqual(result['version'], '1.2.3')
        self.assertEqual(result['tag'], '')

    def test_prerelease_tags_do_not_define_stable_base(self):
        result = self.plan(['fix: repair'], tags=('v1.2.3', 'v9.0.0-beta-1', 'dev', 'beta'))
        self.assertEqual(result['tag'], 'v1.2.4')

    def test_numeric_sort_and_same_commit_retry(self):
        self.assertEqual(self.plan(['fix: repair'], tags=('v1.9.0', 'v1.10.0'))['version'], '1.10.1')
        self.assertEqual(self.plan([], head_tags=('v1.2.3',))['tag'], 'v1.2.3')

    def test_unreachable_newer_stable_tag_is_not_overwritten(self):
        with self.assertRaisesRegex(ValueError, 'history'):
            self.plan(['feat: new feature'], all_tags=('v1.2.3', 'v1.3.0'))

    def test_breaking_changes_increment_major_before_one(self):
        self.assertEqual(self.plan(['feat!: incompatible'], tags=('v0.4.2',))['version'], '1.0.0')

    def test_unpublished_tag_blocks_later_commits_until_original_run_recovers(self):
        for published in [(), ('v1.2.3',)]:
            with self.subTest(published=published), self.assertRaisesRegex(ValueError, 'unpublished.*v1.2.4'):
                plan_version('0.1.0-beta-1', ['v1.2.3', 'v1.2.4'],
                             ['v1.2.3', 'v1.2.4'], [], ['docs: update guide'],
                             published_tags=published)

    def test_pending_tag_at_original_sha_can_be_rebuilt(self):
        result = plan_version('0.1.0-beta-1', ['v1.2.3', 'v1.2.4'],
                              ['v1.2.3', 'v1.2.4'], ['v1.2.4'], ['fix: crash'],
                              published_tags=['v1.2.3'])
        self.assertEqual(result['tag'], 'v1.2.4')


class VersionStampTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for name in ['package.json', 'package-lock.json', 'Cargo.lock', 'src-tauri/tauri.conf.json',
                     'src-tauri/Cargo.toml', 'crates/aevrik-engine/Cargo.toml',
                     'desktop/index.html', 'desktop/views.ts']:
            destination = self.root/name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT/name, destination)

    def test_stamp_matches_all_manifests_ui_and_only_workspace_lock_entries(self):
        before = tomllib.loads((self.root/'Cargo.lock').read_text())['package']
        stamp_version(self.root, '1.3.0')
        self.assertEqual(validate_version(self.root, 'v1.3.0'), '1.3.0')
        after = tomllib.loads((self.root/'Cargo.lock').read_text())['package']
        for old, new in zip(before, after, strict=True):
            if old['name'] in ['aevrik', 'aevrik-engine']:
                self.assertEqual(new['version'], '1.3.0')
            else:
                self.assertEqual(new, old)
        self.assertIn('1.3.0 · Desktop', (self.root/'desktop/index.html').read_text(encoding='utf-8'))
        self.assertIn('Aevrik 1.3.0 · Rust engine', (self.root/'desktop/views.ts').read_text(encoding='utf-8'))
        stamp_version(self.root, '1.3.0')
        self.assertEqual(validate_version(self.root), '1.3.0')

    def test_invalid_version_does_not_modify_files(self):
        before = (self.root/'package.json').read_bytes()
        with self.assertRaises(ValueError):
            stamp_version(self.root, '1.03.0')
        self.assertEqual((self.root/'package.json').read_bytes(), before)

    def test_prepare_reads_commit_bodies_and_stamps_before_packaging(self):
        (self.root/'docs').mkdir()
        responses = {
            ('tag', '--list'): 'dev\nbeta\nv1.2.3\nv9.0.0-beta-1',
            ('tag', '--merged', 'HEAD'): 'v1.2.3',
            ('tag', '--points-at', 'HEAD'): '',
            ('log', '--format=%B%x00', 'v1.2.3..HEAD'): 'feat: new picker\x00fix: crash\x00',
        }
        result = prepare(self.root, read_git=lambda root, *args: responses[args], published_tags=['v1.2.3'])
        self.assertEqual(result['tag'], 'v1.3.0')
        self.assertEqual(validate_version(self.root), '1.3.0')
        self.assertIn('- feat: new picker', (self.root/'docs/RELEASE-NOTES.md').read_text())

    def test_dry_run_does_not_stamp_checkout(self):
        before = (self.root/'package.json').read_bytes()
        source_base = json.loads(before)['version'].split('-', 1)[0]
        result = prepare(self.root, dry_run=True, read_git=lambda root, *args: '')
        self.assertEqual(result['tag'], 'v'+source_base)
        self.assertEqual((self.root/'package.json').read_bytes(), before)

    def test_pending_release_blocks_preparation_before_files_change(self):
        before = (self.root/'package.json').read_bytes()
        responses = {
            ('tag', '--list'): 'v1.2.3\nv1.3.0',
            ('tag', '--merged', 'HEAD'): 'v1.2.3\nv1.3.0',
            ('tag', '--points-at', 'HEAD'): '',
            ('log', '--format=%B%x00', 'v1.3.0..HEAD'): 'docs: update guide\x00',
        }
        with self.assertRaisesRegex(ValueError, 'unpublished.*v1.3.0'):
            prepare(self.root, read_git=lambda root, *args: responses[args], published_tags=['v1.2.3'])
        self.assertEqual((self.root/'package.json').read_bytes(), before)


class PublishedReleaseTests(unittest.TestCase):
    def test_all_pages_exclude_drafts_and_prereleases(self):
        pages = [[{'tag_name': 'v1.3.0', 'draft': True, 'prerelease': False},
                  {'tag_name': 'beta', 'draft': False, 'prerelease': True}],
                 [{'tag_name': 'v1.2.3', 'draft': False, 'prerelease': False}]]
        with patch('prepare_stable.subprocess.run') as run:
            run.return_value.stdout = json.dumps(pages)
            self.assertEqual(published_stable_tags('fixture/repo'), ['v1.2.3'])
            self.assertEqual(run.call_args.args[0],
                             ['gh', 'api', '--paginate', '--slurp', 'repos/fixture/repo/releases?per_page=100'])

    def test_api_failure_is_not_treated_as_empty_release_history(self):
        with patch('prepare_stable.subprocess.run', side_effect=subprocess.CalledProcessError(1, 'gh')):
            with self.assertRaises(subprocess.CalledProcessError):
                published_stable_tags('fixture/repo')


class FakeGitHub:
    """Controlled remote state, with publication failures and immutable tags."""
    def __init__(self, tag_sha=None, draft=None, head='built'):
        self.tag_sha, self.draft, self.head = tag_sha, draft, head
        self.writes = []
        self.fail_upload = False
        self.compare_status = 'ahead'
        self.other_releases = []

    def __call__(self, *args):
        if args[:2] == ('api', 'repos/fixture/repo/git/ref/heads/master'):
            return json.dumps({'object': {'sha': self.head}})
        if args[:2] == ('api', 'repos/fixture/repo/git/matching-refs/tags/v1.2.4'):
            return json.dumps([] if self.tag_sha is None else
                              [{'ref': 'refs/tags/v1.2.4', 'object': {'sha': self.tag_sha, 'type': 'commit'}}])
        if args == ('api', '--paginate', '--slurp', 'repos/fixture/repo/releases?per_page=100'):
            releases = [] if self.draft is None else [{'tag_name': 'v1.2.4', 'draft': self.draft}]
            return json.dumps([releases, self.other_releases])
        if args[:2] == ('api', 'repos/fixture/repo/compare/built...newer'):
            return json.dumps({'status': self.compare_status})
        self.writes.append(args)
        if args[:3] == ('api', '--method', 'POST'):
            assert self.tag_sha is None
            assert args[3:] == ('repos/fixture/repo/git/refs', '-f', 'ref=refs/tags/v1.2.4', '-f', 'sha=built')
            self.tag_sha = 'built'
        elif args[:2] == ('release', 'create'):
            assert '--draft' in args and self.draft is None
            self.draft = True
        elif args[:2] == ('release', 'upload'):
            assert self.draft is True
            if self.fail_upload:
                raise RuntimeError('Upload interrupted')
        elif args[:2] == ('release', 'edit'):
            assert self.draft is True and '--draft=false' in args
            self.draft = False
        else:
            raise AssertionError(f'Unexpected GitHub command: {args}')
        return '{}'


class StablePublishTests(unittest.TestCase):
    def setUp(self):
        fixture = VersionStampTests()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        self.root = fixture.root
        stamp_version(self.root, '1.2.4')
        (self.root/'target/release').mkdir(parents=True)
        (self.root/'target/release/aevrik.exe').write_bytes(b'MZ verified fixture')
        bundle = self.root/'target/release/bundle/nsis'
        bundle.mkdir(parents=True)
        (bundle/'Aevrik_1.2.4_x64-setup.exe').write_bytes(b'MZ verified setup fixture')
        (self.root/'docs').mkdir()
        (self.root/'docs/RELEASE-NOTES.md').write_text('Release fixture')
        (self.root/'docs/PORTABLE-README.txt').write_text('Portable fixture')
        source=self.root/'crates/aevrik-engine/src/packages'
        source.mkdir(parents=True)
        (source/'catalog.json').write_text('[]')
        self.out = self.root/'release'
        with patch.dict('os.environ', {'GITHUB_SHA': 'built'}):
            package_release(self.root, self.out)

    def test_new_release_publishes_only_after_assets_upload(self):
        remote = FakeGitHub()
        self.assertEqual(publish(self.out, 'fixture/repo', 'built', 'v1.2.4', remote), 'Published v1.2.4')
        self.assertEqual(remote.tag_sha, 'built')
        self.assertFalse(remote.draft)
        self.assertEqual([args[:2] for args in remote.writes],
                         [('api', '--method'), ('release', 'create'), ('release', 'upload'), ('release', 'edit')])

    def test_failed_upload_can_resume_without_moving_tag(self):
        remote = FakeGitHub()
        remote.fail_upload = True
        with self.assertRaisesRegex(RuntimeError, 'Upload'):
            publish(self.out, 'fixture/repo', 'built', 'v1.2.4', remote)
        self.assertTrue(remote.draft)
        remote.fail_upload = False
        remote.writes.clear()
        publish(self.out, 'fixture/repo', 'built', 'v1.2.4', remote)
        self.assertEqual([args[:2] for args in remote.writes], [('release', 'upload'), ('release', 'edit')])

    def test_upload_failure_then_docs_push_then_original_sha_recovery(self):
        remote = FakeGitHub()
        remote.fail_upload = True
        with self.assertRaisesRegex(RuntimeError, 'Upload'):
            publish(self.out, 'fixture/repo', 'built', 'v1.2.4', remote)
        remote.head = 'newer'
        tags = ['v1.2.3', 'v1.2.4']
        # A docs-only push must expose the unfinished release, not silently skip it.
        with self.assertRaisesRegex(ValueError, 'unpublished'):
            plan_version('0.1.0-beta-1', tags, tags, [], ['docs: update guide'],
                         published_tags=['v1.2.3'])
        remote.fail_upload = False
        remote.writes.clear()
        self.assertEqual(publish(self.out, 'fixture/repo', 'built', 'v1.2.4', remote), 'Published v1.2.4')
        self.assertEqual(remote.tag_sha, 'built')
        self.assertFalse(remote.draft)
        self.assertEqual([args[:2] for args in remote.writes], [('release', 'upload'), ('release', 'edit')])
        result = plan_version('0.1.0-beta-1', tags, tags, [], ['docs: update guide'], published_tags=tags)
        self.assertEqual(result['tag'], '')

    def test_recovery_after_tag_creation_before_draft_creation(self):
        remote = FakeGitHub(tag_sha='built', head='newer')
        self.assertEqual(publish(self.out, 'fixture/repo', 'built', 'v1.2.4', remote), 'Published v1.2.4')
        self.assertEqual([args[:2] for args in remote.writes],
                         [('release', 'create'), ('release', 'upload'), ('release', 'edit')])

    def test_recovery_rejects_commit_no_longer_in_master(self):
        remote = FakeGitHub(tag_sha='built', draft=True, head='newer')
        remote.compare_status = 'diverged'
        with self.assertRaisesRegex(ValueError, 'ancestor'):
            publish(self.out, 'fixture/repo', 'built', 'v1.2.4', remote)
        self.assertEqual(remote.writes, [])

    def test_recovery_does_not_supersede_newer_published_release(self):
        remote = FakeGitHub(tag_sha='built', draft=True, head='newer')
        remote.other_releases = [{'tag_name': 'v1.3.0', 'draft': False, 'prerelease': False}]
        with self.assertRaisesRegex(ValueError, 'newer stable release'):
            publish(self.out, 'fixture/repo', 'built', 'v1.2.4', remote)
        self.assertEqual(remote.writes, [])

    def test_published_release_and_stale_build_never_write(self):
        for remote in [FakeGitHub(tag_sha='built', draft=False), FakeGitHub(head='newer')]:
            publish(self.out, 'fixture/repo', 'built', 'v1.2.4', remote)
            self.assertEqual(remote.writes, [])

    def test_conflicting_tag_is_never_moved(self):
        remote = FakeGitHub(tag_sha='different')
        with self.assertRaisesRegex(ValueError, 'another commit'):
            publish(self.out, 'fixture/repo', 'built', 'v1.2.4', remote)
        self.assertEqual(remote.writes, [])

    def test_modified_artifacts_fail_before_remote_changes(self):
        (self.out/'Aevrik-1.2.4-windows-x64.exe').write_bytes(b'tampered')
        remote = FakeGitHub()
        with self.assertRaisesRegex(ValueError, 'checksum'):
            publish(self.out, 'fixture/repo', 'built', 'v1.2.4', remote)
        self.assertEqual(remote.writes, [])

    def test_wrong_build_identity_never_contacts_github(self):
        def no_remote(*args):
            raise AssertionError('Metadata must be checked before contacting GitHub')
        with self.assertRaisesRegex(ValueError, 'metadata'):
            publish(self.out, 'fixture/repo', 'different', 'v1.2.4', no_remote)


if __name__ == '__main__':
    unittest.main()
