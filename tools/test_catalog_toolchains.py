import unittest
from unittest.mock import patch
from catalog_sources import runtimes, toolchains


def release(tag, *names, prerelease=False):
    return {'tag_name': tag, 'prerelease': prerelease, 'draft': False, 'assets': [
        {'name': name, 'browser_download_url': f'https://github.com/example/releases/download/{tag}/{name}',
         'digest': 'sha256:'+'e'*64} for name in names]}


class ToolchainSourcesTests(unittest.TestCase):
    def rows(self, adapter, data):
        with patch.object(runtimes, 'metadata', return_value=data):
            return adapter()

    def test_erlang_reads_the_version_from_otp_tags(self):
        rows = self.rows(toolchains.erlang, [release('OTP-29.0-rc1', 'otp_win64_29.0-rc1.zip'),
            release('OTP-28.1.2', 'otp_win32_28.1.2.zip', 'otp_win64_28.1.2.zip')])
        self.assertEqual([(r['version'], r['executable']) for r in rows], [('28.1.2', 'bin/erl.exe')])
        self.assertEqual(rows[0]['url'].rsplit('/', 1)[1], 'otp_win64_28.1.2.zip')

    def test_odin_monthly_tag_becomes_a_dotted_version(self):
        rows = self.rows(toolchains.odin, [release('dev-2026-11', 'odin-windows-amd64-dev-2026-11.zip'),
            release('dev-2026-10a', 'odin-windows-amd64-dev-2026-10a.zip')])
        self.assertEqual([r['version'] for r in rows], ['2026.11'])

    def test_clang_executable_sits_in_the_dated_archive_folder(self):
        rows = self.rows(toolchains.llvm_mingw, [release('20261104', 'llvm-mingw-20261104-msvcrt-x86_64.zip',
            'llvm-mingw-20261104-ucrt-x86_64.zip')])
        self.assertEqual(rows[0]['executable'], 'llvm-mingw-20261104-ucrt-x86_64/bin/clang.exe')

    def test_gcc_keeps_the_newest_revision_of_each_version_without_llvm(self):
        name = 'winlibs-x86_64-posix-seh-gcc-{}-mingw-w64ucrt-14.0.0-r{}.zip'.format
        data = [release('16.2.0posix-r3', name('16.2.0', 3), 'winlibs-x86_64-posix-seh-gcc-16.2.0-llvm-22.1.0-mingw-w64ucrt-14.0.0-r3.zip',
                        'winlibs-x86_64-posix-seh-gcc-16.2.0-mingw-w64msvcrt-14.0.0-r3.zip'),
                release('16.2.0posix-r1', name('16.2.0', 1)), release('15.3.0posix-r2', name('15.3.0', 2))]
        with patch.object(toolchains, 'metadata', return_value=data):
            rows = toolchains.gcc()
        self.assertEqual([r['version'] for r in rows], ['16.2.0-r3', '15.3.0-r2'])
        self.assertEqual(rows[0]['url'].rsplit('/', 1)[1], name('16.2.0', 3))

    def test_julia_takes_the_newest_stable_patch_of_each_line(self):
        def version(stable, kind='zip'):
            return {'stable': stable, 'files': [{'os': 'winnt', 'arch': 'x86_64', 'extension': kind,
                'url': 'https://julialang-s3.julialang.org/bin/winnt/x64/julia.'+kind, 'sha256': 'f'*64}]}
        data = {'1.13.0-rc1': version(False), '1.12.4': version(True), '1.12.3': version(True),
                '1.11.9': version(True), '1.10.11': version(True, 'exe'), '1.9.4': version(True), '1.8.5': version(True)}
        with patch.object(toolchains, 'metadata', return_value=data):
            rows = toolchains.julia()
        self.assertEqual([r['version'] for r in rows], ['1.12.4', '1.11.9', '1.9.4'])
        self.assertEqual(rows[0]['executable'], 'julia-1.12.4/bin/julia.exe')

    def test_dart_uses_the_published_checksum_of_each_line(self):
        index = {'prefixes': [f'channels/stable/release/{v}/' for v in ('3.12.0', '3.13.1', '3.13.0', 'latest', '3.11.4', '3.10.9')]}
        with patch.object(toolchains, 'metadata', return_value=index), \
                patch.object(toolchains, 'fetch', return_value=b'a'*64+b' *dartsdk-windows-x64-release.zip\n'):
            rows = toolchains.dart()
        self.assertEqual([r['version'] for r in rows], ['3.13.1', '3.12.0', '3.11.4'])
        self.assertEqual(rows[0]['sha256'], 'a'*64)
        self.assertTrue(rows[0]['url'].startswith('https://storage.googleapis.com/dart-archive/channels/stable/release/3.13.1/'))
