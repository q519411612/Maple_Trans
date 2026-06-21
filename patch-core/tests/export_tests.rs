use patch_core::resource::export::{collect_export_entries, target_resources, ExportTextEntry};
use patch_core::resource::export::{validate_export_client_dir, write_export_jsonl};
use patch_core::resource::export::RawTextNode;
use patch_core::resource::key::stable_text_key;
use patch_core::resource::wz_img::read_img_text_nodes;
use std::fs;

#[test]
fn target_resources_exclude_dialogue_say_resource() {
    let resources = target_resources();
    let paths: Vec<_> = resources
        .iter()
        .map(|resource| resource.relative_path)
        .collect();

    assert!(paths.contains(&"Data/String/Eqp.img"));
    assert!(paths.contains(&"Data/Quest/QuestInfo.img"));
    assert!(!paths.contains(&"Data/Quest/Say.img"));
}

#[test]
fn stable_text_key_uses_resource_kind_id_and_field() {
    let key = stable_text_key("Data/String/Eqp.img", "Eqp.img/01000001/name").unwrap();

    assert_eq!(key, "string.eqp.01000001.name");
}

#[test]
fn stable_text_key_rejects_unknown_field() {
    let error = stable_text_key("Data/String/Eqp.img", "Eqp.img/01000001/icon").unwrap_err();

    assert!(error.to_string().contains("unsupported text field"));
}

#[test]
fn export_text_entry_serializes_as_json_line() {
    let entry = ExportTextEntry {
        key: "string.eqp.01000001.name".to_owned(),
        source: "Sample text".to_owned(),
        resource: "Data/String/Eqp.img".to_owned(),
        path: "Eqp.img/01000001/name".to_owned(),
    };

    let line = entry.to_json_line().unwrap();

    assert_eq!(
        line,
        r#"{"key":"string.eqp.01000001.name","source":"Sample text","resource":"Data/String/Eqp.img","path":"Eqp.img/01000001/name"}"#
    );
}

#[test]
fn validate_export_client_dir_requires_executable() {
    let temp = tempfile::tempdir().unwrap();

    let error = validate_export_client_dir(temp.path()).unwrap_err();

    assert!(error.to_string().contains("missing MapleLegends.exe"));
}

#[test]
fn validate_export_client_dir_requires_target_resources() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("MapleLegends.exe"), "").unwrap();

    let error = validate_export_client_dir(temp.path()).unwrap_err();

    assert!(error.to_string().contains("missing target resource"));
}

#[test]
fn write_export_jsonl_rejects_duplicate_keys() {
    let temp = tempfile::tempdir().unwrap();
    let entries = vec![
        ExportTextEntry {
            key: "string.eqp.01000001.name".to_owned(),
            source: "A".to_owned(),
            resource: "Data/String/Eqp.img".to_owned(),
            path: "Eqp.img/01000001/name".to_owned(),
        },
        ExportTextEntry {
            key: "string.eqp.01000001.name".to_owned(),
            source: "B".to_owned(),
            resource: "Data/String/Eqp.img".to_owned(),
            path: "Eqp.img/01000002/name".to_owned(),
        },
    ];

    let error = write_export_jsonl(temp.path(), "item.jsonl", &entries).unwrap_err();

    assert!(error.to_string().contains("duplicate export key"));
}

#[test]
fn write_export_jsonl_sorts_entries_by_key() {
    let temp = tempfile::tempdir().unwrap();
    let entries = vec![
        ExportTextEntry {
            key: "string.eqp.01000002.name".to_owned(),
            source: "B".to_owned(),
            resource: "Data/String/Eqp.img".to_owned(),
            path: "Eqp.img/01000002/name".to_owned(),
        },
        ExportTextEntry {
            key: "string.eqp.01000001.name".to_owned(),
            source: "A".to_owned(),
            resource: "Data/String/Eqp.img".to_owned(),
            path: "Eqp.img/01000001/name".to_owned(),
        },
    ];

    let path = write_export_jsonl(temp.path(), "item.jsonl", &entries).unwrap();
    let output = fs::read_to_string(path).unwrap();

    assert!(output.starts_with(r#"{"key":"string.eqp.01000001.name""#));
}

#[test]
fn collect_export_entries_converts_known_text_nodes_to_entries() {
    let nodes = vec![RawTextNode {
        resource: "Data/String/Eqp.img".to_owned(),
        path: "Eqp.img/01000001/name".to_owned(),
        value: "Sample text".to_owned(),
    }];

    let entries = collect_export_entries(nodes).unwrap();

    assert_eq!(entries[0].key, "string.eqp.01000001.name");
    assert_eq!(entries[0].source, "Sample text");
}

#[test]
fn collect_export_entries_reports_unknown_text_nodes() {
    let nodes = vec![RawTextNode {
        resource: "Data/String/Eqp.img".to_owned(),
        path: "Eqp.img/01000001/icon".to_owned(),
        value: "icon data".to_owned(),
    }];

    let error = collect_export_entries(nodes).unwrap_err();

    assert!(error.to_string().contains("unsupported text field"));
}

#[test]
fn read_img_text_nodes_reports_missing_resource() {
    let temp = tempfile::tempdir().unwrap();
    let target = target_resources()[0];

    let error = read_img_text_nodes(temp.path(), target).unwrap_err();

    assert!(error.to_string().contains("missing target resource"));
}
