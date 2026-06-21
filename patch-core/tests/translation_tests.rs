use patch_core::translation::{parse_jsonl, ReviewStatus};

#[test]
fn parses_translation_jsonl_entries() {
    let input = r#"{"key":"item.2000000.name","source":"Red Potion","zh_CN":"红色药水","bilingual":"红色药水 / Red Potion","status":"reviewed"}
"#;

    let entries = parse_jsonl(input).unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, "item.2000000.name");
    assert_eq!(entries[0].zh_cn, "红色药水");
    assert_eq!(entries[0].status, ReviewStatus::Reviewed);
}

#[test]
fn rejects_duplicate_keys() {
    let input = r#"{"key":"item.1.name","source":"A","zh_CN":"甲","bilingual":"甲 / A","status":"draft"}
{"key":"item.1.name","source":"A","zh_CN":"乙","bilingual":"乙 / A","status":"draft"}
"#;

    let error = parse_jsonl(input).unwrap_err().to_string();

    assert!(error.contains("duplicate translation key"));
}
