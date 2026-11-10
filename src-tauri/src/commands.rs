use aevrik_engine::{Engine, ProjectInput, ServiceInput, Snapshot};
use std::sync::{Arc, Mutex};
use tauri::State;

pub struct AppState(
    pub Arc<Mutex<Engine>>,
    pub Arc<crate::operations::Operations>,
);

#[tauri::command]
pub async fn remove_service(state: State<'_, AppState>, id: String) -> Result<(), String> {
    with_engine(state, move |engine| {
        engine.remove_service(&id).map_err(|e| e.to_string())
    })
    .await
}

pub(crate) async fn with_engine<T: Send + 'static>(
    state: State<'_, AppState>,
    operation: impl FnOnce(&mut Engine) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    let engine = Arc::clone(&state.0);
    let operations = Arc::clone(&state.1);
    tauri::async_runtime::spawn_blocking(move || {
        let _operation = operations.enter()?;
        let mut engine = engine.lock().map_err(|_| "Engine state is unavailable.")?;
        operation(&mut engine)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn snapshot(state: State<'_, AppState>) -> Result<Snapshot, String> {
    with_engine(state, |engine| engine.snapshot().map_err(|e| e.to_string())).await
}
#[tauri::command]
pub async fn add_project(state: State<'_, AppState>, input: ProjectInput) -> Result<(), String> {
    with_engine(state, move |engine| {
        engine
            .add_project(input)
            .map(|_| ())
            .map_err(|e| e.to_string())
    })
    .await
}
#[tauri::command]
pub async fn register_service(
    state: State<'_, AppState>,
    input: ServiceInput,
) -> Result<(), String> {
    with_engine(state, move |engine| {
        engine
            .register_service(input)
            .map(|_| ())
            .map_err(|e| e.to_string())
    })
    .await
}
#[tauri::command]
pub async fn control_service(
    state: State<'_, AppState>,
    id: String,
    action: String,
) -> Result<(), String> {
    with_engine(state, move |engine| {
        match action.as_str() {
            "start" => engine.start_service(&id),
            "stop" => engine.stop_service(&id),
            "restart" => engine.restart_service(&id),
            _ => return Err("Unknown service action.".into()),
        }
        .map_err(|e| e.to_string())
    })
    .await
}
#[tauri::command]
pub async fn set_theme(state: State<'_, AppState>, theme: String) -> Result<(), String> {
    with_engine(state, move |engine| {
        engine.set_theme(&theme).map_err(|e| e.to_string())
    })
    .await
}
#[tauri::command]
pub async fn update_project(
    state: State<'_, AppState>,
    id: String,
    service_id: Option<String>,
    runtime: String,
) -> Result<(), String> {
    with_engine(state, move |engine| {
        engine
            .update_project(&id, service_id, runtime)
            .map_err(|e| e.to_string())
    })
    .await
}
#[tauri::command]
pub async fn edit_project(
    state: State<'_, AppState>,
    id: String,
    name: String,
    path: String,
    domain: String,
) -> Result<(), String> {
    with_engine(state, move |engine| {
        engine
            .edit_project(&id, name, path, domain)
            .map_err(|e| e.to_string())
    })
    .await
}
#[tauri::command]
pub async fn remove_project(state: State<'_, AppState>, id: String) -> Result<(), String> {
    with_engine(state, move |engine| {
        engine.remove_project(&id).map_err(|e| e.to_string())
    })
    .await
}
#[tauri::command]
pub async fn update_service(
    state: State<'_, AppState>,
    id: String,
    input: ServiceInput,
) -> Result<(), String> {
    with_engine(state, move |engine| {
        engine.update_service(&id, input).map_err(|e| e.to_string())
    })
    .await
}
/// Returns one message per service that could not start; stopping reports errors directly.
#[tauri::command]
pub async fn control_all(
    state: State<'_, AppState>,
    action: String,
) -> Result<Vec<String>, String> {
    with_engine(state, move |engine| {
        match action.as_str() {
            "start" => engine.start_all(),
            "stop" => engine.stop_all().map(|_| Vec::new()),
            _ => return Err("Unknown service action.".into()),
        }
        .map_err(|e| e.to_string())
    })
    .await
}
#[tauri::command]
pub async fn set_restore_services(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    with_engine(state, move |engine| {
        engine
            .set_restore_services(enabled)
            .map_err(|e| e.to_string())
    })
    .await
}
#[tauri::command]
pub async fn startup_enabled() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(|| {
        aevrik_engine::autostart::enabled().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn set_startup(enabled: bool) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        aevrik_engine::autostart::set(enabled.then_some(executable.as_path()))
            .map_err(|e| e.to_string())?;
        aevrik_engine::autostart::enabled().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
/// Opens a registered project's folder or terminal, or a service's local URL.
/// Targets come from saved registrations, never from a path sent by the page.
#[tauri::command]
pub async fn open_target(
    state: State<'_, AppState>,
    kind: String,
    id: String,
) -> Result<(), String> {
    with_engine(state, move |engine| {
        let snapshot = engine.snapshot().map_err(|e| e.to_string())?;
        if kind == "service-url" {
            let port = snapshot
                .services
                .iter()
                .find(|s| s.id == id)
                .ok_or("Service no longer exists.")?
                .config
                .port
                .ok_or("This service has no expected port.")?;
            return launch("explorer.exe", &[&format!("http://localhost:{port}")], None);
        }
        let project = snapshot
            .projects
            .iter()
            .find(|p| p.id == id)
            .ok_or("Project no longer exists.")?;
        let path = aevrik_engine::npm::display_path(std::path::Path::new(&project.path));
        if !std::path::Path::new(&path).is_dir() {
            return Err("The project folder is no longer available.".into());
        }
        match kind.as_str() {
            "project-folder" => launch("explorer.exe", &[&path], None),
            "project-terminal" => launch("cmd.exe", &[], Some(&path)),
            _ => Err("Unknown target.".into()),
        }
    })
    .await
}

#[cfg(windows)]
fn launch(program: &str, arguments: &[&str], directory: Option<&str>) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_CONSOLE: u32 = 0x00000010;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let windows = std::env::var_os("SystemRoot").ok_or("Windows directory is unavailable.")?;
    let windows = std::path::PathBuf::from(windows);
    let mut command = std::process::Command::new(if program == "cmd.exe" {
        windows.join("System32").join(program)
    } else {
        windows.join(program)
    });
    command.args(arguments);
    // A terminal needs its own console; Explorer only hands the target to the shell.
    if let Some(directory) = directory {
        command
            .current_dir(directory)
            .creation_flags(CREATE_NEW_CONSOLE);
    } else {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    // Reap the launcher without blocking the UI or waiting for the opened window.
    let _ = std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
#[cfg(not(windows))]
fn launch(_: &str, _: &[&str], _: Option<&str>) -> Result<(), String> {
    Err("Opening folders, terminals and URLs is only supported on Windows.".into())
}
#[tauri::command]
pub async fn clear_logs(state: State<'_, AppState>) -> Result<(), String> {
    with_engine(state, |engine| {
        engine.clear_logs();
        Ok(())
    })
    .await
}
#[tauri::command]
pub async fn pick_path(kind: String) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let picker = rfd::FileDialog::new();
        let path = match kind.as_str() {
            "directory" => picker.pick_folder(),
            "executable" => picker
                .add_filter("Windows executable", &["exe"])
                .pick_file(),
            _ => return Err("Unknown picker kind.".into()),
        };
        Ok(path.map(|path| path.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|e| e.to_string())?
}
