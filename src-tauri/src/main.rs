#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod packages;
mod storage;
mod updates;
use aevrik_engine::Engine;
use commands::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;

fn main() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let Some((storage, directory)) = storage::initialize(app)? else {
                app.handle().exit(0);
                return Ok(());
            };
            app.manage(storage);
            let path = directory.join("aevrik.sqlite");
            app.manage(AppState(Arc::new(Mutex::new(Engine::open(&path)?))));
            app.manage(packages::PackageState(Arc::new(
                aevrik_engine::packages::PackageManager::new(directory.join("runtimes"))?,
            )));
            tauri::WebviewWindowBuilder::from_config(app, &app.config().app.windows[0])?
                .data_directory(
                    std::path::PathBuf::from(storage::display(&directory)).join("webview"),
                )
                .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            storage::storage_info,
            storage::set_data_directory,
            updates::refresh_catalog,
            updates::check_app_update,
            updates::open_release_page,
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
