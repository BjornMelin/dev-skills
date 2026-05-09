use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn help_mentions_capsule_commands() {
    let mut command = Command::cargo_bin("codex-dev").expect("binary");

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("capsule"));
}

#[test]
fn capsule_lifecycle_supports_json_and_markdown() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");

    let mut init = Command::cargo_bin("codex-dev").expect("binary");
    let init_output = init
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "Build capsule CLI",
            "--objective",
            "Create task capsules",
            "--branch",
            "feat/codex-dev-task-capsules",
            "--issue",
            "22",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "test-capsule",
            "--created-at",
            "2026-05-09T04:00:00Z",
            "--status",
            "ready_for_pr",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let init_json: Value = serde_json::from_slice(&init_output).expect("init json");
    assert_eq!(init_json["result"]["capsule"]["status"], "ready_for_pr");
    let path = init_json["result"]["path"].as_str().expect("capsule path");

    let mut validate = Command::cargo_bin("codex-dev").expect("binary");
    let validate_output = validate
        .args(["--json", "capsule", "validate", path])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let validate_json: Value = serde_json::from_slice(&validate_output).expect("validate json");
    assert_eq!(validate_json["result"]["valid"], true);

    let mut render = Command::cargo_bin("codex-dev").expect("binary");
    render
        .args(["capsule", "render", path])
        .assert()
        .success()
        .stdout(predicates::str::contains("# Build capsule CLI"));
}

#[test]
fn capsule_validate_fails_for_invalid_capsules() {
    let temp = tempdir().expect("tempdir");
    let missing = temp.path().join("missing");

    let mut validate = Command::cargo_bin("codex-dev").expect("binary");
    let invalid_output = validate
        .args([
            "--json",
            "capsule",
            "validate",
            missing.to_str().expect("utf8 temp path"),
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let invalid_json: Value = serde_json::from_slice(&invalid_output).expect("invalid json");
    assert_eq!(invalid_json["ok"], false);
    assert_eq!(invalid_json["result"]["valid"], false);
}

#[test]
fn capsule_init_errors_keep_json_envelope() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");

    let args = [
        "--json",
        "capsule",
        "init",
        "--title",
        "Build capsule CLI",
        "--root",
        root.to_str().expect("utf8 temp path"),
        "--id",
        "test-capsule",
        "--created-at",
        "2026-05-09T04:00:00Z",
    ];

    Command::cargo_bin("codex-dev")
        .expect("binary")
        .args(args)
        .assert()
        .success();

    let duplicate_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args(args)
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let duplicate_json: Value =
        serde_json::from_slice(&duplicate_output).expect("duplicate init json");
    assert_eq!(duplicate_json["ok"], false);
    assert!(
        duplicate_json["result"]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("already exists")
    );
}

#[test]
fn policy_manifest_and_dry_run_update_capsule() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");

    let manifest_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "policy",
            "manifest",
            "--generated-at",
            "2026-05-09T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let manifest_json: Value = serde_json::from_slice(&manifest_output).expect("manifest json");
    assert_eq!(manifest_json["command"], "policy manifest");
    assert_eq!(
        manifest_json["result"]["schema"],
        "codex-dev.policy-gates.v1"
    );
    let manifest_gates = manifest_json["result"]["gates"]
        .as_array()
        .expect("manifest gates");
    let docs_gate = manifest_gates
        .iter()
        .find(|gate| gate["id"] == "docs-links")
        .expect("docs-links gate");
    assert_eq!(docs_gate["network"], false);

    let init_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "Policy gate smoke",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "policy-smoke",
            "--created-at",
            "2026-05-09T04:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let init_json: Value = serde_json::from_slice(&init_output).expect("init json");
    let capsule = init_json["result"]["path"].as_str().expect("capsule path");

    let run_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "policy",
            "run",
            "--capsule",
            capsule,
            "--checked-at",
            "2026-05-09T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let run_json: Value = serde_json::from_slice(&run_output).expect("policy run json");
    assert_eq!(run_json["command"], "policy run");
    assert_eq!(run_json["result"]["dry_run"], true);
    let run_gates = run_json["result"]["gates"]
        .as_array()
        .expect("policy run gates");
    assert!(run_gates.iter().all(|gate| gate["status"] == "planned"));
}
