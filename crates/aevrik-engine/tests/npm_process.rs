//! Opt-in Windows integration test using a real installed Node/npm distribution.
#![cfg(windows)]
use aevrik_engine::{Engine, ProjectInput, npm::NpmServiceInput};
use std::{
    net::TcpListener,
    path::Path,
    time::{Duration, Instant},
};

fn until(mut check: impl FnMut() -> bool) {
    let start = Instant::now();
    while !check() {
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "npm process timed out"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let destination = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &destination);
        } else if std::fs::hard_link(entry.path(), &destination).is_err() {
            std::fs::copy(entry.path(), &destination).unwrap();
        }
    }
}

#[test]
#[ignore = "requires an installed Node.js with bundled npm; run explicitly on Windows"]
fn npm_runs_hooks_with_selected_node_and_stops_descendants() {
    let original = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|p| p.join("node.exe"))
        .find(|p| aevrik_engine::npm::npm_cli(p).is_ok())
        .expect("Node with npm on PATH");
    let root = tempfile::tempdir().unwrap();
    // Exercise an alias: Windows temp paths can also use short names or different casing.
    let node_dir = root.path().join(".").join("Selected Node");
    std::fs::create_dir_all(&node_dir).unwrap();
    std::fs::copy(&original, node_dir.join("node.exe")).unwrap();
    copy_tree(
        &original.parent().unwrap().join("node_modules/npm"),
        &node_dir.join("node_modules/npm"),
    );
    let cwd = root.path().join("Project with spaces");
    std::fs::create_dir_all(&cwd).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    std::fs::write(cwd.join("package.json"),r#"{"scripts":{"predev":"node -e \"console.log('predev-ran')\"","dev":"node server.cjs","fail":"node -e \"process.exit(7)\""}}"#).unwrap();
    // A real npm script must resolve bare `node` to the selected runtime, not global PATH.
    std::fs::write(cwd.join("server.cjs"),format!(r#"const net=require('node:net'); console.log('selected-node:'+process.execPath); net.createServer().listen({port},'127.0.0.1',()=>console.log('npm-server-ready'));"#)).unwrap();
    let mut engine = Engine::open(&root.path().join("db.sqlite")).unwrap();
    let project = engine
        .add_project(ProjectInput {
            name: "Npm fixture".into(),
            path: cwd.to_string_lossy().into(),
            domain: "npm.test".into(),
            runtime: String::new(),
            service_id: None,
        })
        .unwrap();
    let service = engine
        .register_npm_service(NpmServiceInput {
            project_id: project.id,
            node_executable: node_dir.join("node.exe").to_string_lossy().into(),
            script: "dev".into(),
            port: Some(port),
        })
        .unwrap();
    assert_eq!(engine.snapshot().unwrap().services[0].state, "stopped");
    assert!(TcpListener::bind(("127.0.0.1", port)).is_ok());
    engine.start_service(&service.id).unwrap();
    until(|| {
        engine
            .snapshot()
            .unwrap()
            .logs
            .iter()
            .any(|l| l.message == "npm-server-ready")
    });
    let logs = engine.snapshot().unwrap().logs;
    assert!(logs.iter().any(|l| l.message == "predev-ran"));
    let actual_node = logs
        .iter()
        .find_map(|l| l.message.strip_prefix("selected-node:"))
        .unwrap_or_else(|| {
            panic!(
                "Missing selected Node output: {:?}",
                logs.iter().map(|l| &l.message).collect::<Vec<_>>()
            )
        });
    assert_eq!(
        Path::new(actual_node).canonicalize().unwrap(),
        node_dir.join("node.exe").canonicalize().unwrap(),
        "npm must run the selected Node executable"
    );
    assert!(TcpListener::bind(("127.0.0.1", port)).is_err());
    engine.stop_service(&service.id).unwrap();
    until(|| TcpListener::bind(("127.0.0.1", port)).is_ok());
    engine.clear_logs();
    engine.start_service(&service.id).unwrap();
    until(|| {
        engine
            .snapshot()
            .unwrap()
            .logs
            .iter()
            .any(|l| l.message == "npm-server-ready")
    });
    engine.clear_logs();
    engine.restart_service(&service.id).unwrap();
    until(|| {
        engine
            .snapshot()
            .unwrap()
            .logs
            .iter()
            .any(|l| l.message == "npm-server-ready")
    });
    drop(engine);
    until(|| TcpListener::bind(("127.0.0.1", port)).is_ok());
    let mut engine = Engine::open(&root.path().join("db.sqlite")).unwrap();
    engine.remove_service(&service.id).unwrap();
    let project_id = engine.snapshot().unwrap().projects[0].id.clone();
    let failing = engine
        .register_npm_service(NpmServiceInput {
            project_id,
            node_executable: node_dir.join("node.exe").to_string_lossy().into(),
            script: "fail".into(),
            port: None,
        })
        .unwrap();
    engine.start_service(&failing.id).unwrap();
    until(|| engine.snapshot().unwrap().services[0].state == "exited");
    assert_eq!(engine.snapshot().unwrap().services[0].exit_code, Some(7));
}
