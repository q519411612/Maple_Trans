use std::fs;
use std::path::{Path, PathBuf};

use patch_core::hash::sha256_file_hex;
use patch_core::install::{install_synthetic, InstallRequest};
use patch_core::manifest::Manifest;
use patch_core::plan::LanguageMode;
use patch_core::restore::{restore_backup, RestoreRequest};
use patch_core::translation::parse_jsonl;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "camelCase")]
pub enum ClientStatus {
    #[serde(rename = "notSelected")]
    NotSelected,
    #[serde(rename = "supported")]
    Supported {
        version: String,
        installed: bool,
        changed_files: Vec<String>,
    },
    #[serde(rename = "unsupported")]
    Unsupported { reason: String },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    ok: bool,
    message: String,
    backup_path: Option<String>,
}

#[tauri::command]
pub fn select_game_dir_status(path: String) -> Result<ClientStatus, String> {
    if path.trim().is_empty() {
        return Ok(ClientStatus::NotSelected);
    }

    let game_dir = PathBuf::from(path);
    let manifest = load_manifest()?;
    let resource = manifest
        .resources
        .first()
        .ok_or_else(|| "manifest has no resources".to_owned())?;
    let resource_path = game_dir.join(&resource.source);

    if !resource_path.exists() {
        return Ok(ClientStatus::Unsupported {
            reason: format!("missing resource: {}", resource.source),
        });
    }

    let actual_hash = sha256_file_hex(&resource_path).map_err(|error| error.to_string())?;
    if actual_hash != resource.hash {
        return Ok(ClientStatus::Unsupported {
            reason: "current version is not supported yet".to_owned(),
        });
    }

    Ok(ClientStatus::Supported {
        version: manifest
            .supported_client_versions
            .first()
            .cloned()
            .unwrap_or_else(|| "synthetic".to_owned()),
        installed: false,
        changed_files: vec![resource.source.clone()],
    })
}

#[tauri::command]
pub fn install_localization(path: String, mode: String) -> Result<OperationResult, String> {
    let game_dir = PathBuf::from(path);
    let manifest = load_manifest()?;
    let resource = manifest
        .resources
        .first()
        .ok_or_else(|| "manifest has no resources".to_owned())?;
    let entries = load_entries(&mode)?;
    let backup_dir = app_backup_dir(&game_dir);
    let language_mode = if mode == "zh-CN-bilingual" {
        LanguageMode::Bilingual
    } else {
        LanguageMode::SimplifiedChinese
    };

    let receipt = install_synthetic(InstallRequest {
        game_dir: &game_dir,
        backup_dir: &backup_dir,
        resource,
        entries: &entries,
        language_mode,
        allow_unknown_version: false,
    })
    .map_err(|error| error.to_string())?;

    Ok(OperationResult {
        ok: true,
        message: format!("installed localization for {}", receipt.changed_files.join(", ")),
        backup_path: Some(backup_dir.display().to_string()),
    })
}

#[tauri::command]
pub fn restore_original(path: String) -> Result<OperationResult, String> {
    let game_dir = PathBuf::from(path);
    let backup_dir = app_backup_dir(&game_dir);
    let manifest = load_manifest()?;
    let resource = manifest
        .resources
        .first()
        .ok_or_else(|| "manifest has no resources".to_owned())?;
    let backup = patch_core::backup::BackupRecord {
        files: vec![patch_core::backup::BackupFile {
            relative_path: resource.source.clone(),
            original_hash: resource.hash.clone(),
        }],
    };

    restore_backup(RestoreRequest {
        game_dir: &game_dir,
        backup_dir: &backup_dir,
        record: &backup,
    })
    .map_err(|error| error.to_string())?;

    Ok(OperationResult {
        ok: true,
        message: "restored original files".to_owned(),
        backup_path: Some(backup_dir.display().to_string()),
    })
}

#[tauri::command]
pub fn check_translation_data() -> OperationResult {
    OperationResult {
        ok: true,
        message: "translation data is local; remote update source is not configured yet".to_owned(),
        backup_path: None,
    }
}

fn load_manifest() -> Result<Manifest, String> {
    let path = project_root().join("translations/maplelegends/manifest.toml");
    let input = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    toml::from_str(&input).map_err(|error| error.to_string())
}

fn load_entries(mode: &str) -> Result<Vec<patch_core::translation::TranslationEntry>, String> {
    let path = project_root()
        .join("translations/maplelegends")
        .join(mode)
        .join("item.jsonl");
    let input = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    parse_jsonl(&input).map_err(|error| error.to_string())
}

fn app_backup_dir(game_dir: &Path) -> PathBuf {
    game_dir.join(".open-maple-patch-backup")
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}
