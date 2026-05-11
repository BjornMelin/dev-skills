use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn write_subspawn_plan_fixture(root: &std::path::Path) -> std::path::PathBuf {
    let path = root.join("subspawn-plan.json");
    std::fs::write(
        &path,
        r#"{
  "task": "pre-PR review",
  "mode": "read-only",
  "scope": "branch diff",
  "wait_policy": "strict",
  "rendezvous_required": true,
  "roles": [
    {"name": "reviewer"},
    {"name": "test_runner"}
  ],
  "prompts": [
    {"role": "reviewer", "prompt": "Task: review diff\nRole: reviewer\nReturn format:\n- Status\n- Risks/blockers"},
    {"role": "test_runner", "prompt": "Task: validate diff\nRole: test_runner\nReturn format:\n- Status\n- Risks/blockers"}
  ],
  "registry_issues": [],
  "duplicate_roles_ignored": {
    "test_runner": [
      "skills/subagent-creator/templates/agents/test_runner.toml",
      "skills/subspawn/templates/agents/test_runner.toml"
    ]
  }
}"#,
    )
    .expect("write subspawn fixture");
    path
}

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
    let core_test_gate = manifest_gates
        .iter()
        .find(|gate| gate["id"] == "codex-dev-core-test")
        .expect("codex-dev-core-test gate");
    assert_eq!(
        core_test_gate["command"]
            .as_array()
            .expect("core test command")
            .iter()
            .map(|value| value.as_str().expect("command token must be a string"))
            .collect::<Vec<_>>(),
        vec!["cargo", "test", "-p", "codex-dev-core"]
    );
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
            "2026-05-09T05:00:00.123456789Z",
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

#[test]
fn pr_plan_and_record_support_fixture_mode() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");

    let plan_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "plan",
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "25",
            "--generated-at",
            "2026-05-09T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let plan_json: Value = serde_json::from_slice(&plan_output).expect("plan json");
    assert_eq!(plan_json["command"], "pr plan");
    assert_eq!(
        plan_json["result"]["schema"],
        "codex-dev.pr-control-plan.v1"
    );
    let plan_commands = plan_json["result"]["commands"]
        .as_array()
        .expect("plan commands");
    assert!(plan_commands.iter().any(|command| {
        command["id"] == "gh-review-threads"
            && command["command"]
                .as_array()
                .expect("command argv")
                .iter()
                .any(|part| part == "query=query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){pullRequest(number:$number){reviewThreads(first:100){nodes{id isResolved isOutdated comments(first:10){nodes{id path line originalLine url}}}}}}}")
    }));
    assert!(plan_commands.iter().any(|command| {
        command["id"] == "gh-pr-checks"
            && command["command"]
                .as_array()
                .expect("command argv")
                .iter()
                .any(|part| part == "bucket,completedAt,description,event,link,name,startedAt,state,workflow")
    }));
    assert!(
        plan_commands
            .iter()
            .any(|command| command["id"] == "gh-reviews")
    );
    assert!(
        plan_commands
            .iter()
            .any(|command| command["id"] == "gh-review-comments")
    );

    let init_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "PR control smoke",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "pr-smoke",
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
    let source = temp.path().join("pr-snapshot.json");
    std::fs::write(
        &source,
        r#"{
  "repository": "BjornMelin/dev-skills",
  "number": 25,
  "url": "https://github.com/BjornMelin/dev-skills/pull/25",
  "state": "OPEN",
  "checks": [
    {"name": "GitGuardian", "status": "COMPLETED", "conclusion": "SUCCESS"}
  ],
  "review_threads": {"unresolved": 0}
}"#,
    )
    .expect("write fixture");

    let record_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "record",
            "--capsule",
            capsule,
            "--source",
            source.to_str().expect("utf8 fixture path"),
            "--checked-at",
            "2026-05-09T05:00:00.123456789Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let record_json: Value = serde_json::from_slice(&record_output).expect("record json");
    assert_eq!(record_json["command"], "pr record");
    assert_eq!(record_json["result"]["pr"]["number"], 25);
    assert_eq!(
        record_json["result"]["pr"]["review_threads"]["unresolved"],
        0
    );
    let evidence = std::fs::read_to_string(std::path::Path::new(capsule).join("evidence.jsonl"))
        .expect("evidence");
    let pr_record_entry: Value = serde_json::from_str(
        evidence
            .lines()
            .last()
            .expect("at least one evidence entry"),
    )
    .expect("valid evidence jsonl line");
    assert_eq!(pr_record_entry["schema"], "codex-dev.evidence.v1");
    assert_eq!(pr_record_entry["kind"], "review");
    assert_eq!(pr_record_entry["at"], "2026-05-09T05:00:00.123456789Z");
    let command = pr_record_entry["command"].as_str().expect("command string");
    assert!(command.contains("codex-dev pr record --capsule"));
    assert!(command.contains("--source"));
    assert!(command.contains("--checked-at 2026-05-09T05:00:00.123456789Z"));

    Command::cargo_bin("codex-dev")
        .expect("binary")
        .args(["pr", "status", "--capsule", capsule])
        .assert()
        .success()
        .stdout(predicates::str::contains("BjornMelin/dev-skills#25 open"));
}

