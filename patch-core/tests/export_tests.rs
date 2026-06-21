use patch_core::resource::export::{target_resources, ExportTextEntry};
use patch_core::resource::key::stable_text_key;

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
