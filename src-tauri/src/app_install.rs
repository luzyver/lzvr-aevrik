use crate::{commands::AppState, storage};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

#[derive(Default)]
pub struct InstallState(Mutex<()>);

#[tauri::command]
pub async fn install_app_update(
    app: AppHandle,
    engine: State<'_, AppState>,
    commit: String,
) -> Result<(), String> {
    let engine_state = Arc::clone(&engine.0);
    let operations = Arc::clone(&engine.1);
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<InstallState>();
        let _install = state.0.try_lock().map_err(|_| "An application update is already in progress.")?;
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let application = executable.parent().ok_or("Application folder is unavailable.")?;
        if !application.join("uninstall.exe").is_file() {
            return Err("This is a portable copy. Install Windows Setup from the release page first; future updates can then be installed here.".into());
        }
        // Fail before downloading if work is active, then check again at handoff.
        let ensure_idle = || -> Result<(), String> {
            let mut engine = engine_state.lock().map_err(|_| "Engine state is unavailable.")?;
            if engine.snapshot().map_err(|e| e.to_string())?.services.iter().any(|service| service.pid.is_some()) {
                return Err("Stop running services before installing the application update.".into());
            }
            Ok(())
        };
        {
            let _exclusive = operations.0.try_write().map_err(|_| "Wait for the current operation to finish, then try again.")?;
            ensure_idle()?;
        }
        let installer = aevrik_engine::updates::download_app_update(
            env!("AEVRIK_CHANNEL"), env!("CARGO_PKG_VERSION"), env!("AEVRIK_COMMIT"),
            env!("AEVRIK_BUILT_AT").parse().unwrap_or(0), &commit,
        ).map_err(|e| e.to_string())?;
        let mut closing = operations.0.try_write().map_err(|_| "An operation started during download. Wait for it to finish, then try again.")?;
        ensure_idle()?;
        // NSIS receives the exact installed application folder; data stays in its workspace.
        installer.launch(std::path::Path::new(&storage::display(application))).map_err(|e| e.to_string())?;
        *closing = true;
        app.exit(0);
        Ok(())
    }).await.map_err(|e| e.to_string())?
}
