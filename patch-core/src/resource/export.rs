use crate::error::{PatchError, PatchResult};

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
