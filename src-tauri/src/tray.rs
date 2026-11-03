//! Closing the workspace hides it; only explicit exit shuts down its processes.
use tauri::{
    AppHandle, Manager, Window, WindowEvent,
    menu::{Menu, MenuItem},
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
    let menu = Menu::with_items(app, &[&open, &exit])?;
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
            "exit-aevrik" => app.exit(0),
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
