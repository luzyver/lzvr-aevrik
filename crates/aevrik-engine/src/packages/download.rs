use super::{PackageError, PackageResult};
use std::{
    io::{Read, Write},
    path::Path,
    time::{Duration, Instant},
};
const MAX_DOWNLOAD: u64 = 2 * 1024 * 1024 * 1024;

pub fn fetch(
    url: &str,
    destination: &Path,
    progress: impl FnMut(u64, Option<u64>),
) -> PackageResult<tempfile::NamedTempFile> {
    let client = reqwest::blocking::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(1800))
        .user_agent("Aevrik/0.1.0-beta-1 runtime-installer")
        .build()?;
    let mut response = client.get(url).send()?.error_for_status()?;
    let total = response.content_length();
    if total.is_some_and(|size| size > MAX_DOWNLOAD) {
        return Err(PackageError::Invalid(
            "Download exceeds the 2 GiB limit.".into(),
        ));
    }
    write_stream(&mut response, destination, MAX_DOWNLOAD, progress, total)
}

pub(super) fn write_stream(
    reader: &mut impl Read,
    destination: &Path,
    limit: u64,
    mut progress: impl FnMut(u64, Option<u64>),
    total: Option<u64>,
) -> PackageResult<tempfile::NamedTempFile> {
    let mut file = tempfile::Builder::new()
        .prefix(".download-")
        .tempfile_in(destination)?;
    let mut buffer = [0_u8; 64 * 1024];
    let mut bytes = 0_u64;
    let mut last = Instant::now();
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        if count as u64 > limit.saturating_sub(bytes) {
            return Err(PackageError::Invalid(
                "Download exceeds the archive size limit.".into(),
            ));
        }
        file.write_all(&buffer[..count])?;
        bytes += count as u64;
        if last.elapsed() >= Duration::from_millis(150) {
            progress(bytes, total);
            last = Instant::now();
        }
    }
    file.flush()?;
    progress(bytes, total);
    Ok(file)
}
