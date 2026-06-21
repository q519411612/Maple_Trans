use std::fs;
use std::path::Path;

use crate::backup::BackupRecord;
use crate::error::{PatchError, PatchResult};
use crate::hash::sha256_file_hex;

#[derive(Debug)]
pub struct RestoreRequest<'a> {
    pub game_dir: &'a Path,
    pub backup_dir: &'a Path,
    pub record: &'a BackupRecord,
}

pub fn restore_backup(request: RestoreRequest<'_>) -> PatchResult<()> {
    for file in &request.record.files {
        let backup = request.backup_dir.join(&file.relative_path);
        if !backup.exists() {
            return Err(PatchError::Validation(format!(
                "backup file missing: {}",
                file.relative_path
            )));
        }
        let backup_hash = sha256_file_hex(&backup)?;
        if backup_hash != file.original_hash {
            return Err(PatchError::Validation(format!(
                "backup hash mismatch: {}",
                file.relative_path
            )));
        }

        let target = request.game_dir.join(&file.relative_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&backup, &target)?;
    }

    Ok(())
}
