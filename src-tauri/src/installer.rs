//! Headless Setup entry point. Never opens a WebView or initializes a database.
use std::path::Path;
use tauri::Manager;

pub fn configure(app: &tauri::App, selected: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let preferred = app.path().app_data_dir()?;
    let legacy = preferred
        .parent()
        .ok_or("Application data parent is unavailable.")?
        .join("dev.aevrik.desktop");
    let config = app.path().app_config_dir()?.join("data-location.json");
    let executable = std::env::current_exe()?;
    let application = executable
        .parent()
        .ok_or("Application folder is unavailable.")?;
    aevrik_engine::data_location::configure_installation(
        &preferred,
        &legacy,
        &config,
        application,
        selected,
    )?;
    Ok(())
}

pub fn run(selected: &Path, silent: bool) -> i32 {
    let result = tauri::Builder::default()
        .build(tauri::generate_context!())
        .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
        .and_then(|app| configure(&app, selected));
    finish(result, silent, |error| {
        rfd::MessageDialog::new().set_title("Aevrik data folder")
                .set_description(format!("The data folder could not be configured.\n\n{error}\n\nYour existing data has not been moved or deleted."))
                .set_level(rfd::MessageLevel::Error).show();
    })
}

fn finish(
    result: Result<(), Box<dyn std::error::Error>>,
    silent: bool,
    show_error: impl FnOnce(String),
) -> i32 {
    match result {
        Ok(()) => 0,
        Err(error) => {
            if !silent {
                show_error(error.to_string());
            }
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_configuration_failure_returns_nonzero_without_showing_a_dialog() {
        let result = Err("The selected data folder is unavailable.".into());
        assert_eq!(
            finish(result, true, |_| panic!(
                "Silent Setup must not open a dialog"
            )),
            1
        );
    }

    #[test]
    fn interactive_failure_reports_the_error_and_success_needs_no_dialog() {
        let mut message = String::new();
        assert_eq!(
            finish(Err("Close Aevrik first.".into()), false, |error| message =
                error),
            1
        );
        assert!(message.contains("Close Aevrik first."));
        for silent in [false, true] {
            assert_eq!(
                finish(Ok(()), silent, |_| panic!(
                    "Successful Setup must not open an error dialog"
                )),
                0
            );
        }
    }
}
