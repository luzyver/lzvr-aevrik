use crate::packages::PackageState;
use aevrik_engine::{packages::CatalogUpdate, updates::AppUpdate};
use std::sync::Arc;
use tauri::State;

const CHANNEL: &str = env!("AEVRIK_CHANNEL");

#[tauri::command]
pub async fn refresh_catalog(state: State<'_, PackageState>) -> Result<CatalogUpdate, String> {
    let manager = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || {
        manager.refresh_catalog(CHANNEL).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn check_app_update() -> Result<AppUpdate, String> {
    tauri::async_runtime::spawn_blocking(|| {
        aevrik_engine::updates::check_app(
            CHANNEL,
            env!("CARGO_PKG_VERSION"),
            env!("AEVRIK_COMMIT"),
            env!("AEVRIK_BUILT_AT").parse().unwrap_or(0),
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn open_release_page(tag: String) -> Result<(), String> {
    let url = aevrik_engine::updates::release_page(&tag).map_err(|e| e.to_string())?;
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let windows = std::env::var_os("SystemRoot").ok_or("Windows directory is unavailable.")?;
        let mut opener =
            std::process::Command::new(std::path::PathBuf::from(windows).join("explorer.exe"))
                .arg(url)
                .creation_flags(0x08000000)
                .spawn()
                .map_err(|e| e.to_string())?;
        // Reap the launcher without blocking the UI or waiting for the browser window.
        let _ = std::thread::spawn(move || {
            let _ = opener.wait();
        });
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = url;
        Err("Opening release pages is only supported on Windows.".into())
    }
}