#[test]
fn pr_plan_rejects_invalid_repository_names() {
    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "plan",
            "--repo",
            "BjornMelin",
            "--number",
            "25",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let error_json: Value = serde_json::from_slice(&output).expect("error json");
    assert_eq!(error_json["command"], "pr plan");
    assert!(
        error_json["result"]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("OWNER/REPO")
    );
}

#[test]
fn pr_record_cli_normalizes_github_checks_sources() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let init_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "PR checks",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "pr-checks",
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
    let source = temp.path().join("gh-pr-checks.json");
    std::fs::write(
        &source,
        r#"[
  {"bucket": "fail", "completedAt": "2026-05-09T05:01:00Z", "link": "https://example.test/lint", "name": "lint", "state": "FAILURE"},
  {"bucket": "pass", "completedAt": "2026-05-09T05:02:00Z", "link": "https://example.test/test", "name": "test", "state": "SUCCESS"}
]"#,
    )
    .expect("write fixture");

    let record_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "record",
            "--capsule",
            capsule,
            "--source",
            source.to_str().expect("utf8 fixture path"),
            "--source-kind",
            "gh-pr-checks",
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "46",
            "--retrieved-at",
            "2026-05-09T04:59:00Z",
            "--source-command",
            "gh pr checks 46 --json bucket,completedAt,link,name,state",
            "--checked-at",
            "2026-05-09T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let record_json: Value = serde_json::from_slice(&record_output).expect("record json");

    assert_eq!(
        record_json["result"]["pr"]["repository"],
        "BjornMelin/dev-skills"
    );
    assert_eq!(record_json["result"]["pr"]["number"], 46);
    assert_eq!(
        record_json["result"]["pr"]["checks"][0]["status"],
        "completed"
    );
    assert_eq!(
        record_json["result"]["pr"]["checks"][0]["conclusion"],
        "failure"
    );
    assert_eq!(
        record_json["result"]["pr"]["sources"][0]["parser_version"],
        "codex-dev.pr-source-parser.v1"
    );
    assert_eq!(
        record_json["result"]["pr"]["sources"][0]["retrieved_at"],
        "2026-05-09T04:59:00Z"
    );
    assert_eq!(
        record_json["result"]["pr"]["sources"][0]["command"],
        "gh pr checks 46 --json bucket,completedAt,link,name,state"
    );
}

