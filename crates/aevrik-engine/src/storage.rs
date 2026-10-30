use std::path::{Path, PathBuf};

/// Preserve paths in existing registrations; never copy or move an active database.
pub fn data_directory(preferred: &Path, legacy: &Path) -> PathBuf {
    if !preferred.join("aevrik.sqlite").exists() && legacy.join("aevrik.sqlite").exists() {
        legacy.to_owned()
    } else {
        preferred.to_owned()
    }
}
