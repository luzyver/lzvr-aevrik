use super::{BuildInfo, REPO, check_release, client, invalid};
use crate::packages::PackageResult;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Deserialize)]
pub(super) struct Artifact {
    pub name: String,
    pub bytes: u64,
    pub sha256: String,
}

pub(super) fn installer_artifact<'a>(
    channel: &str,
    info: &'a BuildInfo,
) -> PackageResult<&'a Artifact> {
    let label = match channel {
        "dev" | "beta" => channel,
        "stable" => &info.version,
        _ => return Err(invalid("Unsupported update channel.")),
    };
    if label.is_empty()
        || !label
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
    {
        return Err(invalid("Invalid release version."));
    }
    let name = format!("Aevrik-{label}-windows-x64-setup.exe");
    let mut matches = info
        .artifacts
        .iter()
        .filter(|artifact| artifact.name == name);
    let artifact = matches.next().ok_or_else(|| {
        invalid(
            "This release has no Windows Setup metadata. Open the release page or try again later.",
        )
    })?;
    if matches.next().is_some()
        || artifact.bytes == 0
        || artifact.bytes > 512 * 1024 * 1024
        || artifact.sha256.len() != 64
        || !artifact.sha256.bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err(invalid(
            "Invalid Windows Setup metadata. Try again after publication finishes.",
        ));
    }
    Ok(artifact)
}

pub(super) fn copy_verified(
    mut input: impl Read,
    mut output: impl Write,
    artifact: &Artifact,
) -> PackageResult<()> {
    let mut digest = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        if total > artifact.bytes {
            return Err(invalid("Installer size mismatch. Try again."));
        }
        digest.update(&buffer[..count]);
        output.write_all(&buffer[..count])?;
    }
    if total != artifact.bytes {
        return Err(invalid("Installer download is incomplete. Try again."));
    }
    if !format!("{:x}", digest.finalize()).eq_ignore_ascii_case(&artifact.sha256) {
        return Err(invalid(
            "Installer checksum mismatch. Try again after publication finishes.",
        ));
    }
    output.flush()?;
    Ok(())
}

/// The temporary directory is removed on failure; only a verified file can be launched.
pub struct DownloadedInstaller {
    directory: tempfile::TempDir,
    path: PathBuf,
}

impl DownloadedInstaller {
    pub fn launch(self, application: &Path) -> PackageResult<()> {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let destination = application
                .to_str()
                .ok_or_else(|| invalid("Application path is not valid Unicode."))?
                .trim_start_matches(r"\\?\");
            if !application.is_absolute() || destination.contains(['"', '\r', '\n']) {
                return Err(invalid("Invalid application installation path."));
            }
            // NSIS /D must be the last argument and unquoted, even with spaces.
            let mut process = std::process::Command::new(&self.path)
                .args(["/UPDATE", "/P", "/R"])
                .raw_arg(format!("/D={destination}"))
                .spawn()?;
            // Setup must remain available after this process exits. Windows temp cleanup
            // can reclaim it later; never delete another updater's working directory.
            let _retained = self.directory.keep();
            let _ = std::thread::spawn(move || {
                let _ = process.wait();
            });
            Ok(())
        }
        #[cfg(not(windows))]
        {
            let _ = (self.directory, self.path, application);
            Err(invalid(
                "Application updates are only supported on Windows.",
            ))
        }
    }
}

pub fn download_app_update(
    channel: &str,
    current: &str,
    commit: &str,
    built_at: u64,
    expected_commit: &str,
) -> PackageResult<DownloadedInstaller> {
    let (update, info) = check_release(channel, current, commit, built_at)?;
    if update.status != "available" || update.latest_commit != expected_commit {
        return Err(invalid(
            "The published build changed or is no longer newer. Check for updates again.",
        ));
    }
    let artifact = installer_artifact(channel, &info)?;
    let url = format!("{REPO}/releases/download/{}/{}", update.tag, artifact.name);
    let response = client(Duration::from_secs(600))?
        .get(url)
        .send()?
        .error_for_status()?;
    if response
        .content_length()
        .is_some_and(|bytes| bytes != artifact.bytes)
    {
        return Err(invalid(
            "Installer size mismatch. Try again after publication finishes.",
        ));
    }
    let directory = tempfile::Builder::new()
        .prefix("aevrik-update-")
        .tempdir()?;
    let path = directory.path().join("setup.exe");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    copy_verified(response, &mut file, artifact)?;
    file.sync_all()?;
    drop(file);
    Ok(DownloadedInstaller { directory, path })
}
