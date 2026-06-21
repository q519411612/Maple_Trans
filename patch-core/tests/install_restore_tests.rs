use std::fs;

use patch_core::backup::{create_backup, BackupRequest};
use patch_core::hash::sha256_file_hex;
use patch_core::install::{install_synthetic, InstallRequest};
use patch_core::manifest::{LengthPolicy, ResourceKind, ResourceManifest};
use patch_core::plan::LanguageMode;
use patch_core::restore::{restore_backup, RestoreRequest};
use patch_core::translation::{ReviewStatus, TranslationEntry};

#[test]
fn computes_file_sha256() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sample.txt");
    fs::write(&path, "abc").unwrap();

    let hash = sha256_file_hex(&path).unwrap();

    assert_eq!(
        hash,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn creates_backup_and_restores_original_file() {
    let dir = tempfile::tempdir().unwrap();
    let game_dir = dir.path().join("game");
    let backup_dir = dir.path().join("backup");
    fs::create_dir_all(&game_dir).unwrap();

    let resource = game_dir.join("String/Item.synthetic.json");
    fs::create_dir_all(resource.parent().unwrap()).unwrap();
    fs::write(&resource, r#"{"item.1.name":"Red Potion"}"#).unwrap();

    let record = create_backup(BackupRequest {
        game_dir: &game_dir,
        backup_dir: &backup_dir,
        relative_files: &["String/Item.synthetic.json"],
    })
    .unwrap();

    fs::write(&resource, r#"{"item.1.name":"红色药水"}"#).unwrap();

    restore_backup(RestoreRequest {
        game_dir: &game_dir,
        backup_dir: &backup_dir,
        record: &record,
    })
    .unwrap();

    let restored = fs::read_to_string(&resource).unwrap();
    assert!(restored.contains("Red Potion"));
}

#[test]
fn installs_synthetic_patch_after_hash_match() {
    let dir = tempfile::tempdir().unwrap();
    let game_dir = dir.path().join("game");
    let backup_dir = dir.path().join("backup");
    fs::create_dir_all(game_dir.join("String")).unwrap();

    let resource_path = game_dir.join("String/Item.synthetic.json");
    fs::write(&resource_path, r#"{"item.2000000.name":"Red Potion"}"#).unwrap();
    let original_hash = sha256_file_hex(&resource_path).unwrap();

    let resource = ResourceManifest {
        id: "string-item".to_owned(),
        source: "String/Item.synthetic.json".to_owned(),
        kind: ResourceKind::Item,
        hash: original_hash,
        length_policy: LengthPolicy::StrictName,
    };
    let entries = vec![TranslationEntry {
        key: "item.2000000.name".to_owned(),
        source: "Red Potion".to_owned(),
        zh_cn: "红色药水".to_owned(),
        bilingual: "红色药水 / Red Potion".to_owned(),
        status: ReviewStatus::Reviewed,
    }];

    let receipt = install_synthetic(InstallRequest {
        game_dir: &game_dir,
        backup_dir: &backup_dir,
        resource: &resource,
        entries: &entries,
        language_mode: LanguageMode::SimplifiedChinese,
        allow_unknown_version: false,
    })
    .unwrap();

    let patched = fs::read_to_string(&resource_path).unwrap();
    assert!(patched.contains("红色药水"));
    assert!(backup_dir.join("String/Item.synthetic.json").exists());
    assert_eq!(receipt.changed_files, vec!["String/Item.synthetic.json"]);
}

#[test]
fn rejects_unknown_hash_without_developer_mode() {
    let dir = tempfile::tempdir().unwrap();
    let game_dir = dir.path().join("game");
    let backup_dir = dir.path().join("backup");
    fs::create_dir_all(game_dir.join("String")).unwrap();
    fs::write(
        game_dir.join("String/Item.synthetic.json"),
        r#"{"item.2000000.name":"Red Potion"}"#,
    )
    .unwrap();

    let resource = ResourceManifest {
        id: "string-item".to_owned(),
        source: "String/Item.synthetic.json".to_owned(),
        kind: ResourceKind::Item,
        hash: "not-the-real-hash".to_owned(),
        length_policy: LengthPolicy::StrictName,
    };

    let error = install_synthetic(InstallRequest {
        game_dir: &game_dir,
        backup_dir: &backup_dir,
        resource: &resource,
        entries: &[],
        language_mode: LanguageMode::SimplifiedChinese,
        allow_unknown_version: false,
    })
    .unwrap_err()
    .to_string();

    assert!(error.contains("unsupported client version"));
}
