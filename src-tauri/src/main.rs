#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod packages;
use aevrik_engine::Engine;
use commands::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;
struct AppDataLock {
    _file: std::fs::File,
}

fn main() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let directory = match std::env::var_os("AEVRIK_DATA_DIR") {
                Some(value) => {
                    let path = std::path::PathBuf::from(value);
                    if !path.is_absolute() {
                        return Err("AEVRIK_DATA_DIR must be an absolute directory.".into());
                    }
                    path
                }
                None => {
                    let preferred = app.path().app_data_dir()?;
                    let legacy = preferred
                        .parent()
                        .ok_or("Application data parent is unavailable.")?
                        .join("dev.aevrik.desktop");
                    aevrik_engine::data_directory(&preferred, &legacy)
                }
            };
            std::fs::create_dir_all(&directory)?;
            let lock = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(directory.join(".aevrik.lock"))?;
            lock.try_lock().map_err(
                |_| "Another Aevrik instance is using this data folder. Close it first.",
            )?;
            app.manage(AppDataLock { _file: lock });
            let path = directory.join("aevrik.sqlite");
            app.manage(AppState(Arc::new(Mutex::new(Engine::open(&path)?))));
            app.manage(packages::PackageState(Arc::new(
                aevrik_engine::packages::PackageManager::new(directory.join("runtimes"))?,
            )));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::add_project,
            commands::register_service,
            commands::control_service,
            commands::set_theme,
            commands::update_project,
            commands::clear_logs,
            commands::pick_path,
            commands::remove_service,
            packages::list_packages,
            packages::install_package,
            packages::uninstall_package,
            packages::package_install_root
        ])
        .build(tauri::generate_context!());
    let app = match app {
        Ok(app) => app,
        Err(error) => {
            rfd::MessageDialog::new()
                .set_title("Aevrik could not start")
                .set_description(format!(
                    "The desktop engine could not initialize.\n\n{error}"
                ))
                .set_level(rfd::MessageLevel::Error)
                .show();
            return;
        }
    };
    app.run(|app, event| {
        if let tauri::RunEvent::Exit = event
            && let Some(state) = app.try_state::<AppState>()
        {
            match state.0.lock() {
                Ok(mut engine) => {
                    if let Err(error) = engine.shutdown() {
                        eprintln!("Shutdown: {error}");
                    }
                }
                Err(error) => eprintln!("Shutdown state: {error}"),
            }
        }
    });
}
