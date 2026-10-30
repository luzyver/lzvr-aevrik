use aevrik_engine::{Engine, ServiceInput};
use std::{
    net::TcpListener,
    time::{Duration, Instant},
};

fn service(dir: &std::path::Path, args: Vec<String>, port: Option<u16>) -> ServiceInput {
    ServiceInput {
        name: "Fixture".into(),
        executable: env!("CARGO_BIN_EXE_process-fixture").into(),
        args,
        cwd: dir.to_string_lossy().into(),
        port,
    }
}
fn until(mut condition: impl FnMut() -> bool) {
    let start = Instant::now();
    while !condition() {
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "condition timed out"
        );
        std::thread::sleep(Duration::from_millis(30));
    }
}

#[test]
fn owned_process_can_start_stop_and_restart_with_captured_output() {
    let dir = tempfile::tempdir().expect("temp");
    let mut engine = Engine::open(&dir.path().join("db.sqlite")).expect("engine");
    let listener = TcpListener::bind("127.0.0.1:0").expect("port");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);
    let item = engine
        .register_service(service(dir.path(), vec![port.to_string()], Some(port)))
        .expect("register");
    engine.start_service(&item.id).expect("start");
    assert!(engine.start_service(&item.id).is_err());
    until(|| {
        engine
            .snapshot()
            .expect("snapshot")
            .logs
            .iter()
            .any(|x| x.message.contains("fixture ready"))
    });
    let running = engine.snapshot().expect("snapshot");
    assert_eq!(running.services[0].state, "running");
    assert!(running.services[0].pid.is_some());
    engine.stop_service(&item.id).expect("stop");
    engine.stop_service(&item.id).expect("idempotent stop");
    assert_eq!(
        engine.snapshot().expect("snapshot").services[0].state,
        "stopped"
    );
    engine.start_service(&item.id).expect("restart");
    drop(engine);
    until(|| TcpListener::bind(("127.0.0.1", port)).is_ok());
}

#[test]
fn busy_port_does_not_start_or_stop_an_unowned_listener() {
    let dir = tempfile::tempdir().expect("temp");
    let listener = TcpListener::bind("127.0.0.1:0").expect("listen");
    let port = listener.local_addr().expect("addr").port();
    let mut engine = Engine::open(&dir.path().join("db.sqlite")).expect("engine");
    let item = engine
        .register_service(service(dir.path(), vec![port.to_string()], Some(port)))
        .expect("register");
    assert!(engine.start_service(&item.id).is_err());
    assert_eq!(
        engine.snapshot().expect("snapshot").services[0].state,
        "stopped"
    );
    assert!(listener.local_addr().is_ok());
}

#[test]
fn early_exit_is_visible_and_arguments_are_literal() {
    let dir = tempfile::tempdir().expect("temp");
    let mut engine = Engine::open(&dir.path().join("db.sqlite")).expect("engine");
    let item = engine
        .register_service(service(dir.path(), vec!["exit".into()], None))
        .expect("register");
    engine.start_service(&item.id).expect("start");
    until(|| engine.snapshot().expect("snapshot").services[0].state == "exited");
    assert_eq!(
        engine.snapshot().expect("snapshot").services[0].exit_code,
        Some(7)
    );
    let literal = "two words & $(not-a-command)";
    let echo = engine
        .register_service(service(
            dir.path(),
            vec!["echo".into(), literal.into()],
            None,
        ))
        .expect("register echo");
    engine.start_service(&echo.id).expect("start echo");
    until(|| {
        engine
            .snapshot()
            .expect("snapshot")
            .logs
            .iter()
            .any(|x| x.message == literal)
    });
}

#[cfg(windows)]
#[test]
fn closing_engine_terminates_descendants_and_releases_their_ports() {
    let dir = tempfile::tempdir().expect("temp");
    let listener = TcpListener::bind("127.0.0.1:0").expect("port");
    let port = listener.local_addr().expect("address").port();
    drop(listener);
    let mut engine = Engine::open(&dir.path().join("db.sqlite")).expect("engine");
    let item = engine
        .register_service(service(
            dir.path(),
            vec!["tree".into(), port.to_string()],
            None,
        ))
        .expect("register");
    engine.start_service(&item.id).expect("start parent");
    until(|| {
        engine
            .snapshot()
            .expect("snapshot")
            .logs
            .iter()
            .any(|l| l.message == "fixture ready")
    });
    assert!(TcpListener::bind(("127.0.0.1", port)).is_err());
    drop(engine);
    until(|| TcpListener::bind(("127.0.0.1", port)).is_ok());
}