#[test]
fn pr_record_json_error_does_not_repair_missing_pr_contract() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");

    let init_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "PR strict smoke",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "pr-strict-smoke",
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
    let capsule_path = std::path::Path::new(capsule);
    std::fs::remove_file(capsule_path.join("pr.json")).expect("remove pr contract");
    let capsule_before =
        std::fs::read_to_string(capsule_path.join("capsule.json")).expect("capsule before");
    let evidence_before =
        std::fs::read_to_string(capsule_path.join("evidence.jsonl")).expect("evidence before");

    let source = temp.path().join("pr-snapshot.json");
    std::fs::write(
        &source,
        r#"{
  "repository": "BjornMelin/dev-skills",
  "number": 25,
  "state": "OPEN",
  "review_threads": {"unresolved": 0}
}"#,
    )
    .expect("write fixture");

    let record_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "record",
            "--capsule",
            capsule,
            "--source",
            source.to_str().expect("utf8 fixture path"),
            "--checked-at",
            "2026-05-09T05:00:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let record_json: Value = serde_json::from_slice(&record_output).expect("record error json");
    assert_eq!(record_json["ok"], false);
    assert_eq!(record_json["command"], "pr record");
    assert!(
        record_json["result"]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("missing required file: pr.json")
    );
    assert!(!capsule_path.join("pr.json").exists());
    assert_eq!(
        std::fs::read_to_string(capsule_path.join("capsule.json")).expect("capsule after"),
        capsule_before
    );
    assert_eq!(
        std::fs::read_to_string(capsule_path.join("evidence.jsonl")).expect("evidence after"),
        evidence_before
    );
}

#[test]
fn evidence_append_records_typed_entries_and_status_counts() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");

    let init_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "Evidence append smoke",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "evidence-smoke",
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

    let append_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "evidence",
            "append",
            "--capsule",
            capsule,
            "--kind",
            "decision",
            "--summary",
            "Use one typed append command",
            "--at",
            "2026-05-09T06:00:00Z",
            "--source-id",
            "issue:42",
            "--actor",
            "codex",
            "--tool",
            "codex-dev",
            "--confidence",
            "95",
            "--residual-risk",
            "future PR normalizers still need fixtures",
            "--artifact",
            "docs/reference/codex-dev-cli.md",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let append_json: Value = serde_json::from_slice(&append_output).expect("append json");
    assert_eq!(append_json["command"], "evidence append");
    assert_eq!(append_json["result"]["record"]["kind"], "decision");
    assert_eq!(append_json["result"]["record"]["source_ids"][0], "issue:42");
    assert_eq!(append_json["result"]["record"]["confidence"], 95);
    assert_eq!(append_json["result"]["evidence"]["total"], 2);

    Command::cargo_bin("codex-dev")
        .expect("binary")
        .args(["capsule", "status", capsule])
        .assert()
        .success()
        .stdout(predicates::str::contains("evidence: decision=1"))
        .stdout(predicates::str::contains("manual=1"));

    let status_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args(["--json", "capsule", "status", capsule])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status_json: Value = serde_json::from_slice(&status_output).expect("status json");
    assert_eq!(status_json["result"]["evidence"]["total"], 2);
    let decision = status_json["result"]["evidence"]["by_kind"]
        .as_array()
        .expect("by kind")
        .iter()
        .find(|kind| kind["kind"] == "decision")
        .expect("decision summary");
    assert_eq!(decision["latest_summary"], "Use one typed append command");
}

