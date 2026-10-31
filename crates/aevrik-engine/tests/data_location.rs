use aevrik_engine::data_location::{
    read_location, resolve_location, save_location, validate_destination,
};
use std::{fs, path::Path};

#[test]
fn selection_survives_relaunch_without_changing_previous_data() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("application");
    fs::create_dir(&app).unwrap();
    let previous = root.path().join("previous");
    fs::create_dir(&previous).unwrap();
    fs::write(previous.join("aevrik.sqlite"), b"old database").unwrap();
    let config = root.path().join("profile/data-location.json");
    let chosen = validate_destination(&root.path().join("Data with spaces"), &app).unwrap();
    save_location(&config, &chosen).unwrap();
    assert_eq!(read_location(&config).unwrap(), Some(chosen.clone()));
    assert_eq!(
        resolve_location(&previous, &previous, &config, None).unwrap(),
        Some(chosen)
    );
    assert_eq!(
        fs::read(previous.join("aevrik.sqlite")).unwrap(),
        b"old database"
    );
}

#[test]
fn explicit_environment_override_wins_without_reading_broken_preferences() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("data-location.json");
    fs::write(&config, b"invalid json").unwrap();
    let custom = root.path().join("isolated");
    assert_eq!(
        resolve_location(root.path(), root.path(), &config, Some(&custom)).unwrap(),
        Some(custom)
    );
    assert!(
        resolve_location(
            root.path(),
            root.path(),
            &config,
            Some(Path::new("relative"))
        )
        .is_err()
    );
    assert!(resolve_location(root.path(), root.path(), &config, None).is_err());
}

#[test]
fn missing_saved_drive_is_an_error_instead_of_an_empty_workspace() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("data-location.json");
    let chosen = root.path().join("removed drive");
    fs::create_dir(&chosen).unwrap();
    save_location(&config, &chosen).unwrap();
    fs::remove_dir(&chosen).unwrap();
    assert!(read_location(&config).is_err());
    assert!(!chosen.exists());
}

#[test]
fn existing_installations_keep_their_data_and_new_users_need_a_choice() {
    let root = tempfile::tempdir().unwrap();
    let preferred = root.path().join("preferred");
    let legacy = root.path().join("legacy");
    let config = root.path().join("data-location.json");
    assert_eq!(
        resolve_location(&preferred, &legacy, &config, None).unwrap(),
        None
    );
    fs::create_dir(&legacy).unwrap();
    fs::write(legacy.join("aevrik.sqlite"), b"legacy database").unwrap();
    assert_eq!(
        resolve_location(&preferred, &legacy, &config, None).unwrap(),
        Some(legacy.clone())
    );
    fs::create_dir(&preferred).unwrap();
    fs::write(preferred.join("aevrik.sqlite"), b"current database").unwrap();
    assert_eq!(
        resolve_location(&preferred, &legacy, &config, None).unwrap(),
        Some(preferred)
    );
}

#[test]
fn destination_rejects_relative_app_folders_unrelated_files_and_busy_workspaces() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    fs::create_dir(&app).unwrap();
    assert!(validate_destination(Path::new("relative"), &app).is_err());
    #[cfg(windows)]
    assert!(validate_destination(Path::new(r"\\server\share\data"), &app).is_err());
    assert!(validate_destination(&app, &app).is_err());
    assert!(validate_destination(&app.join("data"), &app).is_err());
    let unrelated = root.path().join("documents");
    fs::create_dir(&unrelated).unwrap();
    fs::write(unrelated.join("important.txt"), b"keep").unwrap();
    assert!(validate_destination(&unrelated, &app).is_err());
    let workspace = root.path().join("workspace");
    fs::create_dir(&workspace).unwrap();
    fs::write(workspace.join("aevrik.sqlite"), b"existing database").unwrap();
    let lock = fs::File::create(workspace.join(".aevrik.lock")).unwrap();
    lock.try_lock().unwrap();
    assert!(validate_destination(&workspace, &app).is_err());
    drop(lock);
    assert!(validate_destination(&workspace, &app).is_ok());
}

#[test]
fn failed_preference_update_keeps_previous_selection() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("data-location.json");
    save_location(&config, root.path()).unwrap();
    let before = fs::read(&config).unwrap();
    assert!(save_location(&config, Path::new("relative")).is_err());
    assert_eq!(fs::read(&config).unwrap(), before);
    // Replacing a directory is an actual filesystem write failure.
    assert!(save_location(root.path(), root.path()).is_err());
    assert_eq!(fs::read(&config).unwrap(), before);
}
