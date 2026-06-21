use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub game: String,
    pub supported_client_versions: Vec<String>,
    pub modes: Vec<String>,
    pub resources: Vec<ResourceManifest>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ResourceManifest {
    pub id: String,
    pub source: String,
    pub kind: ResourceKind,
    pub hash: String,
    pub length_policy: LengthPolicy,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceKind {
    Item,
    Skill,
    Map,
    QuestTitle,
    QuestInfo,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LengthPolicy {
    StrictName,
    ExpandedText,
}

impl LengthPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StrictName => "strict-name",
            Self::ExpandedText => "expanded-text",
        }
    }
}
