use aevrik_engine::{Engine, ProjectInput, ServiceInput};

#[test]
fn identifier_change_keeps_existing_storage_without_copying_or_overwriting() {
    let root = tempfile::tempdir().unwrap();
    let old = root.path().join("dev.aevrik.desktop");
    let new = root.path().join("id.luzyver.aevrik.desktop");
    assert_eq!(aevrik_engine::data_directory(&new, &old), new);
    std::fs::create_dir_all(&old).unwrap();
    std::fs::write(old.join("aevrik.sqlite"), b"old data").unwrap();
    assert_eq!(aevrik_engine::data_directory(&new, &old), old);
    assert!(!new.exists());
    std::fs::create_dir_all(&new).unwrap();
    std::fs::write(new.join("aevrik.sqlite"), b"new data").unwrap();
    assert_eq!(aevrik_engine::data_directory(&new, &old), new);
    assert_eq!(
        std::fs::read(old.join("aevrik.sqlite")).unwrap(),
        b"old data"
    );
}

#[test]
fn removing_a_stopped_service_unlinks_projects_and_preserves_files() {
    let dir = tempfile::tempdir().unwrap();
    let executable = std::path::PathBuf::from(env!("CARGO_BIN_EXE_process-fixture"));
    let mut engine = Engine::open(&dir.path().join("metadata.sqlite")).unwrap();
    let service = engine
        .register_service(ServiceInput {
            npm_script: None,
            name: "Fixture".into(),
            executable: executable.to_string_lossy().into(),
            args: vec!["0".into()],
            cwd: dir.path().to_string_lossy().into(),
            port: None,
        })
        .unwrap();
    let mut input = project(dir.path(), "linked.test");
    input.service_id = Some(service.id.clone());
    engine.add_project(input).unwrap();
    engine.start_service(&service.id).unwrap();
    assert!(engine.remove_service(&service.id).is_err());
    engine.stop_service(&service.id).unwrap();
    engine.remove_service(&service.id).unwrap();
    let snapshot = engine.snapshot().unwrap();
    assert!(snapshot.services.is_empty());
    assert!(snapshot.projects[0].service_id.is_none());
    assert!(executable.is_file());
}

fn project(path: &std::path::Path, domain: &str) -> ProjectInput {
    ProjectInput {
        name: "Atlas".into(),
        path: path.to_string_lossy().into(),
        domain: domain.into(),
        runtime: "PHP".into(),
        service_id: None,
    }
}

fn fixture(dir: &std::path::Path, name: &str) -> ServiceInput {
    ServiceInput {
        npm_script: None,
        name: name.into(),
        executable: env!("CARGO_BIN_EXE_process-fixture").into(),
        args: vec!["0".into()],
        cwd: dir.to_string_lossy().into(),
        port: None,
    }
}

#[test]
fn projects_can_be_edited_and_removed_without_touching_their_folder() {
    let dir = tempfile::tempdir().unwrap();
    let moved = dir.path().join("moved");
    std::fs::create_dir(&moved).unwrap();
    let path = dir.path().join("metadata.sqlite");
    let mut engine = Engine::open(&path).unwrap();
    let first = engine
        .add_project(project(dir.path(), "atlas.test"))
        .unwrap();
    let mut other = project(dir.path(), "other.test");
    other.name = "Other".into();
    let second = engine.add_project(other).unwrap();
    // Another project's name or domain is refused; its own values are not a conflict.
    let folder = dir.path().to_string_lossy().to_string();
    assert!(
        engine
            .edit_project(
                &second.id,
                "Other".into(),
                folder.clone(),
                "ATLAS.test".into()
            )
            .is_err()
    );
    assert!(
        engine
            .edit_project(
                &second.id,
                "atlas".into(),
                folder.clone(),
                "other.test".into()
            )
            .is_err()
    );
    assert!(
        engine
            .edit_project(&first.id, "Atlas".into(), folder, "nope.com".into())
            .is_err()
    );
    engine
        .edit_project(
            &first.id,
            "Atlas 2".into(),
            moved.to_string_lossy().into(),
            "Atlas-2.test".into(),
        )
        .unwrap();
    engine.remove_project(&second.id).unwrap();
    assert!(engine.remove_project(&second.id).is_err());
    drop(engine);
    let snapshot = Engine::open(&path).unwrap().snapshot().unwrap();
    assert_eq!(snapshot.projects.len(), 1);
    assert_eq!(snapshot.projects[0].name, "Atlas 2");
    assert_eq!(snapshot.projects[0].domain, "atlas-2.test");
    assert!(snapshot.projects[0].path.ends_with("moved"));
    assert_eq!(snapshot.projects[0].runtime, "PHP");
    assert!(moved.is_dir() && dir.path().is_dir());
}

