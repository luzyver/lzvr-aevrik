#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod packages;
mod startup;
mod storage;
mod updates;
use commands::AppState;
use tauri::Manager;

fn main() {
    let app = tauri::Builder::default()
        .setup(startup::initialize)
        .invoke_handler(tauri::generate_handler![
            startup::setup_info,
            startup::complete_setup,
            startup::exit_setup,
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
