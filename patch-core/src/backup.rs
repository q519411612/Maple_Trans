use std::fs;
use std::path::Path;

use crate::error::{PatchError, PatchResult};
use crate::hash::sha256_file_hex;

#[derive(Debug)]
pub struct BackupRequest<'a> {
    pub game_dir: &'a Path,
    pub backup_dir: &'a Path,
    pub relative_files: &'a [&'a str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupRecord {
    pub files: Vec<BackupFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupFile {
    pub relative_path: String,
    pub original_hash: String,
}

pub fn create_backup(request: BackupRequest<'_>) -> PatchResult<BackupRecord> {
    fs::create_dir_all(request.backup_dir)?;
    let mut files = Vec::new();

    for relative_file in request.relative_files {
        let source = request.game_dir.join(relative_file);
        let destination = request.backup_dir.join(relative_file);
        let parent = destination.parent().ok_or_else(|| {
            PatchError::Validation(format!("backup path has no parent: {}", relative_file))
        })?;
        fs::create_dir_all(parent)?;
        fs::copy(&source, &destination)?;
        files.push(BackupFile {
            relative_path: (*relative_file).to_owned(),
            original_hash: sha256_file_hex(&source)?,
        });
    }

    Ok(BackupRecord { files })
}
