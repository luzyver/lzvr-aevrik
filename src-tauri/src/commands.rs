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

async fn with_engine<T: Send + 'static>(
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
