use aevrik_engine::data_location::{resolve_location, save_location, validate_destination};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{Manager, State};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    current: String,
    next: Option<String>,
    application: String,
    overridden: bool,
}

pub struct StorageState {
    info: Mutex<StorageInfo>,
    config: PathBuf,
    // Both handles stay open until the process exits.
    _preference_lock: Option<fs::File>,
    _data_lock: fs::File,
}

pub(crate) fn display(path: &Path) -> String {
    path.to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_owned()
}

fn lock(directory: &Path, name: &str) -> Result<fs::File, Box<dyn std::error::Error>> {
    fs::create_dir_all(directory)?;
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join(name))?;
    file.try_lock().map_err(
        |_| "Another Aevrik instance is using these settings or this data folder. Close it first.",
    )?;
    Ok(file)
}

fn choose_first_folder(preferred: &Path, application: &Path) -> Option<PathBuf> {
    loop {
        let choice = rfd::MessageDialog::new()
            .set_title("Choose where Aevrik stores your data")
            .set_description(format!(
                "Data contains your settings, service registrations and default runtime downloads. It is separate from the application.\n\nDefault: {}\n\nYes: choose a folder (for example D:\\AevrikData).\nNo: use the default folder.\nCancel: close Aevrik.", display(preferred)))
            .set_buttons(rfd::MessageButtons::YesNoCancel).show();
        let directory = match choice {
            rfd::MessageDialogResult::Yes => {
                let Some(path) = rfd::FileDialog::new()
                    .set_title("Select an empty folder or an existing Aevrik data folder")
                    .pick_folder()
                else {
                    continue;
                };
                path
            }
            rfd::MessageDialogResult::No => preferred.to_owned(),
            _ => return None,
        };
        match validate_destination(&directory, application) {
            Ok(path) => return Some(path),
            Err(error) => {
                rfd::MessageDialog::new()
                    .set_title("Choose another data folder")
                    .set_description(error.to_string())
                    .set_level(rfd::MessageLevel::Error)
                    .show();
            }
        }
    }
}

pub fn initialize(
    app: &tauri::App,
) -> Result<Option<(StorageState, PathBuf)>, Box<dyn std::error::Error>> {
    let preferred = app.path().app_data_dir()?;
    let legacy = preferred
        .parent()
        .ok_or("Application data parent is unavailable.")?
        .join("dev.aevrik.desktop");
    let config_dir = app.path().app_config_dir()?;
    let config = config_dir.join("data-location.json");
    let application = std::env::current_exe()?
        .parent()
        .ok_or("Application folder is unavailable.")?
        .to_owned();
    let override_path = std::env::var_os("AEVRIK_DATA_DIR").map(PathBuf::from);
    let preference_lock = if override_path.is_none() {
        Some(lock(&config_dir, ".location.lock")?)
    } else {
        None
    };
    let directory = match resolve_location(&preferred, &legacy, &config, override_path.as_deref())?
    {
        Some(path) => path,
        None => match choose_first_folder(&preferred, &application) {
            Some(path) => path,
            None => return Ok(None),
        },
    };
    let data_lock = lock(&directory, ".aevrik.lock")?;
    let directory = fs::canonicalize(directory)?;
    if override_path.is_none() {
        save_location(&config, &directory)?;
    }
    let state = StorageState {
        info: Mutex::new(StorageInfo {
            current: display(&directory),
            next: None,
            application: display(&application),
            overridden: override_path.is_some(),
        }),
        config,
        _preference_lock: preference_lock,
        _data_lock: data_lock,
    };
    Ok(Some((state, directory)))
}

#[tauri::command]
pub fn storage_info(state: State<'_, StorageState>) -> Result<StorageInfo, String> {
    state
        .info
        .lock()
        .map(|info| info.clone())
        .map_err(|_| "Storage settings are unavailable.".into())
}

#[tauri::command]
pub fn set_data_directory(
    state: State<'_, StorageState>,
    directory: String,
) -> Result<StorageInfo, String> {
    let mut info = state
        .info
        .lock()
        .map_err(|_| "Storage settings are unavailable.")?;
    if info.overridden {
        return Err("AEVRIK_DATA_DIR controls this session. Remove that override before changing the saved folder.".into());
    }
    let candidate = PathBuf::from(directory);
    let current = fs::canonicalize(&info.current).map_err(|e| e.to_string())?;
    let selected = if candidate.is_absolute()
        && fs::canonicalize(&candidate).ok().as_ref() == Some(&current)
    {
        current.clone()
    } else {
        validate_destination(&candidate, Path::new(&info.application)).map_err(|e| e.to_string())?
    };
    save_location(&state.config, &selected).map_err(|e| e.to_string())?;
    info.next = (selected != current).then(|| display(&selected));
    Ok(info.clone())
}
