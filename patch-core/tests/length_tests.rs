use patch_core::length::{validate_text_length, TextTarget};
use patch_core::manifest::LengthPolicy;

#[test]
fn accepts_short_strict_name() {
    validate_text_length(
        TextTarget {
            key: "item.1.name",
            text: "红色药水",
        },
        LengthPolicy::StrictName,
    )
    .unwrap();
}

#[test]
fn rejects_overlong_strict_name() {
    let error = validate_text_length(
        TextTarget {
            key: "item.1.name",
            text: "这是一段明显超过首版严格名称预算的物品名称",
        },
        LengthPolicy::StrictName,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("item.1.name"));
    assert!(error.contains("strict-name"));
}
