use super::*;
use std::io::{Cursor, Write};
use zip::{ZipWriter, write::SimpleFileOptions};

fn fixture(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in entries {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(body).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

#[test]
fn catalog_available_without_installed_services() {
    let root = tempfile::tempdir().unwrap();
    let manager = PackageManager::new(root.path().to_owned()).unwrap();
    let list = manager.list().unwrap();
    assert!(list.len() >= 4);
    assert!(list.iter().all(|p| p.status == "available"));
    assert!(manager.install("untrusted-url").is_err());
}

#[test]
fn checksum_failure_and_traversal_leave_no_installation() {
    let root = tempfile::tempdir().unwrap();
    let good = fixture(&[("caddy.exe", b"fixture")]);
    let mut package = catalog::all().unwrap().remove(0);
    assert!(archive::unpack(&good[..], &package, root.path()).is_err());
    assert!(!root.path().join("caddy.exe").exists());
    let evil = fixture(&[("../escaped.exe", b"bad")]);
    package.sha256 = archive::digest(&evil);
    assert!(archive::unpack(&evil[..], &package, root.path()).is_err());
    assert!(!root.path().parent().unwrap().join("escaped.exe").exists());
}

#[test]
fn missing_executable_is_rejected_and_valid_archive_installs_atomically() {
    let root = tempfile::tempdir().unwrap();
    let mut package = catalog::all().unwrap().remove(0);
    let missing = fixture(&[("readme.txt", b"not executable")]);
    package.sha256 = archive::digest(&missing);
    assert!(archive::publish(&missing[..], &package, root.path()).is_err());
    assert!(!root.path().join(&package.id).exists());
    let good = fixture(&[("caddy.exe", b"fixture")]);
    package.sha256 = archive::digest(&good);
    archive::publish(&good[..], &package, root.path()).unwrap();
    let exe = root
        .path()
        .join(&package.id)
        .join(&package.version)
        .join("caddy.exe");
    assert_eq!(std::fs::read(&exe).unwrap(), b"fixture");
    assert!(archive::publish(&good[..], &package, root.path()).is_err());
    assert_eq!(std::fs::read(&exe).unwrap(), b"fixture");
}

#[test]
fn windows_aliases_and_duplicate_paths_are_rejected() {
    for name in [
        "C:/escape.exe",
        "file:stream",
        "CON",
        "folder/../escape",
        "trailing. ",
        "folder\\escape.exe",
    ] {
        assert!(archive::safe_path(name).is_err(), "{name}");
    }
    let root = tempfile::tempdir().unwrap();
    let bytes = fixture(&[("caddy.exe", b"first"), ("CADDY.EXE", b"second")]);
    let mut package = catalog::all().unwrap().remove(0);
    package.sha256 = archive::digest(&bytes);
    assert!(archive::unpack(&bytes[..], &package, root.path()).is_err());
}

#[test]
fn cross_instance_lock_rejects_duplicate_download() {
    let root = tempfile::tempdir().unwrap();
    let manager = PackageManager::new(root.path().to_owned()).unwrap();
    let lock = std::fs::File::create(root.path().join(".locks/caddy.lock")).unwrap();
    lock.lock().unwrap();
    let error = manager.install("caddy").unwrap_err().to_string();
    assert!(error.contains("already being installed"), "{error}");
    assert_eq!(manager.list().unwrap()[0].status, "available");
}

#[test]
fn symlinks_are_rejected_and_installed_marker_survives_reopen() {
    let root = tempfile::tempdir().unwrap();
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.add_symlink("caddy.exe", "../outside", SimpleFileOptions::default())
        .unwrap();
    let bytes = zip.finish().unwrap().into_inner();
    let mut package = catalog::all().unwrap().remove(0);
    package.sha256 = archive::digest(&bytes);
    assert!(archive::publish(&bytes, &package, root.path()).is_err());
    let good = fixture(&[("caddy.exe", b"fixture")]);
    package.sha256 = archive::digest(&good);
    archive::publish(&good, &package, root.path()).unwrap();
    assert!(archive::installed(root.path(), &package).is_some());
    std::fs::remove_file(
        root.path()
            .join(&package.id)
            .join(&package.version)
            .join("caddy.exe"),
    )
    .unwrap();
    assert!(archive::installed(root.path(), &package).is_none());
}
#[test]
fn catalog_has_product_breadth_and_multiple_versions() {
    let all = catalog::all().unwrap();
    let names: std::collections::HashSet<_> = all.iter().map(|p| p.name.as_str()).collect();
    assert!(
        names.len() >= 10,
        "Expected at least ten real products, got {}",
        names.len()
    );
    assert!(all.iter().filter(|p| p.name == "PHP").count() >= 7);
    assert!(all.iter().filter(|p| p.name == "Node.js").count() >= 12);
    for name in names {
        assert!(
            all.iter().filter(|p| p.name == name).count() >= 2,
            "{name} needs version choices"
        );
    }
    let ids: std::collections::HashSet<_> = all.iter().map(|p| &p.id).collect();
    assert_eq!(ids.len(), all.len(), "Release IDs must be unique");
    for (id, version) in [
        ("caddy", "2.11.7"),
        ("php84", "8.4.27-1"),
        ("node24", "24.22.2"),
        ("mariadb118", "11.8.9"),
    ] {
        assert_eq!(all.iter().find(|p| p.id == id).unwrap().version, version);
    }
}

#[test]
fn two_versions_install_side_by_side_without_replacing_each_other() {
    let root = tempfile::tempdir().unwrap();
    let mut first = catalog::all()
        .unwrap()
        .into_iter()
        .find(|p| p.id == "caddy")
        .unwrap();
    let first_bytes = fixture(&[("caddy.exe", b"first version")]);
    first.sha256 = archive::digest(&first_bytes);
    archive::publish(&first_bytes, &first, root.path()).unwrap();
    let mut second = first.clone();
    second.id = "caddy-other-version".into();
    second.version = "2.0.0".into();
    let second_bytes = fixture(&[("caddy.exe", b"second version")]);
    second.sha256 = archive::digest(&second_bytes);
    archive::publish(&second_bytes, &second, root.path()).unwrap();
    assert_eq!(
        std::fs::read(archive::installed(root.path(), &first).unwrap()).unwrap(),
        b"first version"
    );
    assert_eq!(
        std::fs::read(archive::installed(root.path(), &second).unwrap()).unwrap(),
        b"second version"
    );
}

#[test]
fn version_02_completion_markers_remain_readable() {
    let root = tempfile::tempdir().unwrap();
    let mut package = catalog::all()
        .unwrap()
        .into_iter()
        .find(|p| p.id == "caddy")
        .unwrap();
    let bytes = fixture(&[("caddy.exe", b"old installation")]);
    package.sha256 = archive::digest(&bytes);
    archive::publish(&bytes, &package, root.path()).unwrap();
    let marker_path = root
        .path()
        .join(&package.id)
        .join(&package.version)
        .join(".aevrik-package.json");
    let mut marker: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&marker_path).unwrap()).unwrap();
    for key in ["family", "architecture", "channel"] {
        marker.as_object_mut().unwrap().remove(key);
    }
    std::fs::write(marker_path, serde_json::to_vec(&marker).unwrap()).unwrap();
    assert!(archive::installed(root.path(), &package).is_some());
}

