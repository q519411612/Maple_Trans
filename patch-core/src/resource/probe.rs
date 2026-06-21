use std::fs;
use std::io::Read;
use std::path::Path;

use wz_reader::util::version::WzMapleVersion;
use wz_reader::WzNode;
use wzlib_rs::{parse_list_file_with_iv, WzMapleVersion as WzLibMapleVersion};

use crate::error::PatchResult;
use crate::hash::sha256_file_hex;
use crate::resource::export::target_resources;

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

impl ListProbe {
    pub fn status(&self) -> ProbeStatus {
        probe_status(self.exists, &self.attempts)
    }
}

impl ResourceProbe {
    pub fn status(&self) -> ProbeStatus {
        probe_status(self.exists, &self.attempts)
    }
}

pub fn format_magic(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn probe_client(game_dir: &Path) -> PatchResult<ProbeReport> {
    let client_root = game_dir.display().to_string();
    let executable_exists = game_dir.join("MapleLegends.exe").is_file();

    Ok(ProbeReport {
        client_root,
        executable_exists,
        list: probe_list(game_dir)?,
        resources: probe_resources(game_dir)?,
    })
}

fn probe_status(exists: bool, attempts: &[ProbeAttempt]) -> ProbeStatus {
    if !exists {
        return ProbeStatus::Missing;
    }

    if attempts
        .iter()
        .any(|attempt| attempt.status == ProbeStatus::Readable)
    {
        ProbeStatus::Readable
    } else {
        ProbeStatus::Failed
    }
}

fn probe_list(game_dir: &Path) -> PatchResult<ListProbe> {
    let relative_path = "list.wz";
    let path = game_dir.join(relative_path);
    let exists = path.is_file();
    let sha256 = optional_hash(&path)?;
    let magic = optional_magic(&path)?;
    let attempts = if exists {
        match fs::read(&path) {
            Ok(data) => list_attempts(&data),
            Err(error) => vec![ProbeAttempt {
                mode: "read".to_owned(),
                status: ProbeStatus::Failed,
                detail: error.to_string(),
            }],
        }
    } else {
        Vec::new()
    };

    Ok(ListProbe {
        path: relative_path.to_owned(),
        exists,
        sha256,
        magic,
        attempts,
    })
}

fn probe_resources(game_dir: &Path) -> PatchResult<Vec<ResourceProbe>> {
    target_resources()
        .iter()
        .map(|target| {
            let path = game_dir.join(target.relative_path);
            let exists = path.is_file();
            let sha256 = optional_hash(&path)?;
            let magic = optional_magic(&path)?;
            let attempts = if exists {
                img_attempts(&path)
            } else {
                Vec::new()
            };

            Ok(ResourceProbe {
                id: target.id.to_owned(),
                path: target.relative_path.to_owned(),
                exists,
                sha256,
                magic,
                attempts,
            })
        })
        .collect()
}

fn optional_hash(path: &Path) -> PatchResult<Option<String>> {
    if path.is_file() {
        sha256_file_hex(path).map(Some)
    } else {
        Ok(None)
    }
}

fn optional_magic(path: &Path) -> PatchResult<Option<String>> {
    if !path.is_file() {
        return Ok(None);
    }

    let mut file = fs::File::open(path)?;
    let mut buffer = [0u8; 16];
    let count = file.read(&mut buffer)?;
    Ok(Some(format_magic(&buffer[..count])))
}

fn list_attempts(data: &[u8]) -> Vec<ProbeAttempt> {
    [
        ("GMS", WzLibMapleVersion::Gms),
        ("EMS", WzLibMapleVersion::Ems),
        ("BMS", WzLibMapleVersion::Bms),
    ]
    .into_iter()
    .map(
        |(mode, version)| match parse_list_file_with_iv(data, version.iv()) {
            Ok(entries) if list_entries_are_plausible(&entries) => ProbeAttempt {
                mode: mode.to_owned(),
                status: ProbeStatus::Readable,
                detail: format!(
                    "entries={} first={}",
                    entries.len(),
                    entries.first().map_or("", String::as_str)
                ),
            },
            Ok(entries) => ProbeAttempt {
                mode: mode.to_owned(),
                status: ProbeStatus::Failed,
                detail: format!(
                    "entries={} first={} validation=not_plausible",
                    entries.len(),
                    entries.first().map_or("", String::as_str)
                ),
            },
            Err(error) => ProbeAttempt {
                mode: mode.to_owned(),
                status: ProbeStatus::Failed,
                detail: error.to_string(),
            },
        },
    )
    .collect()
}

fn img_attempts(path: &Path) -> Vec<ProbeAttempt> {
    let version_attempts = [
        ("auto", None),
        ("GMS", Some(WzMapleVersion::GMS)),
        ("EMS", Some(WzMapleVersion::EMS)),
        ("BMS", Some(WzMapleVersion::BMS)),
    ];
    let iv_attempts = [
        ("iv-zero", [0x00, 0x00, 0x00, 0x00]),
        ("iv-gms", [0x4D, 0x23, 0xC7, 0x2B]),
        ("iv-msea", [0xB9, 0x7D, 0x63, 0xE9]),
    ];

    let mut attempts = version_attempts
        .into_iter()
        .map(
            |(mode, version)| match WzNode::from_img_file(path, version, None) {
                Ok(node) => ProbeAttempt {
                    mode: mode.to_owned(),
                    status: ProbeStatus::Readable,
                    detail: format!("root={}", node.name),
                },
                Err(error) => ProbeAttempt {
                    mode: mode.to_owned(),
                    status: ProbeStatus::Failed,
                    detail: error.to_string(),
                },
            },
        )
        .collect::<Vec<_>>();

    attempts.extend(
        iv_attempts.into_iter().map(|(mode, iv)| {
            match WzNode::from_img_file_with_iv(path, iv, None) {
                Ok(node) => ProbeAttempt {
                    mode: mode.to_owned(),
                    status: ProbeStatus::Readable,
                    detail: format!("root={}", node.name),
                },
                Err(error) => ProbeAttempt {
                    mode: mode.to_owned(),
                    status: ProbeStatus::Failed,
                    detail: error.to_string(),
                },
            }
        }),
    );

    attempts
}

pub fn list_entries_are_plausible(entries: &[String]) -> bool {
    entries.first().is_some_and(|entry| entry == "dummy")
        && entries.iter().any(|entry| {
            let lower = entry.to_ascii_lowercase();
            lower.contains('/') && lower.ends_with(".img")
        })
}
