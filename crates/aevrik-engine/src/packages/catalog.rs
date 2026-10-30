use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
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
    pub sha256: String,
    pub executable: String,
}
fn default_architecture() -> String {
    "x64".into()
}
pub fn all() -> super::PackageResult<Vec<Package>> {
    let packages: Vec<Package> = serde_json::from_str(include_str!("catalog.json"))?;
    let mut ids = std::collections::HashSet::new();
    for package in &packages {
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
            || package.sha256.len() != 64
            || !package.sha256.chars().all(|c| c.is_ascii_hexdigit())
            || url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(super::PackageError::Invalid(
                "Invalid bundled package metadata.".into(),
            ));
        }
        super::archive::safe_path(&package.version)?;
        super::archive::safe_path(&package.executable)?;
    }
    Ok(packages)
}
