import unittest
import importlib.util
from unittest.mock import patch
from catalog_sources import languages


class RuntimeSourcesTests(unittest.TestCase):
    def test_jre_uses_runtime_archive_root_and_publisher_checksum(self):
        self.assertTrue(hasattr(languages, 'java_jre'), 'Missing JRE adapter')
        data = [{'release_name': 'jdk-21.0.9+10', 'binary': {'package': {
            'link': 'https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.9%2B10/jre.zip',
            'checksum': 'a'*64}}}]
        with patch.object(languages, 'metadata', return_value=data):
            rows = languages.java_jre(majors=[21])
        self.assertEqual(rows[0]['executable'], 'jdk-21.0.9+10-jre/bin/java.exe')
        self.assertEqual(rows[0]['family'], 'java-jre')
        self.assertEqual(rows[0]['sha256'], 'a'*64)

    def test_wasmtime_selects_cli_zip_not_c_api_or_prerelease(self):
        self.assertIsNotNone(importlib.util.find_spec('catalog_sources.runtimes'), 'Missing runtime adapters')
        from catalog_sources import runtimes
        def asset(name):
            return {'name': name, 'browser_download_url': 'https://github.com/bytecodealliance/wasmtime/releases/download/v50.0.0/'+name, 'digest': 'sha256:'+'b'*64}
        data = [{'tag_name': 'v51.0.0', 'prerelease': True, 'draft': False, 'assets': []},
                {'tag_name': 'v50.0.0', 'prerelease': False, 'draft': False, 'assets': [
                    asset('wasmtime-v50.0.0-x86_64-windows-c-api.zip'),
                    asset('wasmtime-v50.0.0-aarch64-windows.zip'),
                    asset('wasmtime-v50.0.0-x86_64-windows.zip')]}]
        with patch.object(runtimes, 'metadata', return_value=data):
            rows = runtimes.wasmtime()
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]['executable'], 'wasmtime-v50.0.0-x86_64-windows/wasmtime.exe')
        self.assertEqual(rows[0]['sha256'], 'b'*64)

    def test_luau_uses_windows_archive_and_skips_drafts(self):
        self.assertIsNotNone(importlib.util.find_spec('catalog_sources.runtimes'), 'Missing runtime adapters')
        from catalog_sources import runtimes
        data = [{'tag_name': '0.700', 'prerelease': False, 'draft': False, 'assets': [
            {'name': 'luau-windows.zip', 'browser_download_url': 'https://github.com/luau-lang/luau/releases/download/0.700/luau-windows.zip', 'digest': 'sha256:'+'c'*64}]},
            {'tag_name': '0.701', 'prerelease': False, 'draft': True, 'assets': []}]
        with patch.object(runtimes, 'metadata', return_value=data):
            rows = runtimes.luau()
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]['version'], '0.700')
        self.assertEqual(rows[0]['executable'], 'luau.exe')
        self.assertEqual(rows[0]['sha256'], 'c'*64)