#[test]
fn a_stopped_service_can_be_edited_and_a_running_one_cannot() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("metadata.sqlite");
    let mut engine = Engine::open(&path).unwrap();
    let service = engine
        .register_service(fixture(dir.path(), "Fixture"))
        .unwrap();
    let mut edited = fixture(dir.path(), "Renamed");
    edited.port = Some(8123);
    edited.args = vec!["0".into(), "extra".into()];
    engine.start_service(&service.id).unwrap();
    assert!(engine.update_service(&service.id, edited.clone()).is_err());
    engine.stop_service(&service.id).unwrap();
    let mut invalid = edited.clone();
    invalid.executable = "missing.exe".into();
    assert!(engine.update_service(&service.id, invalid).is_err());
    assert!(engine.update_service("missing", edited.clone()).is_err());
    engine.update_service(&service.id, edited).unwrap();
    drop(engine);
    let snapshot = Engine::open(&path).unwrap().snapshot().unwrap();
    assert_eq!(snapshot.services[0].id, service.id);
    assert_eq!(snapshot.services[0].config.name, "Renamed");
    assert_eq!(snapshot.services[0].config.port, Some(8123));
    assert_eq!(snapshot.services[0].config.args, ["0", "extra"]);
}

#[test]
fn a_service_that_exited_by_itself_is_not_restored() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("metadata.sqlite");
    let mut engine = Engine::open(&path).unwrap();
    engine.set_restore_services(true).unwrap();
    let kept = engine
        .register_service(fixture(dir.path(), "Kept"))
        .unwrap();
    let mut failing = fixture(dir.path(), "Failing");
    failing.args = vec!["exit".into()];
    let failing = engine.register_service(failing).unwrap();
    assert!(engine.start_all().unwrap().is_empty());
    // Wait for the process to end without telling the engine about it.
    std::thread::sleep(std::time::Duration::from_secs(2));
    engine.shutdown().unwrap();
    drop(engine);
    let mut engine = Engine::open(&path).unwrap();
    engine.restore_services().unwrap();
    let snapshot = engine.snapshot().unwrap();
    let state = |id: &str| {
        snapshot
            .services
            .iter()
            .find(|s| s.id == id)
            .unwrap()
            .state
            .clone()
    };
    assert_eq!(state(&kept.id), "running");
    // A restored process would be running or exited again, never stopped.
    assert_eq!(state(&failing.id), "stopped");
}

#[test]
fn running_services_are_restored_only_when_the_preference_is_enabled() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("metadata.sqlite");
    let mut engine = Engine::open(&path).unwrap();
    let kept = engine
        .register_service(fixture(dir.path(), "Kept"))
        .unwrap();
    let stopped = engine
        .register_service(fixture(dir.path(), "Stopped"))
        .unwrap();
    assert!(engine.start_all().unwrap().is_empty());
    engine.stop_service(&stopped.id).unwrap();
    drop(engine);
    let mut engine = Engine::open(&path).unwrap();
    engine.restore_services().unwrap();
    let snapshot = engine.snapshot().unwrap();
    assert!(!snapshot.restore_services);
    assert!(snapshot.services.iter().all(|s| s.state == "stopped"));
    engine.start_service(&kept.id).unwrap();
    engine.set_restore_services(true).unwrap();
    drop(engine);
    let mut engine = Engine::open(&path).unwrap();
    engine.restore_services().unwrap();
    let snapshot = engine.snapshot().unwrap();
    assert!(snapshot.restore_services);
    let state = |id: &str| {
        snapshot
            .services
            .iter()
            .find(|s| s.id == id)
            .unwrap()
            .state
            .clone()
    };
    assert_eq!(state(&kept.id), "running");
    assert_eq!(state(&stopped.id), "stopped");
    engine.stop_all().unwrap();
    assert!(
        engine
            .snapshot()
            .unwrap()
            .services
            .iter()
            .all(|s| s.state != "running")
    );
}

