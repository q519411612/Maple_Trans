use patch_core::manifest::{LengthPolicy, ResourceKind, ResourceManifest};
use patch_core::plan::{build_patch_plan, LanguageMode};
use patch_core::resource::synthetic::SyntheticResource;
use patch_core::resource::ResourceText;
use patch_core::translation::{ReviewStatus, TranslationEntry};

#[test]
fn reads_and_writes_synthetic_resource_text() {
    let input = r#"{"item.2000000.name":"Red Potion"}"#;
    let mut resource = SyntheticResource::from_str(input).unwrap();

    resource.set_text("item.2000000.name", "红色药水").unwrap();
    let output = resource.to_string_pretty().unwrap();

    assert!(output.contains("红色药水"));
}

#[test]
fn builds_patch_plan_for_reviewed_entries() {
    let resource = ResourceManifest {
        id: "string-item".to_owned(),
        source: "String/Item.synthetic.json".to_owned(),
        kind: ResourceKind::Item,
        hash: "abc123".to_owned(),
        length_policy: LengthPolicy::StrictName,
    };
    let entries = vec![TranslationEntry {
        key: "item.2000000.name".to_owned(),
        source: "Red Potion".to_owned(),
        zh_cn: "红色药水".to_owned(),
        bilingual: "红药/Red".to_owned(),
        status: ReviewStatus::Reviewed,
    }];

    let plan = build_patch_plan(&resource, &entries, LanguageMode::SimplifiedChinese).unwrap();

    assert_eq!(plan.edits.len(), 1);
    assert_eq!(plan.edits[0].replacement, "红色药水");
}

#[test]
fn builds_patch_plan_for_bilingual_entries() {
    let resource = ResourceManifest {
        id: "string-item".to_owned(),
        source: "String/Item.synthetic.json".to_owned(),
        kind: ResourceKind::Item,
        hash: "abc123".to_owned(),
        length_policy: LengthPolicy::StrictName,
    };
    let entries = vec![TranslationEntry {
        key: "item.2000000.name".to_owned(),
        source: "Red Potion".to_owned(),
        zh_cn: "红色药水".to_owned(),
        bilingual: "红药/Red".to_owned(),
        status: ReviewStatus::Reviewed,
    }];

    let plan = build_patch_plan(&resource, &entries, LanguageMode::Bilingual).unwrap();

    assert_eq!(plan.edits.len(), 1);
    assert_eq!(plan.edits[0].replacement, "红药/Red");
}

#[test]
fn skips_draft_entries() {
    let resource = ResourceManifest {
        id: "string-item".to_owned(),
        source: "String/Item.synthetic.json".to_owned(),
        kind: ResourceKind::Item,
        hash: "abc123".to_owned(),
        length_policy: LengthPolicy::StrictName,
    };
    let entries = vec![TranslationEntry {
        key: "item.2000000.name".to_owned(),
        source: "Red Potion".to_owned(),
        zh_cn: "红色药水".to_owned(),
        bilingual: "红药/Red".to_owned(),
        status: ReviewStatus::Draft,
    }];

    let plan = build_patch_plan(&resource, &entries, LanguageMode::SimplifiedChinese).unwrap();

    assert!(plan.edits.is_empty());
}
