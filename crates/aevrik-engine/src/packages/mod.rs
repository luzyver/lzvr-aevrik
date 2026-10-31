//! Pinned, isolated runtime installation. This manager never starts a service.
mod archive;
mod catalog;
mod catalog_update;
#[cfg(test)]
mod catalog_update_tests;
pub use catalog_update::CatalogUpdate;
mod defaults;
mod download;
pub use defaults::{DefaultPlan, DefaultStatus, Scope};
mod locations;
mod removal;
#[cfg(test)]
mod tests;
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("{0}")]
    Invalid(String),
    #[error("Package I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Download failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Invalid ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Package metadata: {0}")]
    Json(#[from] serde_json::Error),
}
pub(crate) type PackageResult<T> = Result<T, PackageError>;

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    status: String,
    bytes: u64,
    total: Option<u64>,
    error: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageView {
    #[serde(flatten)]
    package: catalog::Package,
    pub status: String,
    bytes: u64,
    total: Option<u64>,
    error: Option<String>,
    path: Option<String>,
    install_directory: Option<String>,
    terminal_default: DefaultStatus,
}
pub struct PackageManager {
    root: PathBuf,
    progress: Mutex<HashMap<String, Progress>>,
}
impl PackageManager {
    pub fn new(root: PathBuf) -> PackageResult<Self> {
        fs::create_dir_all(root.join(".locks"))?;
        Ok(Self {
            root,
            progress: Mutex::new(HashMap::new()),
        })
    }
    pub fn list(&self) -> PackageResult<Vec<PackageView>> {
        let roots = self.package_roots()?;
        let defaults = defaults::DefaultsSnapshot::read()?;
        let progress = self
            .progress
            .lock()
            .map_err(|_| PackageError::Invalid("Installer state is unavailable.".into()))?;
        Ok(self
            .catalog()?
            .into_iter()
            .map(|package| {
                let root = roots.get(&package.id).unwrap_or(&self.root);
                let path = archive::installed(root, &package);
                let current = progress.get(&package.id).cloned().unwrap_or_default();
                PackageView {
                    terminal_default: defaults.status(&package.family, path.as_deref()),
                    install_directory: path.as_ref().map(|_| {
                        root.join(&package.id)
                            .join(&package.version)
                            .to_string_lossy()
                            .into_owned()
                    }),
                    status: if path.is_some() {
                        "installed".into()
                    } else if current.status.is_empty() || current.status == "installed" {
                        "available".into()
                    } else {
                        current.status
                    },
                    bytes: current.bytes,
                    total: current.total,
                    error: current.error,
                    path: path.map(|p| p.to_string_lossy().into_owned()),
                    package,
                }
            })
            .collect())
    }
    fn progress(
        &self,
        id: &str,
        status: &str,
        bytes: u64,
        total: Option<u64>,
        error: Option<String>,
    ) {
        if let Ok(mut map) = self.progress.lock() {
            map.insert(
                id.into(),
                Progress {
                    status: status.into(),
                    bytes,
                    total,
                    error,
                },
            );
        }
    }
    pub fn install(&self, id: &str) -> PackageResult<()> {
        self.install_at(id, None)
    }
    fn package_lock(&self, id: &str) -> PackageResult<fs::File> {
        archive::safe_path(id)?;
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join(".locks").join(format!("{id}.lock")))?;
        lock.try_lock().map_err(|_| {
            PackageError::Invalid(
                "This package is already being installed or removed by another request.".into(),
            )
        })?;
        Ok(lock)
    }
    pub fn install_at(&self, id: &str, destination: Option<&Path>) -> PackageResult<()> {
        let package = self
            .catalog()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| PackageError::Invalid("Unknown catalog package.".into()))?;
        let _lock = self.package_lock(id)?;
        let previous = self.package_root(id)?;
        if archive::installed(&previous, &package).is_some() {
            return Ok(());
        }
        let root = locations::validated_root(
            &destination
                .map(Path::to_path_buf)
                .unwrap_or(self.install_root()?),
        )?;
        self.set_install_root(&root)?;
        self.record_location(id, &root)?;
        self.progress(id, "downloading", 0, None, None);
        let result = (|| {
            let mut file = download::fetch(&package.url, &root, |bytes, total| {
                self.progress(id, "downloading", bytes, total, None)
            })?;
            self.progress(
                id,
                "verifying",
                file.as_file().metadata()?.len(),
                Some(file.as_file().metadata()?.len()),
                None,
            );
            archive::publish_reader(file.as_file_mut(), &package, &root)
        })();
        match &result {
            Ok(()) => self.progress(id, "installed", 0, None, None),
            Err(error) => self.progress(id, "failed", 0, None, Some(error.to_string())),
        }
        result
    }
}
