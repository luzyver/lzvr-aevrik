import unittest
from unittest.mock import patch
from catalog_sources import devtools, runtimes


def release(tag, *names, prerelease=False):
    return {'tag_name': tag, 'prerelease': prerelease, 'draft': False, 'assets': [
        {'name': name, 'browser_download_url': f'https://github.com/example/releases/download/{tag}/{name}',
         'digest': 'sha256:'+'d'*64} for name in names]}


class DevtoolSourcesTests(unittest.TestCase):
    def rows(self, adapter, data):
        with patch.object(runtimes, 'metadata', return_value=data):
            return adapter()

    def test_version_in_asset_name_drops_the_tag_prefix(self):
        rows = self.rows(devtools.gh, [release('v2.90.0', 'gh_2.90.0_windows_arm64.zip', 'gh_2.90.0_windows_amd64.zip')])
        self.assertEqual(rows[0]['url'].rsplit('/', 1)[1], 'gh_2.90.0_windows_amd64.zip')
        self.assertEqual((rows[0]['version'], rows[0]['executable'], rows[0]['category']), ('2.90.0', 'bin/gh.exe', 'Development tools'))

    def test_hugo_takes_the_extended_edition_only(self):
        rows = self.rows(devtools.hugo, [release('v0.150.0', 'hugo_0.150.0_windows-amd64.zip',
            'hugo_extended_withdeploy_0.150.0_windows-amd64.zip', 'hugo_extended_0.150.0_windows-amd64.zip')])
        self.assertEqual(rows[0]['url'].rsplit('/', 1)[1], 'hugo_extended_0.150.0_windows-amd64.zip')

    def test_typst_executable_sits_in_the_archive_folder(self):
        rows = self.rows(devtools.typst, [release('v0.20.0', prerelease=True),
            release('v0.19.0', 'typst-aarch64-pc-windows-msvc.zip', 'typst-x86_64-pc-windows-msvc.zip')])
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]['executable'], 'typst-x86_64-pc-windows-msvc/typst.exe')

    def test_keeps_the_three_newest_releases_and_their_digests(self):
        data = [release(f'0.9.{n}', 'uv-x86_64-pc-windows-msvc.zip') for n in (5, 4, 3, 2)]
        rows = self.rows(devtools.uv, data)
        self.assertEqual([r['version'] for r in rows], ['0.9.5', '0.9.4', '0.9.3'])
        self.assertTrue(all(r['sha256'] == 'd'*64 for r in rows))