#[test]
fn project_persists_and_domains_are_case_insensitively_unique() {
    let dir = tempfile::tempdir().expect("temp");
    let database = dir.path().join("metadata.sqlite");
    let mut engine = Engine::open(&database).expect("engine");
    let created = engine
        .add_project(project(dir.path(), "Atlas.test"))
        .expect("create");
    assert_eq!(created.domain, "atlas.test");
    let mut duplicate_domain = project(dir.path(), "ATLAS.TEST");
    duplicate_domain.name = "Different name".into();
    assert!(engine.add_project(duplicate_domain).is_err());
    assert!(
        engine
            .add_project(project(dir.path(), "other.test"))
            .is_err()
    );
    drop(engine);
    let mut reopened = Engine::open(&database).expect("reopen");
    let snapshot = reopened.snapshot().expect("snapshot");
    assert_eq!(snapshot.projects.len(), 1);
    assert_eq!(snapshot.projects[0].id, created.id);
    assert_eq!(snapshot.projects[0].state, "stopped");
}

#[test]
fn invalid_paths_domains_and_executables_do_not_create_records() {
    let dir = tempfile::tempdir().expect("temp");
    let mut engine = Engine::open(&dir.path().join("metadata.sqlite")).expect("engine");
    assert!(
        engine
            .add_project(project(&dir.path().join("missing"), "app.test"))
            .is_err()
    );
    for domain in ["localhost", "a..test", "-a.test", "a.test/path", "a.com"] {
        assert!(engine.add_project(project(dir.path(), domain)).is_err());
    }
    let invalid = ServiceInput {
        npm_script: None,
        name: "Bad".into(),
        executable: "cmd.exe".into(),
        args: vec![],
        cwd: dir.path().to_string_lossy().into(),
        port: None,
    };
    assert!(engine.register_service(invalid).is_err());
    let snapshot = engine.snapshot().expect("snapshot");
    assert!(snapshot.projects.is_empty());
    assert!(snapshot.services.is_empty());
}

#[test]
fn preferences_persist_and_invalid_theme_is_rejected() {
    let dir = tempfile::tempdir().expect("temp");
    let path = dir.path().join("metadata.sqlite");
    let mut engine = Engine::open(&path).expect("engine");
    engine.set_theme("light").expect("save");
    assert!(engine.set_theme("banana").is_err());
    drop(engine);
    assert_eq!(
        Engine::open(&path)
            .expect("reopen")
            .snapshot()
            .expect("snapshot")
            .theme,
        "light"
    );
}

#[test]
fn corrupt_database_is_reported_without_overwriting_it() {
    let dir = tempfile::tempdir().expect("temp");
    let path = dir.path().join("metadata.sqlite");
    std::fs::write(&path, "not a database").expect("write");
    assert!(Engine::open(&path).is_err());
    assert_eq!(
        std::fs::read_to_string(path).expect("read"),
        "not a database"
    );
}

#[test]
fn project_properties_persist_and_unknown_services_are_rejected() {
    let dir = tempfile::tempdir().expect("temp");
    let path = dir.path().join("metadata.sqlite");
    let mut engine = Engine::open(&path).expect("engine");
    let item = engine
        .add_project(project(dir.path(), "atlas.test"))
        .expect("project");
    assert!(
        engine
            .update_project(&item.id, Some("missing".into()), "Invalid".into())
            .is_err()
    );
    assert_eq!(
        engine.snapshot().expect("snapshot").projects[0].runtime,
        "PHP"
    );
    engine
        .update_project(&item.id, None, "PHP 8.4".into())
        .expect("update");
    drop(engine);
    assert_eq!(
        Engine::open(&path)
            .expect("reopen")
            .snapshot()
            .expect("snapshot")
            .projects[0]
            .runtime,
        "PHP 8.4"
    );
}
