use patch_core::update::{validate_update_manifest, DataFile, UpdateManifest};

#[test]
fn accepts_translation_data_update_manifest() {
    let manifest = UpdateManifest {
        version: "2026.06.21".to_owned(),
        manifest_url: "https://example.invalid/manifest.toml".to_owned(),
        manifest_sha256: "a".repeat(64),
        data_files: vec![DataFile {
            path: "zh-CN/item.jsonl".to_owned(),
            sha256: "b".repeat(64),
        }],
    };

    validate_update_manifest(&manifest).unwrap();
}

#[test]
fn rejects_client_resource_assets() {
    let manifest = UpdateManifest {
        version: "2026.06.21".to_owned(),
        manifest_url: "https://example.invalid/manifest.toml".to_owned(),
        manifest_sha256: "a".repeat(64),
        data_files: vec![DataFile {
            path: "String.wz".to_owned(),
            sha256: "b".repeat(64),
        }],
    };

    let error = validate_update_manifest(&manifest)
        .unwrap_err()
        .to_string();

    assert!(error.contains("client resource asset"));
}
