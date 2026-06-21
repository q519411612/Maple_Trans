use patch_core::manifest::{LengthPolicy, Manifest, ResourceKind};

#[test]
fn parses_manifest_with_supported_modes_and_resources() {
    let input = r#"
game = "maplelegends"
supported_client_versions = ["synthetic-001"]
modes = ["zh-CN", "zh-CN-bilingual"]

[[resources]]
id = "string-item"
source = "String/Item.synthetic.json"
kind = "item"
hash = "abc123"
length_policy = "strict-name"
"#;

    let manifest: Manifest = toml::from_str(input).unwrap();

    assert_eq!(manifest.game, "maplelegends");
    assert_eq!(manifest.supported_client_versions, vec!["synthetic-001"]);
    assert_eq!(manifest.modes, vec!["zh-CN", "zh-CN-bilingual"]);
    assert_eq!(manifest.resources[0].kind, ResourceKind::Item);
    assert_eq!(
        manifest.resources[0].length_policy,
        LengthPolicy::StrictName
    );
}
