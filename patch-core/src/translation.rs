use std::collections::HashSet;

use serde::Deserialize;

use crate::error::{PatchError, PatchResult};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct TranslationEntry {
    pub key: String,
    pub source: String,
    #[serde(rename = "zh_CN")]
    pub zh_cn: String,
    pub bilingual: String,
    pub status: ReviewStatus,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewStatus {
    Draft,
    Reviewed,
}

pub fn parse_jsonl(input: &str) -> PatchResult<Vec<TranslationEntry>> {
    let mut entries = Vec::new();
    let mut keys = HashSet::new();

    for (index, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let entry: TranslationEntry = serde_json::from_str(line).map_err(|error| {
            PatchError::Validation(format!("invalid jsonl line {}: {}", index + 1, error))
        })?;

        if !keys.insert(entry.key.clone()) {
            return Err(PatchError::Validation(format!(
                "duplicate translation key: {}",
                entry.key
            )));
        }

        entries.push(entry);
    }

    Ok(entries)
}
