//! Read-only release checks. Download URLs are fixed to the project repository.
use crate::packages::{PackageError, PackageResult};
use serde::{Deserialize, Serialize};
use std::{io::Read, time::Duration};

const REPO: &str = "https://github.com/luzyver/lzvr-aevrik";
const API: &str = "https://api.github.com/repos/luzyver/lzvr-aevrik/releases";

fn invalid(message: &str) -> PackageError {
    PackageError::Invalid(message.into())
}

pub(crate) fn release_download_base(channel: &str) -> PackageResult<String> {
    match channel {
        "dev" | "beta" => Ok(format!("{REPO}/releases/download/{channel}")),
        "stable" => Ok(format!("{REPO}/releases/latest/download")),
        _ => Err(invalid(
            "This local build has no release channel. Use an official dev, beta or stable build.",
        )),
    }
}

pub fn release_page(tag: &str) -> PackageResult<String> {
    let rolling = matches!(tag, "dev" | "beta");
    let versioned = tag.starts_with('v')
        && tag.len() > 1
        && tag.len() <= 80
        && tag[1..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'));
    if !(rolling || versioned) {
        return Err(invalid("Invalid release tag."));
    }
    Ok(format!("{REPO}/releases/tag/{tag}"))
}

pub(crate) fn fetch_json(url: &str, limit: usize) -> PackageResult<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Aevrik-Update-Check")
        .https_only(true)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 {
                return attempt.error("Too many redirects");
            }
            match attempt.url().host_str() {
                Some(
                    "github.com"
                    | "api.github.com"
                    | "release-assets.githubusercontent.com"
                    | "objects.githubusercontent.com",
                ) => attempt.follow(),
                _ => attempt.error("Unexpected update server redirect"),
            }
        }))
        .build()?;
    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(invalid(
            "No published update information is available for this channel yet. Keep using this build and try again after the next release.",
        ));
    }
    let response = response.error_for_status()?;
    if response
        .content_length()
        .is_some_and(|size| size > limit as u64)
    {
        return Err(invalid("Update response is too large."));
    }
    let mut bytes = Vec::new();
    response.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(invalid("Update response is too large."));
    }
    Ok(bytes)
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildInfo {
    version: String,
    channel: String,
    commit: String,
    built_at: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdate {
    pub channel: String,
    pub current_version: String,
    pub latest_version: String,
    pub current_commit: String,
    pub latest_commit: String,
    pub tag: String,
    pub status: String,
    pub message: String,
}

fn version(value: &str) -> Option<[u64; 3]> {
    let parts = value
        .split('.')
        .map(str::parse)
        .collect::<Result<Vec<u64>, _>>()
        .ok()?;
    parts.try_into().ok()
}

fn compare(
    channel: &str,
    current: &str,
    commit: &str,
    built_at: u64,
    remote: &BuildInfo,
) -> PackageResult<&'static str> {
    if channel == "stable" {
        let local = version(current)
            .ok_or_else(|| invalid("Current stable build has an invalid version."))?;
        let latest = version(&remote.version)
            .ok_or_else(|| invalid("Release contains an invalid stable version."))?;
        Ok(if latest > local {
            "available"
        } else if latest == local {
            "current"
        } else {
            "ahead"
        })
    } else {
        if commit.is_empty() {
            return Ok("unknown");
        }
        if remote.commit == commit {
            return Ok("current");
        }
        Ok(match remote.built_at {
            Some(time) if built_at > 0 && time > built_at => "available",
            Some(time) if built_at > 0 && time <= built_at => "ahead",
            _ => "unknown",
        })
    }
}

pub fn check_app(
    channel: &str,
    current: &str,
    commit: &str,
    built_at: u64,
) -> PackageResult<AppUpdate> {
    release_download_base(channel)?;
    let endpoint = if channel == "stable" {
        format!("{API}/latest")
    } else {
        format!("{API}/tags/{channel}")
    };
    let release: Release = serde_json::from_slice(&fetch_json(&endpoint, 256 * 1024)?)?;
    release_page(&release.tag_name)?;
    if release.draft
        || (channel == "stable" && release.prerelease)
        || (channel != "stable" && release.tag_name != channel)
    {
        return Err(invalid(
            "Release does not match this build's update channel.",
        ));
    }
    let info: BuildInfo = serde_json::from_slice(&fetch_json(
        &format!(
            "{REPO}/releases/download/{}/build-info.json",
            release.tag_name
        ),
        64 * 1024,
    )?)?;
    if info.channel
        != if channel == "stable" {
            "versioned"
        } else {
            channel
        }
        || (channel == "stable" && release.tag_name != format!("v{}", info.version))
    {
        return Err(invalid(
            "Release metadata does not match the selected channel. Try again after publication finishes.",
        ));
    }
    let status = compare(channel, current, commit, built_at, &info)?;
    let message = match status {
        "available" => {
            "A newer build is available. Open the release page to download Windows Setup. Installation is manual."
        }
        "current" => "You are using the published build for this channel.",
        "ahead" => "Your build is newer than the published release. No downgrade is offered.",
        _ => {
            "This release lacks enough build information to compare. You can inspect its release page."
        }
    };
    Ok(AppUpdate {
        channel: channel.into(),
        current_version: current.into(),
        latest_version: info.version,
        current_commit: commit.into(),
        latest_commit: info.commit,
        tag: release.tag_name,
        status: status.into(),
        message: message.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn info(version: &str, sha: &str, time: Option<u64>) -> BuildInfo {
        BuildInfo {
            version: version.into(),
            channel: "dev".into(),
            commit: sha.into(),
            built_at: time,
        }
    }
    #[test]
    fn rolling_checks_commit_and_build_time_not_version_string() {
        let remote = info("0.1.0-beta-1", "new", Some(200));
        assert_eq!(
            compare("dev", "0.1.0-beta-1", "old", 100, &remote).unwrap(),
            "available"
        );
        assert_eq!(
            compare("dev", "0.1.0-beta-1", "new", 100, &remote).unwrap(),
            "current"
        );
        assert_eq!(
            compare("beta", "0.1.0-beta-1", "local", 300, &remote).unwrap(),
            "ahead"
        );
        assert_eq!(
            compare("dev", "0.1.0-beta-1", "", 100, &remote).unwrap(),
            "unknown"
        );
        assert_eq!(
            compare(
                "dev",
                "0.1.0-beta-1",
                "old",
                100,
                &info("0.1.0-beta-1", "new", None)
            )
            .unwrap(),
            "unknown"
        );
    }
    #[test]
    fn stable_versions_compare_numerically_and_reject_prereleases() {
        assert_eq!(
            compare("stable", "1.9.0", "", 0, &info("1.10.0", "", None)).unwrap(),
            "available"
        );
        assert_eq!(
            compare("stable", "2.0.0", "", 0, &info("1.10.0", "", None)).unwrap(),
            "ahead"
        );
        assert!(compare("stable", "1.0.0", "", 0, &info("2.0.0-beta-1", "", None)).is_err());
    }
    #[test]
    fn release_routes_reject_untrusted_paths() {
        assert!(release_download_base("../evil").is_err());
        assert!(release_page("v1.0.0/../../evil").is_err());
        assert!(release_page("https://evil.test").is_err());
        assert!(
            release_download_base("beta")
                .unwrap()
                .ends_with("/download/beta")
        );
        assert!(
            release_page("v1.2.0-beta-1")
                .unwrap()
                .ends_with("/tag/v1.2.0-beta-1")
        );
    }
}
