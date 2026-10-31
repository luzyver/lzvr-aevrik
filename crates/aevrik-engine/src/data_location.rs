//! Bootstrap preferences live outside the selected workspace; switching never migrates files.
use crate::{Result, error::invalid};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Location {
    version: u8,
    directory: PathBuf,
}

fn absolute(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err(invalid(
            "Choose an absolute data-folder path, such as D:\\AevrikData.",
        ));
    }
    #[cfg(windows)]
    if !matches!(path.components().next(), Some(std::path::Component::Prefix(prefix))
        if matches!(prefix.kind(), std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)))
    {
        return Err(invalid(
            "Choose a folder on a local drive, such as D:\\AevrikData.",
        ));
    }
    Ok(())
}

pub fn read_location(config: &Path) -> Result<Option<PathBuf>> {
    let bytes = match fs::read(config) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let location: Location = serde_json::from_slice(&bytes)?;
    absolute(&location.directory)?;
    if location.version != 1 {
        return Err(invalid(
            "This data-folder preference was written by an unsupported version.",
        ));
    }
    if !location.directory.is_dir() {
        return Err(invalid(format!(
            "Saved data folder is unavailable: {}. Reconnect its drive, or use AEVRIK_DATA_DIR to open another folder. Preference file: {}",
            location.directory.display(),
            config.display()
        )));
    }
    Ok(Some(location.directory))
}

pub fn resolve_location(
    preferred: &Path,
    legacy: &Path,
    config: &Path,
    override_path: Option<&Path>,
) -> Result<Option<PathBuf>> {
    if let Some(path) = override_path {
        absolute(path)?;
        return Ok(Some(path.to_owned()));
    }
    if let Some(path) = read_location(config)? {
        return Ok(Some(path));
    }
    let existing = crate::data_directory(preferred, legacy);
    Ok(existing.join("aevrik.sqlite").is_file().then_some(existing))
}

pub fn save_location(config: &Path, directory: &Path) -> Result<()> {
    absolute(directory)?;
    if !directory.is_dir() {
        return Err(invalid("The selected data folder is unavailable."));
    }
    let parent = config
        .parent()
        .ok_or_else(|| invalid("Preference directory is unavailable."))?;
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    let location = Location {
        version: 1,
        directory: directory.to_owned(),
    };
    temporary.write_all(&serde_json::to_vec_pretty(&location)?)?;
    temporary.as_file().sync_all()?;
    temporary.persist(config).map_err(|error| error.error)?;
    Ok(())
}

pub fn validate_destination(directory: &Path, application: &Path) -> Result<PathBuf> {
    absolute(directory)?;
    let app = fs::canonicalize(application)?;
    // Check before creating directories, and again after resolving junctions.
    if directory.starts_with(application) || application.starts_with(directory) {
        return Err(invalid(
            "Keep the data folder separate from the application installation folder.",
        ));
    }
    fs::create_dir_all(directory)?;
    let directory = fs::canonicalize(directory)?;
    if directory.starts_with(&app) || app.starts_with(&directory) {
        return Err(invalid(
            "Keep the data folder separate from the application installation folder.",
        ));
    }
    if !directory.join("aevrik.sqlite").is_file() {
        for entry in fs::read_dir(&directory)? {
            let name = entry?.file_name();
            if ![".aevrik.lock", ".location.lock", "data-location.json"]
                .iter()
                .any(|allowed| name == *allowed)
            {
                return Err(invalid(
                    "Choose an empty folder or an existing Aevrik data folder. Other files will not be overwritten.",
                ));
            }
        }
    }
    // Probe write access without replacing an existing file.
    let _probe = tempfile::NamedTempFile::new_in(&directory)?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join(".aevrik.lock"))?;
    lock.try_lock().map_err(|_| {
        invalid("Another Aevrik instance is using this data folder. Close it first.")
    })?;
    Ok(directory)
}
