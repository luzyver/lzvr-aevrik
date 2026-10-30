import json
import hashlib
import tempfile
import unittest
import zipfile
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
        (self.root/'docs/RELEASE-NOTES.md').write_text('Fixture release notes')
        (self.root/'docs/PORTABLE-README.txt').write_text('Fixture portable instructions')

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
        self.assertEqual(len(list(out.glob('*.exe'))),1)
        with zipfile.ZipFile(out/'Aevrik-dev-windows-x64.zip') as archive:
            self.assertEqual(archive.read('Aevrik-dev-windows-x64.exe'),replacement)
        for artifact in result['artifacts']:
            digest=hashlib.sha256((out/artifact['name']).read_bytes()).hexdigest()
            self.assertEqual(artifact['sha256'],digest)
            self.assertIn(f"{digest}  {artifact['name']}",(out/'SHA256SUMS.txt').read_text())


if __name__ == '__main__': unittest.main()
