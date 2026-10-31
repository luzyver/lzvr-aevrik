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
    // This handle stays open until the process exits.
    _data_lock: fs::File,
}

impl StorageState {
    pub fn current_directory(&self) -> Result<PathBuf, String> {
        self.info
            .lock()
            .map(|info| PathBuf::from(&info.current))
            .map_err(|_| "Storage settings are unavailable.".into())
    }
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

// Preference locking is retained while setup is open and after activation.
pub struct StorageBootstrap {
    pub preferred: PathBuf,
    pub application: PathBuf,
    pub config: PathBuf,
    overridden: bool,
    _preference_lock: Option<fs::File>,
}

impl StorageBootstrap {
    pub fn activate(
        &self,
        directory: &Path,
    ) -> Result<(StorageState, PathBuf), Box<dyn std::error::Error>> {
        let data_lock = lock(directory, ".aevrik.lock")?;
        let directory = fs::canonicalize(directory)?;
        let state = StorageState {
            info: Mutex::new(StorageInfo {
                current: display(&directory),
                next: None,
                application: display(&self.application),
                overridden: self.overridden,
            }),
            config: self.config.clone(),
            _data_lock: data_lock,
        };
        Ok((state, directory))
    }

    pub fn remember(&self, directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !self.overridden {
            save_location(&self.config, directory)?;
        }
        Ok(())
    }
}

pub fn prepare(
    app: &tauri::App,
) -> Result<(StorageBootstrap, Option<PathBuf>), Box<dyn std::error::Error>> {
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
    let directory = resolve_location(&preferred, &legacy, &config, override_path.as_deref())?;
    Ok((
        StorageBootstrap {
            preferred,
            application,
            config,
            overridden: override_path.is_some(),
            _preference_lock: preference_lock,
        },
        directory,
    ))
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
