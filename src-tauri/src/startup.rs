//! First-run setup owns no workspace until the user confirms a data folder.
use crate::{
    commands::AppState,
    packages::PackageState,
    storage::{self, StorageBootstrap, StorageState},
};
use aevrik_engine::{Engine, data_location::validate_destination, packages::PackageManager};
use serde::Serialize;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub struct StartupState {
    bootstrap: StorageBootstrap,
    completed: Mutex<bool>,
}

#[derive(Serialize)]
pub struct SetupInfo {
    preferred: String,
    application: String,
}

fn open_workspace(
    app: &AppHandle,
    bootstrap: &StorageBootstrap,
    directory: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if app.try_state::<StorageState>().is_none() {
        let (storage, directory) = bootstrap.activate(directory)?;
        let engine = Engine::open(&directory.join("aevrik.sqlite"))?;
        let packages = PackageManager::new(directory.join("runtimes"))?;
        // Persist only after the chosen workspace can actually be opened.
        bootstrap.remember(&directory)?;
        app.manage(storage);
        let operations = Arc::new(crate::operations::Operations::default());
        app.manage(AppState(
            Arc::new(Mutex::new(engine)),
            Arc::clone(&operations),
        ));
        app.manage(PackageState(Arc::new(packages), operations));
    }
    // Retain initialized state and its lock if WebView creation fails. A retry
    // recreates the window, never removes state referenced by IPC handlers.
    WebviewWindowBuilder::from_config(app, &app.config().app.windows[0])?
        .data_directory(std::path::PathBuf::from(storage::display(directory)).join("webview"))
        .build()?;
    Ok(())
}

pub fn initialize(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let (bootstrap, directory) = storage::prepare(app)?;
    if let Some(directory) = &directory {
        open_workspace(app.handle(), &bootstrap, directory)?;
    }
    app.manage(StartupState {
        bootstrap,
        completed: Mutex::new(directory.is_some()),
    });
    if directory.is_none() {
        WebviewWindowBuilder::new(app, "setup", WebviewUrl::App("setup.html".into()))
            .title("Aevrik — Choose your data folder")
            .inner_size(680.0, 560.0)
            .min_inner_size(520.0, 520.0)
            .incognito(true)
            // Never populate the still-unselected default workspace with WebView files.
            .data_directory(app.path().app_cache_dir()?.join("setup-webview"))
            .build()?;
    }
    Ok(())
}

fn require_setup(window: &WebviewWindow) -> Result<(), String> {
    if window.label() != "setup" {
        return Err("This action is only available during first-launch setup.".into());
    }
    Ok(())
}

#[tauri::command]
pub fn setup_info(
    window: WebviewWindow,
    state: State<'_, StartupState>,
) -> Result<SetupInfo, String> {
    require_setup(&window)?;
    Ok(SetupInfo {
        preferred: storage::display(&state.bootstrap.preferred),
        application: storage::display(&state.bootstrap.application),
    })
}

#[tauri::command]
pub async fn complete_setup(
    app: AppHandle,
    window: WebviewWindow,
    directory: String,
) -> Result<(), String> {
    require_setup(&window)?;
    // WebView creation must run outside a synchronous Windows IPC handler.
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<StartupState>();
        let mut completed = state
            .completed
            .lock()
            .map_err(|_| "Setup is unavailable. Close and reopen Aevrik.")?;
        if !*completed {
            let directory = if let Some(storage) = app.try_state::<StorageState>() {
                let current = storage.current_directory()?;
                if std::fs::canonicalize(&directory).map_err(|e| e.to_string())?
                    != std::fs::canonicalize(&current).map_err(|e| e.to_string())? {
                    return Err(format!("Your workspace is already initialized at {}. Choose that folder to retry opening it, or exit and reopen Aevrik.", storage::display(&current)));
                }
                current
            } else {
                validate_destination(Path::new(&directory), &state.bootstrap.application)
                    .map_err(|e| e.to_string())?
            };
            open_workspace(&app, &state.bootstrap, &directory).map_err(|e| e.to_string())?;
            *completed = true;
        }
        // A failed close can be retried without opening a second workspace.
        window.close().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn exit_setup(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    require_setup(&window)?;
    app.exit(0);
    Ok(())
}
