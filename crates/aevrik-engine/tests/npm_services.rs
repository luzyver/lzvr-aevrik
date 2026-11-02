use aevrik_engine::{
    Engine, ProjectInput,
    npm::{self, NpmServiceInput},
};
use std::path::Path;

#[test]
fn old_service_json_remains_a_manual_executable_service() {
    let input:aevrik_engine::ServiceInput=serde_json::from_str(r#"{"name":"Old service","executable":"C:\\server.exe","cwd":"C:\\project","args":["--serve"],"port":8080}"#).unwrap();
    assert!(input.npm_script.is_none());
    assert_eq!(input.args, vec!["--serve"]);
    assert!(
        serde_json::to_value(&input)
            .unwrap()
            .get("npmScript")
            .is_none()
    );
}

fn project(engine: &mut Engine, path: &Path) -> String {
    engine
        .add_project(ProjectInput {
            name: "Portfolio".into(),
            path: path.to_string_lossy().into(),
            domain: "portfolio.test".into(),
            runtime: String::new(),
            service_id: None,
        })
        .unwrap()
        .id
}

fn node_fixture(root: &Path) -> String {
    let dir = root.join("Node runtime");
    std::fs::create_dir_all(dir.join("node_modules/npm/bin")).unwrap();
    std::fs::write(dir.join("node.exe"), "not executed").unwrap();
    std::fs::write(dir.join("node_modules/npm/bin/npm-cli.js"), "not executed").unwrap();
    dir.join("node.exe").to_string_lossy().into()
}

#[test]
fn discovers_scripts_without_executing_them_and_rejects_invalid_manifests() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("package.json");
    std::fs::write(
        &manifest,
        r#"{"scripts":{"dev":"node server.js","build":"echo build"}}"#,
    )
    .unwrap();
    let scripts = npm::scripts(dir.path()).unwrap();
    assert_eq!(
        scripts.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        vec!["build", "dev"]
    );
    assert_eq!(scripts[1].command, "node server.js");
    for invalid in ["broken", r#"{"scripts":{"dev":42}}"#, r#"{"scripts":[]}"#] {
        std::fs::write(&manifest, invalid).unwrap();
        assert!(npm::scripts(dir.path()).is_err());
    }
    std::fs::write(&manifest, "{}").unwrap();
    assert!(npm::scripts(dir.path()).unwrap().is_empty());
    std::fs::write(&manifest, vec![b' '; 1_048_577]).unwrap();
    assert!(npm::scripts(dir.path()).is_err());
}

#[test]
fn npm_service_is_linked_atomically_persists_and_revalidates_script() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("metadata.sqlite");
    let manifest = dir.path().join("package.json");
    std::fs::write(&manifest, r#"{"scripts":{"dev":"node server.js"}}"#).unwrap();
    let mut engine = Engine::open(&db).unwrap();
    let id = project(&mut engine, dir.path());
    let node = node_fixture(dir.path());
    let input = |script: &str| NpmServiceInput {
        project_id: id.clone(),
        node_executable: node.clone(),
        script: script.into(),
        port: None,
    };
    assert!(engine.register_npm_service(input("missing")).is_err());
    assert!(engine.snapshot().unwrap().services.is_empty());
    let service = engine.register_npm_service(input("dev")).unwrap();
    assert_eq!(service.config.npm_script.as_deref(), Some("dev"));
    assert_eq!(&service.config.args[1..], &["run", "--", "dev"]);
    assert!(engine.register_npm_service(input("dev")).is_err());
    assert_eq!(engine.snapshot().unwrap().services.len(), 1);
    drop(engine);
    let mut engine = Engine::open(&db).unwrap();
    let snapshot = engine.snapshot().unwrap();
    assert_eq!(
        snapshot.projects[0].service_id.as_deref(),
        Some(service.id.as_str())
    );
    assert_eq!(
        snapshot.services[0].config.npm_script.as_deref(),
        Some("dev")
    );
    std::fs::write(&manifest, r#"{"scripts":{}}"#).unwrap();
    assert!(
        engine
            .start_service(&service.id)
            .unwrap_err()
            .to_string()
            .contains("script")
    );
}

#[test]
fn missing_bundled_npm_and_unknown_project_do_not_create_services() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::open(&dir.path().join("db.sqlite")).unwrap();
    let id = project(&mut engine, dir.path());
    std::fs::write(
        dir.path().join("package.json"),
        r#"{"scripts":{"dev":"echo ok"}}"#,
    )
    .unwrap();
    let node = node_fixture(dir.path());
    std::fs::remove_file(
        Path::new(&node)
            .parent()
            .unwrap()
            .join("node_modules/npm/bin/npm-cli.js"),
    )
    .unwrap();
    assert!(
        engine
            .register_npm_service(NpmServiceInput {
                project_id: id,
                node_executable: node,
                script: "dev".into(),
                port: None
            })
            .is_err()
    );
    assert!(engine.project_scripts("missing").is_err());
    assert!(engine.snapshot().unwrap().services.is_empty());
}
