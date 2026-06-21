use patch_core::resource::probe::{
    format_magic, probe_client, ListProbe, ProbeAttempt, ProbeReport, ProbeStatus, ResourceProbe,
};

#[test]
fn format_magic_renders_lowercase_hex_pairs() {
    let magic = format_magic(&[0x73, 0xF8, 0xC2, 0x13]);

    assert_eq!(magic, "73 f8 c2 13");
}

#[test]
fn probe_report_serializes_deterministic_json() {
    let report = ProbeReport {
        client_root: "/client".to_owned(),
        executable_exists: true,
        list: ListProbe {
            path: "list.wz".to_owned(),
            exists: true,
            sha256: Some("hash".to_owned()),
            magic: Some("01 02".to_owned()),
            attempts: vec![ProbeAttempt {
                mode: "GMS".to_owned(),
                status: ProbeStatus::Readable,
                detail: "entries=371 first=dummy".to_owned(),
            }],
        },
        resources: vec![ResourceProbe {
            id: "string-eqp".to_owned(),
            path: "Data/String/Eqp.img".to_owned(),
            exists: true,
            sha256: Some("resource-hash".to_owned()),
            magic: Some("73 f8".to_owned()),
            attempts: vec![ProbeAttempt {
                mode: "auto".to_owned(),
                status: ProbeStatus::Failed,
                detail: "Unable to guess version".to_owned(),
            }],
        }],
    };

    let json = serde_json::to_string(&report).unwrap();

    assert_eq!(
        json,
        r#"{"client_root":"/client","executable_exists":true,"list":{"path":"list.wz","exists":true,"sha256":"hash","magic":"01 02","attempts":[{"mode":"GMS","status":"readable","detail":"entries=371 first=dummy"}]},"resources":[{"id":"string-eqp","path":"Data/String/Eqp.img","exists":true,"sha256":"resource-hash","magic":"73 f8","attempts":[{"mode":"auto","status":"failed","detail":"Unable to guess version"}]}]}"#
    );
}

#[test]
fn probe_client_reports_missing_files_without_aborting() {
    let temp = tempfile::tempdir().unwrap();

    let report = probe_client(temp.path()).unwrap();

    assert!(!report.executable_exists);
    assert_eq!(report.list.status(), ProbeStatus::Missing);
    assert_eq!(report.resources.len(), 9);
    assert!(report
        .resources
        .iter()
        .all(|resource| resource.status() == ProbeStatus::Missing));
}
