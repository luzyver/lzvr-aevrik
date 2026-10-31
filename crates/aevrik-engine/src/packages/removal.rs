use super::{PackageError, PackageManager, PackageResult, archive, catalog};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn invalid(message: &str) -> PackageError {
    PackageError::Invalid(message.into())
}

fn refuse_link(path: &Path) -> PackageResult<()> {
    let metadata = fs::symlink_metadata(path)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(invalid(
                "Uninstall refused: the installation contains a junction or symbolic link.",
            ));
        }
    }
    if metadata.file_type().is_symlink() {
        return Err(invalid("Uninstall refused: symbolic link."));
    }
    Ok(())
}

fn inspect_tree(directory: &Path) -> PackageResult<()> {
    refuse_link(directory)?;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        refuse_link(&entry.path())?;
        if entry.file_type()?.is_dir() {
            if entry.path().join(".aevrik-package.json").exists() {
                return Err(invalid(
                    "Uninstall refused: another managed installation is nested in this folder. Remove it first.",
                ));
            }
            inspect_tree(&entry.path())?;
        }
    }
    Ok(())
}

fn key(path: &Path) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_owned());
    canonical
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

impl PackageManager {
    pub fn uninstall(&self, id: &str, protected_paths: &[PathBuf]) -> PackageResult<()> {
        let package = self
            .catalog()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| invalid("Unknown catalog package."))?;
        let _windows_lock = super::defaults::PathChangeLock::acquire()?;
        let _defaults_lock = self.package_lock("runtime-defaults")?;
        let _lock = self.package_lock(id)?;
        let root = self.package_root(id)?;
        let directory = root.join(&package.id).join(&package.version);
        refuse_link(&root)?;
        refuse_link(&root.join(&package.id))?;
        refuse_link(&directory)?;
        let canonical_root = root.canonicalize()?;
        let canonical_dir = directory.canonicalize()?;
        if canonical_dir != canonical_root.join(&package.id).join(&package.version) {
            return Err(invalid(
                "Uninstall refused: installation path escaped its recorded root.",
            ));
        }
        let marker: catalog::Package =
            serde_json::from_slice(&fs::read(directory.join(".aevrik-package.json"))?)?;
        if marker.id != package.id
            || marker.version != package.version
            || marker.sha256 != package.sha256
            || marker.sha512 != package.sha512
            || marker.executable != package.executable
            || archive::installed(&root, &package).is_none()
        {
            return Err(invalid(
                "Uninstall refused: ownership marker does not match this package.",
            ));
        }
        let prefix = key(&canonical_dir) + "\\";
        if protected_paths
            .iter()
            .any(|p| key(p) == key(&canonical_dir) || key(p).starts_with(&prefix))
        {
            return Err(invalid(
                "This runtime is referenced by a service or project. Stop and remove its service registration, or move the project, before uninstalling.",
            ));
        }
        inspect_tree(&directory)?;
        super::defaults::DefaultsSnapshot::read()?.protect(&canonical_dir)?;
        // Only the exact marker-owned version is removed, never its root, sibling versions or external data.
        fs::remove_dir_all(&canonical_dir)?;
        self.forget_location(id)?;
        self.progress(id, "available", 0, None, None);
        Ok(())
    }
}