#[test]
fn subagents_record_plan_outcome_and_synthesis() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let plan = write_subspawn_plan_fixture(temp.path());

    let init_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "Subagent evidence smoke",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "subagent-smoke",
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

    let plan_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "subagents",
            "record-plan",
            "--capsule",
            capsule,
            "--batch-id",
            "pre-pr-review",
            "--source",
            plan.to_str().expect("utf8 plan path"),
            "--command",
            "python3 skills/subspawn/scripts/subspawn_plan.py plan --preset review --json",
            "--recorded-at",
            "2026-05-09T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let plan_json: Value = serde_json::from_slice(&plan_output).expect("plan json");
    assert_eq!(plan_json["command"], "subagents record-plan");
    assert_eq!(
        plan_json["result"]["batch"]["agents"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        plan_json["result"]["batch"]["prompts"][0]["prompt_hash"]
            .as_str()
            .expect("prompt hash")
            .starts_with("sha256:")
    );
    assert!(
        plan_json["result"]["batch"]["prompts"][0]
            .get("prompt")
            .is_none(),
        "raw prompt text must not be returned in batch records"
    );
    assert_eq!(
        plan_json["result"]["batch"]["duplicate_roles_ignored"]["test_runner"]
            .as_array()
            .expect("duplicate paths")
            .len(),
        2
    );

    let outcome_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "subagents",
            "record-outcome",
            "--capsule",
            capsule,
            "--batch-id",
            "pre-pr-review",
            "--role",
            "reviewer",
            "--status",
            "completed",
            "--summary",
            "no blocking findings",
            "--disposition",
            "accepted",
            "--human-verified",
            "--source-id",
            "reviewer:1",
            "--artifact",
            "review-notes.md",
            "--recorded-at",
            "2026-05-09T05:10:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let outcome_json: Value = serde_json::from_slice(&outcome_output).expect("outcome json");
    assert_eq!(outcome_json["command"], "subagents record-outcome");
    assert_eq!(outcome_json["result"]["agent"]["status"], "completed");
    assert_eq!(outcome_json["result"]["agent"]["disposition"], "accepted");
    assert_eq!(outcome_json["result"]["agent"]["human_verified"], true);

    let synthesis_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "subagents",
            "record-synthesis",
            "--capsule",
            capsule,
            "--batch-id",
            "pre-pr-review",
            "--status",
            "partial",
            "--summary",
            "reviewer completed; test_runner still pending",
            "--human-verified",
            "--source-id",
            "synthesis:pre-pr-review",
            "--artifact",
            "review-summary.md",
            "--recorded-at",
            "2026-05-09T05:20:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let synthesis_json: Value = serde_json::from_slice(&synthesis_output).expect("synthesis json");
    assert_eq!(synthesis_json["command"], "subagents record-synthesis");
    assert_eq!(synthesis_json["result"]["synthesis"]["status"], "partial");
    assert_eq!(synthesis_json["result"]["evidence"]["total"], 4);

    let subagents: Value = serde_json::from_str(
        &std::fs::read_to_string(std::path::Path::new(capsule).join("subagents.json"))
            .expect("subagents"),
    )
    .expect("subagents json");
    assert_eq!(subagents["batches"][0]["id"], "pre-pr-review");
    assert_eq!(subagents["batches"][0]["synthesis"]["status"], "partial");
    assert!(
        subagents["batches"][0]["prompts"][0]
            .get("prompt")
            .is_none(),
        "raw prompt text must not be persisted in subagents.json"
    );
}

#[test]
fn evidence_append_json_errors_are_typed_and_do_not_write() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");

    let init_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "Evidence invalid smoke",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "evidence-invalid-smoke",
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
    let evidence_path = std::path::Path::new(capsule).join("evidence.jsonl");
    let evidence_before = std::fs::read_to_string(&evidence_path).expect("evidence before");

    let append_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "evidence",
            "append",
            "--capsule",
            capsule,
            "--kind",
            "ci",
            "--summary",
            "",
            "--at",
            "2026-05-09T06:00:00Z",
            "--exit-code",
            "1",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let append_json: Value = serde_json::from_slice(&append_output).expect("append error json");
    assert_eq!(append_json["ok"], false);
    assert_eq!(append_json["command"], "evidence append");
    let message = append_json["result"]["error"]["message"]
        .as_str()
        .expect("message");
    assert!(message.contains("summary must not be empty"));
    assert!(message.contains("exit_code requires command"));
    assert_eq!(
        std::fs::read_to_string(&evidence_path).expect("evidence after"),
        evidence_before
    );
}
