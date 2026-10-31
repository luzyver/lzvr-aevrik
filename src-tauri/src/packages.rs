use aevrik_engine::packages::{DefaultPlan, PackageManager, PackageView, Scope};
use std::sync::Arc;
use tauri::State;

pub struct PackageState(
    pub Arc<PackageManager>,
    pub Arc<crate::operations::Operations>,
);

#[tauri::command]
pub async fn preview_runtime_default(
    state: State<'_, PackageState>,
    id: String,
    removing: bool,
) -> Result<DefaultPlan, String> {
    let manager = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || {
        manager
            .preview_default(&id, removing)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn apply_runtime_default(
    state: State<'_, PackageState>,
    id: String,
    removing: bool,
    scope: Scope,
    fingerprint: String,
) -> Result<(), String> {
    let manager = Arc::clone(&state.0);
    let operations = Arc::clone(&state.1);
    tauri::async_runtime::spawn_blocking(move || {
        let _operation = operations.enter()?;
        manager
            .apply_default(&id, removing, scope, &fingerprint)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_packages(state: State<'_, PackageState>) -> Result<Vec<PackageView>, String> {
    let manager = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || manager.list().map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn install_package(
    state: State<'_, PackageState>,
    id: String,
    destination: Option<String>,
) -> Result<(), String> {
    let manager = Arc::clone(&state.0);
    let operations = Arc::clone(&state.1);
    tauri::async_runtime::spawn_blocking(move || {
        let _operation = operations.enter()?;
        manager
            .install_at(&id, destination.as_deref().map(std::path::Path::new))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn package_install_root(state: State<'_, PackageState>) -> Result<String, String> {
    let manager = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || {
        manager
            .install_root()
            .map(|p| p.to_string_lossy().trim_start_matches(r"\\?\").to_owned())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn uninstall_package(
    state: State<'_, PackageState>,
    engine: State<'_, crate::commands::AppState>,
    id: String,
) -> Result<(), String> {
    let manager = Arc::clone(&state.0);
    let operations = Arc::clone(&state.1);
    let engine = Arc::clone(&engine.0);
    tauri::async_runtime::spawn_blocking(move || {
        let _operation = operations.enter()?;
        // Serialize against service registration/start until removal finishes.
        let engine = engine.lock().map_err(|_| "Engine state is unavailable.")?;
        let protected = engine
            .protected_runtime_paths()
            .map_err(|e| e.to_string())?;
        manager
            .uninstall(&id, &protected)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
