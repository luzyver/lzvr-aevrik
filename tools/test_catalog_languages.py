import unittest
from unittest.mock import patch
from catalog_sources import languages


class AdditionalLanguagesTests(unittest.TestCase):
    def test_malformed_publisher_digest_is_rejected(self):
        from catalog_sources.common import entry
        with self.assertRaisesRegex(ValueError,'SHA-512'):
            entry('dotnet-sdk','.NET SDK','10.0.100','Language','SDK','https://builds.dotnet.microsoft.com/sdk.zip','dotnet.exe',sha512='bad')

    def test_dotnet_uses_publisher_sha512_and_only_stable_windows_zip(self):
        self.assertTrue(hasattr(languages, 'dotnet'), 'Missing .NET catalog adapter')
        data = {'releases': [{'sdk': {'version': '10.0.101', 'files': [
            {'rid': 'win-x64', 'name': 'dotnet-sdk-win-x64.zip', 'url': 'https://builds.dotnet.microsoft.com/sdk.zip', 'hash': 'a'*128},
            {'rid': 'win-arm64', 'name': 'sdk.zip', 'url': 'https://builds.dotnet.microsoft.com/arm.zip', 'hash': 'b'*128}]},
            'runtime': {'version': '10.0.1', 'files': []}},
            {'sdk': {'version': '11.0.100-preview.1', 'files': []}}]}
        with patch.object(languages, 'metadata', return_value=data):
            rows = languages.dotnet(channels=['10.0'])
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]['executable'], 'dotnet.exe')
        self.assertEqual(rows[0]['sha512'], 'a'*128)
        self.assertEqual(rows[0]['sha256'], '')
        self.assertEqual(rows[0]['family'], 'dotnet-sdk')

    def test_dotnet_runtime_ignores_apphost_before_runtime(self):
        apphost = {'rid': 'win-x64', 'name': 'dotnet-apphost-pack-win-x64.zip',
                   'url': 'https://builds.dotnet.microsoft.com/apphost.zip', 'hash': 'b'*128}
        runtime = {'rid': 'win-x64', 'name': 'dotnet-runtime-win-x64.zip',
                   'url': 'https://builds.dotnet.microsoft.com/runtime.zip', 'hash': 'a'*128}
        data = {'releases': [{'runtime': {'version': '10.0.14', 'files': [apphost, runtime]}}]}
        with patch.object(languages, 'metadata', return_value=data):
            rows = languages.dotnet(channels=['10.0'])
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]['url'], runtime['url'])
        self.assertEqual(rows[0]['sha512'], runtime['hash'])
        self.assertEqual(rows[0]['executable'], 'dotnet.exe')

    def test_dotnet_does_not_substitute_apphost_when_runtime_is_missing(self):
        data = {'releases': [{'runtime': {'version': '10.0.14', 'files': [
            {'rid': 'win-x64', 'name': 'dotnet-apphost-pack-win-x64.zip',
             'url': 'https://builds.dotnet.microsoft.com/apphost.zip', 'hash': 'b'*128}]}}]}
        with patch.object(languages, 'metadata', return_value=data):
            self.assertEqual(languages.dotnet(channels=['10.0']), [])

    def test_zig_excludes_nightly_and_uses_archive_root(self):
        self.assertTrue(hasattr(languages, 'zig'), 'Missing Zig catalog adapter')
        data = {'master': {'version': '0.17.0-dev.1'}, '0.16.0': {'x86_64-windows': {
            'tarball': 'https://ziglang.org/download/0.16.0/zig-x86_64-windows-0.16.0.zip', 'shasum': 'b'*64}}}
        with patch.object(languages, 'metadata', return_value=data):
            rows = languages.zig()
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]['executable'], 'zig-x86_64-windows-0.16.0/zig.exe')
        self.assertEqual(rows[0]['sha256'], 'b'*64)

    def test_nim_uses_x64_archives_and_checksum_without_downloading_archive(self):
        self.assertTrue(hasattr(languages, 'nim'), 'Missing Nim catalog adapter')
        def fetch(url):
            if url.endswith('install.html'):
                return b'<a href="https://github.com/nim-lang/nightlies/releases/download/version-2-2/nim-2.2.6-windows_x64.zip">x64</a><a href="download/nim-2.2.6_x32.zip">x32</a>'
            if url.endswith('.sha256'):
                return b'cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc  nim-2.2.6_x64.zip'
            self.fail('Must not download full archive for publisher hash')
        with patch.object(languages, 'fetch', side_effect=fetch):
            rows = languages.nim()
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]['executable'], 'nim-2.2.6/bin/nim.exe')
        self.assertEqual(rows[0]['sha256'], 'c'*64)
