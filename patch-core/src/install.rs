use std::fs;
use std::path::Path;

use crate::backup::{create_backup, BackupRecord, BackupRequest};
use crate::error::{PatchError, PatchResult};
use crate::hash::sha256_file_hex;
use crate::manifest::ResourceManifest;
use crate::plan::{build_patch_plan, LanguageMode};
use crate::resource::synthetic::SyntheticResource;
use crate::resource::ResourceText;
use crate::translation::TranslationEntry;

#[derive(Debug)]
pub struct InstallRequest<'a> {
    pub game_dir: &'a Path,
    pub backup_dir: &'a Path,
    pub resource: &'a ResourceManifest,
    pub entries: &'a [TranslationEntry],
    pub language_mode: LanguageMode,
    pub allow_unknown_version: bool,
}

#[derive(Debug)]
pub struct InstallReceipt {
    pub backup: BackupRecord,
    pub changed_files: Vec<String>,
}

pub fn install_synthetic(request: InstallRequest<'_>) -> PatchResult<InstallReceipt> {
    let target = request.game_dir.join(&request.resource.source);
    let actual_hash = sha256_file_hex(&target)?;
    if actual_hash != request.resource.hash && !request.allow_unknown_version {
        return Err(PatchError::UnsupportedClientVersion(actual_hash));
    }

    let backup = create_backup(BackupRequest {
        game_dir: request.game_dir,
        backup_dir: request.backup_dir,
        relative_files: &[request.resource.source.as_str()],
    })?;

    let plan = build_patch_plan(request.resource, request.entries, request.language_mode)?;
    let input = fs::read_to_string(&target)?;
    let mut resource = SyntheticResource::from_str(&input)?;
    for edit in &plan.edits {
        resource.set_text(&edit.key, &edit.replacement)?;
    }

    let temporary = target.with_extension("synthetic.json.tmp");
    fs::write(&temporary, resource.to_string_pretty()?)?;
    fs::remove_file(&target)?;
    fs::rename(&temporary, &target)?;

    Ok(InstallReceipt {
        backup,
        changed_files: vec![request.resource.source.clone()],
    })
}
