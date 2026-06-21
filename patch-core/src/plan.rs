use crate::error::PatchResult;
use crate::length::{validate_text_length, TextTarget};
use crate::manifest::ResourceManifest;
use crate::translation::{ReviewStatus, TranslationEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageMode {
    SimplifiedChinese,
    Bilingual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchPlan {
    pub resource_id: String,
    pub source: String,
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub key: String,
    pub replacement: String,
}

pub fn build_patch_plan(
    resource: &ResourceManifest,
    entries: &[TranslationEntry],
    mode: LanguageMode,
) -> PatchResult<PatchPlan> {
    let mut edits = Vec::new();

    for entry in entries {
        if entry.status != ReviewStatus::Reviewed {
            continue;
        }

        let replacement = match mode {
            LanguageMode::SimplifiedChinese => entry.zh_cn.clone(),
            LanguageMode::Bilingual => entry.bilingual.clone(),
        };

        validate_text_length(
            TextTarget {
                key: &entry.key,
                text: &replacement,
            },
            resource.length_policy.clone(),
        )?;

        edits.push(TextEdit {
            key: entry.key.clone(),
            replacement,
        });
    }

    Ok(PatchPlan {
        resource_id: resource.id.clone(),
        source: resource.source.clone(),
        edits,
    })
}
