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
