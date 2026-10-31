use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub id: String,
    #[serde(default)]
    pub family: String,
    #[serde(default = "default_architecture")]
    pub architecture: String,
    #[serde(default)]
    pub channel: String,
    pub name: String,
    pub version: String,
    pub category: String,
    pub description: String,
    pub url: String,
    pub source: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sha512: String,
    pub executable: String,
}
fn default_architecture() -> String {
    "x64".into()
}
pub fn all() -> super::PackageResult<Vec<Package>> {
    parse(include_str!("catalog.json").as_bytes())
}

pub fn parse(bytes: &[u8]) -> super::PackageResult<Vec<Package>> {
    let packages: Vec<Package> = serde_json::from_slice(bytes)?;
    validate(&packages)?;
    Ok(packages)
}

pub fn validate(packages: &[Package]) -> super::PackageResult<()> {
    if packages.len() > 10_000 {
        return Err(super::PackageError::Invalid(
            "Catalog has too many entries.".into(),
        ));
    }
    let mut ids = std::collections::HashSet::new();
    for package in packages {
        let url = reqwest::Url::parse(&package.url)
            .map_err(|_| super::PackageError::Invalid("Invalid catalog URL.".into()))?;
        if package.id.is_empty()
            || !package
                .id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '.'))
            || !ids.insert(&package.id)
            || package.version.contains(['/', '\\', ':'])
            || package.version.is_empty()
            || !((package.sha256.len() == 64
                && package.sha256.chars().all(|c| c.is_ascii_hexdigit())
                && package.sha512.is_empty())
                || (package.sha512.len() == 128
                    && package.sha512.chars().all(|c| c.is_ascii_hexdigit())
                    && package.sha256.is_empty()))
            || url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.port().is_some()
            || !matches!(
                url.host_str(),
                Some(
                    "api.nuget.org"
                        | "archive.mariadb.org"
                        | "fastdl.mongodb.org"
                        | "github.com"
                        | "go.dev"
                        | "nginx.org"
                        | "nodejs.org"
                        | "windows.php.net"
                        | "www.sqlite.org"
                        | "builds.dotnet.microsoft.com"
                        | "dotnetcli.azureedge.net"
                        | "download.visualstudio.microsoft.com"
                        | "ziglang.org"
                        | "nim-lang.org"
                )
            )
        {
            return Err(super::PackageError::Invalid(
                "Invalid package metadata or unsupported publisher.".into(),
            ));
        }
        super::archive::safe_path(&package.version)?;
        super::archive::safe_path(&package.executable)?;
    }
    Ok(())
}

impl Package {
    pub fn same_identity(&self, other: &Self) -> bool {
        self.id == other.id
            && self.family == other.family
            && self.name == other.name
            && self.version == other.version
            && self.url == other.url
            && self.sha256 == other.sha256
            && self.sha512 == other.sha512
            && self.executable == other.executable
            && self.architecture == other.architecture
    }
}
