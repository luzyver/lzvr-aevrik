use super::{PackageError, PackageManager, PackageResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Locations {
    default_root: Option<PathBuf>,
    #[serde(default)]
    packages: HashMap<String, PathBuf>,
}

impl PackageManager {
    fn locations<T>(
        &self,
        write: bool,
        operation: impl FnOnce(&mut Locations) -> T,
    ) -> PackageResult<T> {
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join(".locks/locations.lock"))?;
        lock.lock()?;
        let path = self.root.join(".locations.json");
        let mut locations = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Locations::default(),
            Err(e) => return Err(e.into()),
        };
        let result = operation(&mut locations);
        if write {
            let mut temporary = tempfile::NamedTempFile::new_in(&self.root)?;
            serde_json::to_writer(temporary.as_file_mut(), &locations)?;
            temporary.as_file().sync_all()?;
            temporary.persist(path).map_err(|e| e.error)?;
        }
        Ok(result)
    }
    pub fn install_root(&self) -> PackageResult<PathBuf> {
        self.locations(false, |index| {
            index.default_root.clone().unwrap_or(self.root.clone())
        })
    }
    pub fn set_install_root(&self, root: &Path) -> PackageResult<()> {
        let root = validated_root(root)?;
        self.locations(true, |index| index.default_root = Some(root))
    }
    pub(super) fn package_root(&self, id: &str) -> PackageResult<PathBuf> {
        self.locations(false, |index| {
            index.packages.get(id).cloned().unwrap_or(self.root.clone())
        })
    }
    pub(super) fn package_roots(&self) -> PackageResult<HashMap<String, PathBuf>> {
        self.locations(false, |index| index.packages.clone())
    }
    pub(super) fn record_location(&self, id: &str, root: &Path) -> PackageResult<()> {
        let root = validated_root(root)?;
        self.locations(true, |index| {
            index.packages.insert(id.into(), root);
        })
    }
    pub(super) fn forget_location(&self, id: &str) -> PackageResult<()> {
        self.locations(true, |index| {
            index.packages.remove(id);
        })
    }
}

pub(super) fn validated_root(root: &Path) -> PackageResult<PathBuf> {
    if !root.is_absolute() {
        return Err(PackageError::Invalid(
            "Choose an absolute installation folder, such as D:\\Aevrik\\runtimes.".into(),
        ));
    }
    fs::create_dir_all(root)?;
    let root = root.canonicalize()?;
    // Check writability before starting a network request; this file deletes on drop.
    let _probe = tempfile::NamedTempFile::new_in(&root)?;
    Ok(root)
}
