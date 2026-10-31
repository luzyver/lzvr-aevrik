use super::*;

fn feed(packages: &[catalog::Package]) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"schemaVersion":1,"packages":packages})).unwrap()
}

#[test]
fn catalog_updates_persist_additions_and_keep_omitted_versions() {
    let root = tempfile::tempdir().unwrap();
    let manager = PackageManager::new(root.path().to_owned()).unwrap();
    let mut extra = catalog::all().unwrap().remove(0);
    extra.id = "caddy-test-new".into();
    extra.version = "99.0.0".into();
    let before = manager.list().unwrap().len();
    assert_eq!(
        manager
            .apply_catalog(&feed(std::slice::from_ref(&extra)))
            .unwrap()
            .added,
        1
    );
    let reopened = PackageManager::new(root.path().to_owned()).unwrap();
    assert_eq!(reopened.list().unwrap().len(), before + 1);
    assert_eq!(reopened.apply_catalog(&feed(&[])).unwrap().added, 0);
    assert_eq!(reopened.list().unwrap().len(), before + 1);
    // Changing a pin must fail atomically, even alongside a valid new package.
    extra.sha256 = "f".repeat(64);
    let saved = std::fs::read(root.path().join(".catalog.json")).unwrap();
    assert!(reopened.apply_catalog(&feed(&[extra])).is_err());
    assert_eq!(
        std::fs::read(root.path().join(".catalog.json")).unwrap(),
        saved
    );
}

#[test]
fn untrusted_catalog_never_replaces_offline_data() {
    let root = tempfile::tempdir().unwrap();
    let manager = PackageManager::new(root.path().to_owned()).unwrap();
    for body in [
        b"not json".as_slice(),
        br#"{"schemaVersion":99,"packages":[]}"#,
    ] {
        assert!(manager.apply_catalog(body).is_err());
    }
    for (id, url) in [
        ("../escape", "https://github.com/a.zip"),
        ("new", "http://github.com/a.zip"),
        ("new", "https://localhost/a.zip"),
        ("new", "https://github.com.evil.invalid/a.zip"),
    ] {
        let mut package = catalog::all().unwrap().remove(0);
        package.id = id.into();
        package.url = url.into();
        assert!(manager.apply_catalog(&feed(&[package])).is_err());
    }
    assert!(!root.path().join(".catalog.json").exists());
    assert!(!manager.list().unwrap().is_empty());
}

#[test]
fn cached_package_installation_and_uninstall_use_its_exact_saved_identity() {
    use std::io::{Cursor, Write};
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let manager = PackageManager::new(root.path().to_owned()).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file("caddy.exe", zip::write::SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"test executable").unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    let mut package = catalog::all().unwrap().remove(0);
    package.id = "caddy-cached".into();
    package.version = "98.0.0".into();
    package.sha256 = archive::digest(&bytes);
    manager
        .apply_catalog(&feed(std::slice::from_ref(&package)))
        .unwrap();
    manager
        .record_location(&package.id, external.path())
        .unwrap();
    archive::publish(&bytes, &package, external.path()).unwrap();
    let reopened = PackageManager::new(root.path().to_owned()).unwrap();
    let items = reopened.list().unwrap();
    assert_eq!(
        items
            .iter()
            .find(|p| p.package.id == package.id)
            .unwrap()
            .status,
        "installed"
    );
    let executable = external
        .path()
        .join(&package.id)
        .join(&package.version)
        .join("caddy.exe");
    assert!(
        reopened
            .uninstall(&package.id, std::slice::from_ref(&executable))
            .is_err()
    );
    assert!(executable.exists());
    reopened.uninstall(&package.id, &[]).unwrap();
    assert!(!executable.exists());
    assert!(root.path().join(".catalog.json").is_file());
}
