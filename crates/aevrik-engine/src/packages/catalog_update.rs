use super::{PackageError, PackageManager, PackageResult, catalog};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
};

const LIMIT: usize = 4 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Feed {
    schema_version: u32,
    packages: Vec<catalog::Package>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogUpdate {
    pub added: usize,
    pub total: usize,
}

fn merge(
    mut known: Vec<catalog::Package>,
    incoming: Vec<catalog::Package>,
) -> PackageResult<Vec<catalog::Package>> {
    let mut identities: HashMap<String, usize> = known
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id.clone(), i))
        .collect();
    for package in incoming {
        if let Some(index) = identities.get(&package.id) {
            if !known[*index].same_identity(&package) {
                return Err(PackageError::Invalid(format!(
                    "Catalog changed the pinned identity of {}. Existing catalog retained.",
                    package.id
                )));
            }
        } else {
            if known.iter().any(|p| {
                p.family == package.family
                    && p.version == package.version
                    && p.architecture == package.architecture
            }) {
                return Err(PackageError::Invalid(
                    "Catalog reused a product version with a different ID.".into(),
                ));
            }
            identities.insert(package.id.clone(), known.len());
            known.push(package);
        }
    }
    catalog::validate(&known)?;
    Ok(known)
}

fn decode(bytes: &[u8]) -> PackageResult<Vec<catalog::Package>> {
    if bytes.len() > LIMIT {
        return Err(PackageError::Invalid(
            "Catalog response is too large.".into(),
        ));
    }
    let feed: Feed = serde_json::from_slice(bytes)?;
    if feed.schema_version != 1 {
        return Err(PackageError::Invalid(
            "This catalog requires a newer Aevrik application.".into(),
        ));
    }
    catalog::validate(&feed.packages)?;
    Ok(feed.packages)
}

impl PackageManager {
    pub(super) fn catalog(&self) -> PackageResult<Vec<catalog::Package>> {
        let bundled = catalog::all()?;
        match fs::File::open(self.root.join(".catalog.json")) {
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take(LIMIT as u64 + 1).read_to_end(&mut bytes)?;
                merge(bundled, decode(&bytes)?)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(bundled),
            Err(error) => Err(error.into()),
        }
    }

    pub fn refresh_catalog(&self, channel: &str) -> PackageResult<CatalogUpdate> {
        let route = crate::updates::release_download_base(channel)?;
        let bytes = crate::updates::fetch_json(&format!("{route}/runtime-catalog.json"), LIMIT)?;
        self.apply_catalog(&bytes)
    }

    pub(super) fn apply_catalog(&self, bytes: &[u8]) -> PackageResult<CatalogUpdate> {
        let incoming = decode(bytes)?;
        let _lock = self.package_lock("catalog-refresh")?;
        let known = self.catalog()?;
        let before = known.len();
        let merged = merge(known, incoming)?;
        let result = CatalogUpdate {
            added: merged.len() - before,
            total: merged.len(),
        };
        let feed = Feed {
            schema_version: 1,
            packages: merged,
        };
        let serialized = serde_json::to_vec(&feed)?;
        if serialized.len() > LIMIT {
            return Err(PackageError::Invalid(
                "Combined catalog exceeds the size limit.".into(),
            ));
        }
        let mut file = tempfile::NamedTempFile::new_in(&self.root)?;
        file.write_all(&serialized)?;
        file.as_file().sync_all()?;
        file.persist(self.root.join(".catalog.json"))
            .map_err(|error| error.error)?;
        Ok(result)
    }
}