#[test]
fn chosen_drive_and_default_survive_restart_without_moving_legacy_packages() {
    let metadata = tempfile::tempdir().unwrap();
    let drive = tempfile::tempdir().unwrap();
    let manager = PackageManager::new(metadata.path().to_owned()).unwrap();
    manager.set_install_root(drive.path()).unwrap();
    let package = catalog::all().unwrap().remove(0);
    manager.record_location(&package.id, drive.path()).unwrap();
    let reopened = PackageManager::new(metadata.path().to_owned()).unwrap();
    assert_eq!(
        reopened.install_root().unwrap(),
        drive.path().canonicalize().unwrap()
    );
    assert_eq!(
        reopened.package_root(&package.id).unwrap(),
        drive.path().canonicalize().unwrap()
    );
    assert_eq!(reopened.package_root("php84").unwrap(), metadata.path());
    assert!(
        manager
            .set_install_root(std::path::Path::new("relative"))
            .is_err()
    );
}

#[test]
fn uninstall_refuses_references_then_removes_only_selected_owned_version() {
    let root = tempfile::tempdir().unwrap();
    let drive = tempfile::tempdir().unwrap();
    let manager = PackageManager::new(root.path().to_owned()).unwrap();
    let package = catalog::all().unwrap().remove(0);
    let dir = drive.path().join(&package.id).join(&package.version);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(&package.executable), b"fixture").unwrap();
    std::fs::write(
        dir.join(".aevrik-package.json"),
        serde_json::to_vec(&package).unwrap(),
    )
    .unwrap();
    manager.record_location(&package.id, drive.path()).unwrap();
    let neighbor = drive.path().join("keep.txt");
    std::fs::write(&neighbor, b"unrelated").unwrap();
    assert!(
        manager
            .uninstall(&package.id, &[dir.join(&package.executable)])
            .is_err()
    );
    assert!(dir.exists());
    let nested = dir.join("nested-runtime");
    std::fs::create_dir(&nested).unwrap();
    std::fs::write(
        nested.join(".aevrik-package.json"),
        b"other managed installation",
    )
    .unwrap();
    assert!(manager.uninstall(&package.id, &[]).is_err());
    assert!(nested.exists());
    std::fs::remove_file(nested.join(".aevrik-package.json")).unwrap();
    std::fs::remove_dir(&nested).unwrap();
    manager.uninstall(&package.id, &[]).unwrap();
    assert!(!dir.exists());
    assert_eq!(std::fs::read(neighbor).unwrap(), b"unrelated");
    assert_eq!(manager.list().unwrap()[0].status, "available");
}

