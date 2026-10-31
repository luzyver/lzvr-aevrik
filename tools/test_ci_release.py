import json
import hashlib
import tempfile
import unittest
import zipfile
from unittest.mock import patch
from pathlib import Path
from ci_release import validate_version, package_release


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        for folder in ['src-tauri', 'crates/aevrik-engine', 'target/release', 'docs']:
            (self.root/folder).mkdir(parents=True)
        for name in ['package.json', 'package-lock.json']:
            (self.root/name).write_text(json.dumps({'version':'0.1.0-beta-1','packages':{'':{'version':'0.1.0-beta-1'}}}))
        (self.root/'src-tauri/tauri.conf.json').write_text(json.dumps({'version':'0.1.0-beta-1','identifier':'id.luzyver.aevrik.desktop'}))
        for name in ['src-tauri/Cargo.toml','crates/aevrik-engine/Cargo.toml']:
            (self.root/name).write_text('[package]\nversion="0.1.0-beta-1"\n')
        (self.root/'target/release/aevrik.exe').write_bytes(b'MZ controlled packaging fixture')
        self.bundle = self.root/'target/release/bundle/nsis'
        self.bundle.mkdir(parents=True)
        (self.bundle/'Aevrik_0.1.0-beta-1_x64-setup.exe').write_bytes(b'MZ setup fixture')
        (self.root/'docs/RELEASE-NOTES.md').write_text('Fixture release notes')
        (self.root/'docs/PORTABLE-README.txt').write_text('Fixture portable instructions')
        source=self.root/'crates/aevrik-engine/src/packages'
        source.mkdir(parents=True)
        (source/'catalog.json').write_text('[{"id":"fixture"}]')

    def test_release_includes_versioned_catalog_feed_and_hash(self):
        out=self.root/'release'
        with patch.dict('os.environ',{'GITHUB_SHA':'fixture-commit','AEVRIK_BUILT_AT':'1234567890'}):
            result=package_release(self.root,out,channel='dev')
        self.assertTrue((out/'runtime-catalog.json').exists(), 'Missing catalog release asset')
        feed=json.loads((out/'runtime-catalog.json').read_text())
        self.assertEqual(feed, {'schemaVersion':1,'packages':[{'id':'fixture'}]})
        self.assertIn('runtime-catalog.json', {item['name'] for item in result['artifacts']})
        self.assertEqual(result['builtAt'],1234567890)
        self.assertEqual(result['commit'],'fixture-commit')

    def tearDown(self):
        self.temp.cleanup()

    def test_tag_must_match_every_manifest(self):
        self.assertEqual(validate_version(self.root,'v0.1.0-beta-1'),'0.1.0-beta-1')
        with self.assertRaises(ValueError): validate_version(self.root,'v0.5.0')
        (self.root/'src-tauri/Cargo.toml').write_text('[package]\nversion="0.3.0"\n')
        with self.assertRaises(ValueError): validate_version(self.root,'v0.1.0-beta-1')

    def test_packaging_uses_validated_version_and_rejects_mismatched_tag(self):
        out=self.root/'release'
        result=package_release(self.root,out,'v0.1.0-beta-1')
        self.assertTrue((out/'Aevrik-0.1.0-beta-1-windows-x64.exe').exists())
        self.assertTrue((out/'Aevrik-0.1.0-beta-1-windows-x64.zip').exists())
        self.assertIn('sha256',result['artifacts'][0])
        with self.assertRaises(ValueError): package_release(self.root,out,'v0.3.0')

    def test_dev_build_replaces_stable_asset_names_and_records_actual_version(self):
        out=self.root/'release'
        package_release(self.root,out,channel='dev')
        replacement=b'MZ newer development build'
        (self.root/'target/release/aevrik.exe').write_bytes(replacement)
        result=package_release(self.root,out,channel='dev')
        self.assertEqual(result['version'],'0.1.0-beta-1')
        self.assertEqual(result['channel'],'dev')
        self.assertEqual((out/'Aevrik-dev-windows-x64.exe').read_bytes(),replacement)
        self.assertEqual(len(list(out.glob('*.exe'))),2)
        with zipfile.ZipFile(out/'Aevrik-dev-windows-x64.zip') as archive:
            self.assertEqual(archive.read('Aevrik-dev-windows-x64.exe'),replacement)
        for artifact in result['artifacts']:
            digest=hashlib.sha256((out/artifact['name']).read_bytes()).hexdigest()
            self.assertEqual(artifact['sha256'],digest)
            self.assertIn(f"{digest}  {artifact['name']}",(out/'SHA256SUMS.txt').read_text())

    def test_beta_build_replaces_its_assets_without_touching_dev(self):
        out=self.root/'release'
        package_release(self.root,out,channel='dev')
        dev_bytes=(out/'Aevrik-dev-windows-x64.exe').read_bytes()
        package_release(self.root,out,channel='beta')
        replacement=b'MZ newer beta build'
        (self.root/'target/release/aevrik.exe').write_bytes(replacement)
        result=package_release(self.root,out,channel='beta')
        self.assertEqual(result['version'],'0.1.0-beta-1')
        self.assertEqual(result['channel'],'beta')
        self.assertEqual((out/'Aevrik-beta-windows-x64.exe').read_bytes(),replacement)
        self.assertEqual((out/'Aevrik-dev-windows-x64.exe').read_bytes(),dev_bytes)
        with zipfile.ZipFile(out/'Aevrik-beta-windows-x64.zip') as archive:
            self.assertEqual(archive.read('Aevrik-beta-windows-x64.exe'),replacement)
        self.assertEqual({a['name'] for a in result['artifacts']},
                         {'Aevrik-beta-windows-x64.exe','Aevrik-beta-windows-x64.zip','Aevrik-beta-windows-x64-setup.exe','runtime-catalog.json'})
        for artifact in result['artifacts']:
            digest=hashlib.sha256((out/artifact['name']).read_bytes()).hexdigest()
            self.assertEqual(artifact['sha256'],digest)
            self.assertIn(f"{digest}  {artifact['name']}",(out/'SHA256SUMS.txt').read_text())

    def test_unknown_channel_is_rejected(self):
        with self.assertRaises(ValueError):
            package_release(self.root,self.root/'release',channel='feature')

    def test_checksums_use_unix_line_endings_for_publication_runner(self):
        out=self.root/'release'
        package_release(self.root,out,channel='beta')
        checksums=(out/'SHA256SUMS.txt').read_bytes()
        self.assertNotIn(b'\r',checksums)
        self.assertEqual(checksums.count(b'\n'),4)

    def test_setup_is_packaged_and_hashed_separately_from_portable(self):
        out = self.root/'release'
        result = package_release(self.root, out, channel='dev')
        setup = out/'Aevrik-dev-windows-x64-setup.exe'
        self.assertEqual(setup.read_bytes(), b'MZ setup fixture')
        metadata = next(item for item in result['artifacts'] if item['name'] == setup.name)
        self.assertEqual(metadata['sha256'], hashlib.sha256(b'MZ setup fixture').hexdigest())
        with zipfile.ZipFile(out/'Aevrik-dev-windows-x64.zip') as archive:
            self.assertNotIn(setup.name, archive.namelist())

    def test_packaging_rejects_missing_or_ambiguous_installer(self):
        setup = self.bundle/'Aevrik_0.1.0-beta-1_x64-setup.exe'
        setup.unlink()
        with self.assertRaisesRegex(ValueError, 'installer'):
            package_release(self.root, self.root/'release')
        setup.write_bytes(b'MZ setup')
        (self.bundle/'stale-setup.exe').write_bytes(b'MZ old')
        with self.assertRaisesRegex(ValueError, 'installer'):
            package_release(self.root, self.root/'release')


if __name__ == '__main__': unittest.main()
