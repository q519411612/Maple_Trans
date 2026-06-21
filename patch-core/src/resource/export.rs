use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{PatchError, PatchResult};
use crate::resource::key::stable_text_key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetResource {
    pub id: &'static str,
    pub relative_path: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExportTextEntry {
    pub key: String,
    pub source: String,
    pub resource: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTextNode {
    pub resource: String,
    pub path: String,
    pub value: String,
}

const TARGET_RESOURCES: [TargetResource; 9] = [
    TargetResource {
        id: "string-eqp",
        relative_path: "Data/String/Eqp.img",
    },
    TargetResource {
        id: "string-consume",
        relative_path: "Data/String/Consume.img",
    },
    TargetResource {
        id: "string-etc",
        relative_path: "Data/String/Etc.img",
    },
    TargetResource {
        id: "string-ins",
        relative_path: "Data/String/Ins.img",
    },
    TargetResource {
        id: "string-cash",
        relative_path: "Data/String/Cash.img",
    },
    TargetResource {
        id: "string-pet",
        relative_path: "Data/String/Pet.img",
    },
    TargetResource {
        id: "string-skill",
        relative_path: "Data/String/Skill.img",
    },
    TargetResource {
        id: "string-map",
        relative_path: "Data/String/Map.img",
    },
    TargetResource {
        id: "quest-info",
        relative_path: "Data/Quest/QuestInfo.img",
    },
];

impl ExportTextEntry {
    pub fn to_json_line(&self) -> PatchResult<String> {
        serde_json::to_string(self)
            .map_err(|error| PatchError::Validation(format!("cannot write export entry: {error}")))
    }
}

pub fn target_resources() -> &'static [TargetResource] {
    &TARGET_RESOURCES
}

pub fn validate_export_client_dir(game_dir: &Path) -> PatchResult<()> {
    let executable = game_dir.join("MapleLegends.exe");
    if !executable.is_file() {
        return Err(PatchError::Validation(format!(
            "missing MapleLegends.exe: {}",
            executable.display()
        )));
    }

    for resource in target_resources() {
        let resource_path = game_dir.join(resource.relative_path);
        if !resource_path.is_file() {
            return Err(PatchError::Validation(format!(
                "missing target resource: {}",
                resource_path.display()
            )));
        }
    }

    Ok(())
}

pub fn write_export_jsonl(
    out_dir: &Path,
    file_name: &str,
    entries: &[ExportTextEntry],
) -> PatchResult<PathBuf> {
    let mut seen = BTreeSet::new();
    for entry in entries {
        if !seen.insert(&entry.key) {
            return Err(PatchError::Validation(format!(
                "duplicate export key: {}",
                entry.key
            )));
        }
    }

    let mut sorted = entries.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.key.cmp(&right.key));

    fs::create_dir_all(out_dir)?;
    let out_path = out_dir.join(file_name);
    let mut output = String::new();
    for entry in sorted {
        output.push_str(&entry.to_json_line()?);
        output.push('\n');
    }
    fs::write(&out_path, output)?;

    Ok(out_path)
}

pub fn collect_export_entries(nodes: Vec<RawTextNode>) -> PatchResult<Vec<ExportTextEntry>> {
    nodes
        .into_iter()
        .map(|node| {
            let key = stable_text_key(&node.resource, &node.path)?;
            Ok(ExportTextEntry {
                key,
                source: node.value,
                resource: node.resource,
                path: node.path,
            })
        })
        .collect()
}
