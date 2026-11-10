//! Closing the workspace hides it; only explicit exit shuts down its processes.
use tauri::{
    AppHandle, Manager, Window, WindowEvent,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

const ID: &str = "aevrik-background";

fn restore(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    }
    Ok(())
}

/// Runs off the menu thread; failures land in the Output log the workspace shows.
fn control_all(app: &AppHandle, start: bool) {
    let Some(state) = app.try_state::<crate::commands::AppState>() else {
        return;
    };
    let (engine, operations) = (state.0.clone(), state.1.clone());
    std::thread::spawn(move || {
        let Ok(_operation) = operations.enter() else {
            return;
        };
        let Ok(mut engine) = engine.lock() else {
            return;
        };
        let result = if start {
            engine.start_all().map(|_| ())
        } else {
            engine.stop_all()
        };
        if let Err(error) = result {
            eprintln!("Tray service control: {error}");
        }
    });
}

/// Exit from the tray stops services, so it goes through the workspace's
/// confirmation while any are alive. It exits directly only when the engine
/// reports that nothing is running; any failure cancels the exit.
fn request_exit(app: &AppHandle) {
    match confirm_exit(app) {
        Ok(true) => app.exit(0),
        Ok(false) => {}
        Err(error) => {
            eprintln!("Tray exit: {error}");
            // Off the menu thread, so the tray stays responsive behind the message.
            std::thread::spawn(move || {
                rfd::MessageDialog::new()
                    .set_title("Aevrik did not exit")
                    .set_description(format!(
                        "Services are still running and the confirmation could not be shown.\n\n{error}\n\nChoose Stop all services in the tray menu, then exit again."
                    ))
                    .set_level(rfd::MessageLevel::Warning)
                    .show();
            });
        }
    }
}

/// `Ok(true)` when nothing is running; `Ok(false)` once the workspace was asked to confirm.
fn confirm_exit(app: &AppHandle) -> Result<bool, String> {
    // Before the workspace exists there is no engine and nothing to stop.
    let Some(state) = app.try_state::<crate::commands::AppState>() else {
        return Ok(true);
    };
    let running = state
        .0
        .lock()
        .map_err(|error| error.to_string())?
        .running_count()
        .map_err(|error| error.to_string())?;
    if running == 0 {
        return Ok(true);
    }
    let window = app
        .get_webview_window("main")
        .ok_or("The Aevrik window is not available.")?;
    restore(app).map_err(|error| error.to_string())?;
    window
        .eval("window.dispatchEvent(new Event('aevrik-exit-requested'))")
        .map_err(|error| error.to_string())?;
    Ok(false)
}

pub fn initialize(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if app.tray_by_id(ID).is_some() {
        return Ok(());
    }
    let open = MenuItem::with_id(app, "open-workspace", "Open Aevrik", true, None::<&str>)?;
    let exit = MenuItem::with_id(
        app,
        "exit-aevrik",
        "Exit and stop services",
        true,
        None::<&str>,
    )?;
    let start = MenuItem::with_id(app, "start-all", "Start all services", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop-all", "Stop all services", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &start, &stop, &separator, &exit])?;
    let icon = app
        .default_window_icon()
        .ok_or("Aevrik's tray icon is missing")?;
    TrayIconBuilder::with_id(ID)
        .icon(icon.clone())
        .tooltip("Aevrik — services keep running when the window is closed")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open-workspace" => {
                if let Err(error) = restore(app) {
                    eprintln!("Restore window: {error}");
                }
            }
            "start-all" => control_all(app, true),
            "stop-all" => control_all(app, false),
            "exit-aevrik" => request_exit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) && let Err(error) = restore(tray.app_handle())
            {
                eprintln!("Restore window: {error}");
            }
        })
        .build(app)?;
    Ok(())
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if window.label() == "main"
        && window.app_handle().tray_by_id(ID).is_some()
        && let WindowEvent::CloseRequested { api, .. } = event
    {
        api.prevent_close();
        if let Err(error) = window.hide() {
            eprintln!("Hide window: {error}");
        }
    }
}

#[tauri::command]
pub fn exit_app(app: AppHandle, window: Window) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Exit is only available from the workspace.".into());
    }
    app.exit(0);
    Ok(())
}
