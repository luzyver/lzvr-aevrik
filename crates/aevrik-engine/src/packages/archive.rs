use super::{PackageError, PackageResult, catalog::Package};
use sha2::{Digest, Sha256, Sha512};
use std::{
    collections::HashSet,
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

const MAX_EXPANDED: u64 = 8 * 1024 * 1024 * 1024;
#[cfg(test)]
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn safe_path(name: &str) -> PackageResult<PathBuf> {
    let trimmed = name.trim_end_matches('/');
    if trimmed.is_empty() || name.starts_with('/') || name.contains(['\\', ':']) {
        return Err(PackageError::Invalid("Unsafe archive path.".into()));
    }
    for part in trimmed.split('/') {
        let base = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        let device = matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || ["COM", "LPT"].iter().any(|prefix| {
                base.strip_prefix(prefix).is_some_and(|suffix| {
                    matches!(
                        suffix,
                        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                    )
                })
            });
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.ends_with(['.', ' '])
            || device
            || part
                .chars()
                .any(|c| c.is_control() || "<>\"|?*".contains(c))
        {
            return Err(PackageError::Invalid("Unsafe archive path.".into()));
        }
    }
    Ok(PathBuf::from(trimmed))
}

#[cfg(test)]
pub fn unpack(bytes: &[u8], package: &Package, destination: &Path) -> PackageResult<()> {
    unpack_reader(&mut std::io::Cursor::new(bytes), package, destination)
}
fn unpack_reader<R: Read + Seek>(
    reader: &mut R,
    package: &Package,
    destination: &Path,
) -> PackageResult<()> {
    reader.seek(SeekFrom::Start(0))?;
    let mut hash = Sha256::new();
    let mut hash512 = Sha512::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        if package.sha512.is_empty() {
            hash.update(&buffer[..count]);
        } else {
            hash512.update(&buffer[..count]);
        }
    }
    let matches = if package.sha512.is_empty() {
        format!("{:x}", hash.finalize()) == package.sha256
    } else {
        format!("{:x}", hash512.finalize()) == package.sha512
    };
    if !matches {
        return Err(PackageError::Invalid(
            "Checksum mismatch. No package was installed; retry the download.".into(),
        ));
    }
    reader.seek(SeekFrom::Start(0))?;
    let mut archive = zip::ZipArchive::new(reader)?;
    if archive.len() > 40_000 {
        return Err(PackageError::Invalid("Too many archive entries.".into()));
    }
    let mut total = 0u64;
    let mut names = HashSet::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let relative = safe_path(entry.name())?;
        if !names.insert(relative.to_string_lossy().to_lowercase()) {
            return Err(PackageError::Invalid("Duplicate archive path.".into()));
        }
        let kind = entry.unix_mode().unwrap_or(0) & 0o170000;
        if kind != 0 && kind != 0o100000 && kind != 0o040000 {
            return Err(PackageError::Invalid(
                "Archive links and special files are not supported.".into(),
            ));
        }
        let target = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(target)?;
            continue;
        }
        if entry.size() > MAX_EXPANDED - total {
            return Err(PackageError::Invalid(
                "Archive exceeds the extraction limit.".into(),
            ));
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)?;
        let count = std::io::copy(
            &mut (&mut entry).take(MAX_EXPANDED - total + 1),
            &mut output,
        )?;
        total += count;
        if total > MAX_EXPANDED {
            return Err(PackageError::Invalid(
                "Archive exceeds the extraction limit.".into(),
            ));
        }
    }
    if !destination.join(safe_path(&package.executable)?).is_file() {
        return Err(PackageError::Invalid(
            "The archive is missing its expected executable.".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
pub fn publish(bytes: &[u8], package: &Package, root: &Path) -> PackageResult<()> {
    publish_reader(&mut std::io::Cursor::new(bytes), package, root)
}
pub fn publish_reader<R: Read + Seek>(
    reader: &mut R,
    package: &Package,
    root: &Path,
) -> PackageResult<()> {
    let final_path = root.join(&package.id).join(&package.version);
    if final_path.exists() {
        return Err(PackageError::Invalid(
            "A runtime directory already exists. It has been left untouched.".into(),
        ));
    }
    let staging = tempfile::Builder::new()
        .prefix(".install-")
        .tempdir_in(root)?;
    unpack_reader(reader, package, staging.path())?;
    fs::write(
        staging.path().join(".aevrik-package.json"),
        serde_json::to_vec(package)?,
    )?;
    fs::create_dir_all(final_path.parent().unwrap())?;
    // The caller holds the cross-process install lock. Rename publishes only a complete tree.
    fs::rename(staging.path(), &final_path)?;
    Ok(())
}

pub fn installed(root: &Path, package: &Package) -> Option<PathBuf> {
    let directory = root.join(&package.id).join(&package.version);
    let marker: Package =
        serde_json::from_slice(&fs::read(directory.join(".aevrik-package.json")).ok()?).ok()?;
    let executable = directory.join(&package.executable);
    (marker.id == package.id
        && marker.version == package.version
        && marker.sha256 == package.sha256
        && marker.sha512 == package.sha512
        && marker.executable == package.executable
        && executable.is_file())
    .then_some(executable)
}
