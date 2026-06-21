#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProbeReport {
    pub client_root: String,
    pub executable_exists: bool,
    pub list: ListProbe,
    pub resources: Vec<ResourceProbe>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ListProbe {
    pub path: String,
    pub exists: bool,
    pub sha256: Option<String>,
    pub magic: Option<String>,
    pub attempts: Vec<ProbeAttempt>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ResourceProbe {
    pub id: String,
    pub path: String,
    pub exists: bool,
    pub sha256: Option<String>,
    pub magic: Option<String>,
    pub attempts: Vec<ProbeAttempt>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProbeAttempt {
    pub mode: String,
    pub status: ProbeStatus,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    Readable,
    Failed,
    Missing,
}

pub fn format_magic(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}
