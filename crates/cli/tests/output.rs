use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

#[test]
fn empty_scan_writes_only_json_to_stdout() {
    let temp = TempDir::new().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_planetes-scan"))
        .args(["--store", "epic", "--epic-manifests"])
        .arg(temp.path())
        .arg("--strict")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["games"], serde_json::json!([]));
    assert_eq!(report["warnings"], serde_json::json!([]));
}

#[test]
fn strict_scan_preserves_json_report_and_returns_two_on_warning() {
    let temp = TempDir::new().unwrap();
    for strict in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_planetes-scan"));
        command
            .args(["--store", "epic", "--epic-manifests"])
            .arg(temp.path().join("missing"));
        if strict {
            command.arg("--strict");
        }
        let output = command.output().unwrap();
        assert_eq!(output.status.code(), Some(if strict { 2 } else { 0 }));
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["warnings"].as_array().unwrap().len(), 1);
    }
}
