use serde::{Deserialize, Serialize};

use crate::error::{PatchError, PatchResult};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateManifest {
    pub version: String,
    pub manifest_url: String,
    pub manifest_sha256: String,
    pub data_files: Vec<DataFile>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DataFile {
    pub path: String,
    pub sha256: String,
}

pub fn validate_update_manifest(manifest: &UpdateManifest) -> PatchResult<()> {
    validate_hash("manifest_sha256", &manifest.manifest_sha256)?;

    for file in &manifest.data_files {
        reject_client_asset(&file.path)?;
        validate_hash(&file.path, &file.sha256)?;
    }

    Ok(())
}

fn validate_hash(label: &str, hash: &str) -> PatchResult<()> {
    let is_sha256 = hash.len() == 64 && hash.chars().all(|character| character.is_ascii_hexdigit());
    if !is_sha256 {
        return Err(PatchError::Validation(format!(
            "invalid sha256 for {}",
            label
        )));
    }
    Ok(())
}

fn reject_client_asset(path: &str) -> PatchResult<()> {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".wz") || lower.ends_with(".img") || lower.ends_with(".exe") {
        return Err(PatchError::Validation(format!(
            "client resource asset is not allowed in data updates: {}",
            path
        )));
    }
    Ok(())
}