#[test]
fn uninstall_rejects_missing_or_modified_ownership_marker() {
    let root = tempfile::tempdir().unwrap();
    let manager = PackageManager::new(root.path().to_owned()).unwrap();
    let package = catalog::all().unwrap().remove(0);
    let dir = root.path().join(&package.id).join(&package.version);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(&package.executable), b"personal").unwrap();
    assert!(manager.uninstall(&package.id, &[]).is_err());
    std::fs::write(dir.join(".aevrik-package.json"), b"{}").unwrap();
    assert!(manager.uninstall(&package.id, &[]).is_err());
    assert!(dir.join(&package.executable).exists());
}

#[test]
fn file_backed_archive_verifies_checksum_before_publication() {
    let root = tempfile::tempdir().unwrap();
    let mut file = tempfile::NamedTempFile::new_in(root.path()).unwrap();
    let bytes = fixture(&[("caddy.exe", b"streamed fixture")]);
    file.write_all(&bytes).unwrap();
    let mut package = catalog::all().unwrap().remove(0);
    assert!(archive::publish_reader(file.as_file_mut(), &package, root.path()).is_err());
    package.sha256 = archive::digest(&bytes);
    archive::publish_reader(file.as_file_mut(), &package, root.path()).unwrap();
    assert_eq!(
        std::fs::read(archive::installed(root.path(), &package).unwrap()).unwrap(),
        b"streamed fixture"
    );
}

#[test]
fn bounded_download_is_written_on_the_selected_drive_and_rejects_overflow() {
    let root = tempfile::tempdir().unwrap();
    let bytes = vec![42_u8; 1024];
    let file = download::write_stream(&mut Cursor::new(&bytes), root.path(), 1024, |_, _| {}, None)
        .unwrap();
    assert_eq!(file.path().parent().unwrap(), root.path());
    assert_eq!(std::fs::read(file.path()).unwrap(), bytes);
    assert!(
        download::write_stream(&mut Cursor::new(&bytes), root.path(), 1000, |_, _| {}, None)
            .is_err()
    );
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
}

#[test]
#[cfg(windows)]
fn uninstall_refuses_junctions_without_touching_external_files() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let manager = PackageManager::new(root.path().to_owned()).unwrap();
    let package = catalog::all().unwrap().remove(0);
    let directory = root.path().join(&package.id).join(&package.version);
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join(&package.executable), b"fixture").unwrap();
    std::fs::write(
        directory.join(".aevrik-package.json"),
        serde_json::to_vec(&package).unwrap(),
    )
    .unwrap();
    std::fs::write(outside.path().join("keep.txt"), b"external").unwrap();
    let junction = directory.join("external-link");
    let status = std::process::Command::new("cmd")
        .args(["/c", "mklink", "/J"])
        .arg(&junction)
        .arg(outside.path())
        .output()
        .unwrap();
    assert!(status.status.success(), "{:?}", status);
    assert!(manager.uninstall(&package.id, &[]).is_err());
    assert_eq!(
        std::fs::read(outside.path().join("keep.txt")).unwrap(),
        b"external"
    );
    std::fs::remove_dir(junction).unwrap();
}
