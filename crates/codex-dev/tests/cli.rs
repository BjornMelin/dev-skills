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
fn codex_dev_generates_shell_completion() {
    Command::cargo_bin("codex-dev")
        .expect("binary")
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicates::str::contains("codex-dev"))
        .stdout(predicates::str::contains("capsule"));
}

#[test]
fn codex_dev_generates_manpage() {
    Command::cargo_bin("codex-dev")
        .expect("binary")
        .arg("manpage")
        .assert()
        .success()
        .stdout(predicates::str::contains("codex-dev"))
        // The roff renderer escapes hyphens in command names as `\-`.
        .stdout(predicates::str::contains("codex\\-dev\\-capsule"));
}

fn init_capsule_fixture(root: &std::path::Path, id: &str, title: &str) -> String {
    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            title,
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            id,
            "--created-at",
            "2026-05-09T04:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("init json");
    json["result"]["path"]
        .as_str()
        .expect("capsule path")
        .to_string()
}

fn write_pr_agent_source_fixtures(source_dir: &std::path::Path, number: u64) {
    std::fs::create_dir_all(source_dir).expect("source dir");
    std::fs::write(
        source_dir.join("gh-pr-view.json"),
        format!(
            r#"{{
  "number": {number},
  "url": "https://github.com/BjornMelin/dev-skills/pull/{number}",
  "state": "OPEN",
  "isDraft": false,
  "mergeable": "MERGEABLE",
  "mergeStateStatus": "CLEAN",
  "reviewDecision": "APPROVED",
  "headRefOid": "abc123",
  "headRefName": "feature",
  "baseRefName": "main",
  "baseRefOid": "base123",
  "statusCheckRollup": [],
  "labels": [{{"name": "ready"}}]
}}"#
        ),
    )
    .expect("write pr view");
    std::fs::write(
        source_dir.join("gh-pr-checks.json"),
        r#"[
  {"bucket": "pass", "completedAt": "2026-05-09T05:01:00Z", "link": "https://example.test/ci", "name": "ci", "state": "SUCCESS"}
]"#,
    )
    .expect("write checks");
    std::fs::write(
        source_dir.join("gh-reviews.json"),
        r#"[
  {"id": 1, "user": {"login": "coderabbitai"}, "state": "APPROVED", "submitted_at": "2026-05-09T05:00:00Z"}
]"#,
    )
    .expect("write reviews");
    std::fs::write(source_dir.join("gh-review-comments.json"), "[]").expect("write comments");
    std::fs::write(
        source_dir.join("gh-review-threads.json"),
        r#"[
  {
    "data": {
      "repository": {
        "pullRequest": {
          "reviewThreads": {
            "nodes": [
              {"id": "resolved", "isResolved": true, "isOutdated": false}
            ],
            "pageInfo": {"hasNextPage": false, "endCursor": null}
          }
        }
      }
    }
  }
]"#,
    )
    .expect("write threads");
    std::fs::write(
        source_dir.join("gh-rate-limit.json"),
        r#"{"resources":{"core":{"limit":5000,"remaining":4999,"reset":1770000000}}}"#,
    )
    .expect("write rate limit");
}

fn write_pr_view_fixture(
    source_dir: &std::path::Path,
    number: u64,
    is_draft: bool,
    mergeable: &str,
    merge_state_status: &str,
    review_decision: &str,
) {
    std::fs::write(
        source_dir.join("gh-pr-view.json"),
        format!(
            r#"{{
  "number": {number},
  "url": "https://github.com/BjornMelin/dev-skills/pull/{number}",
  "state": "OPEN",
  "isDraft": {is_draft},
  "mergeable": "{mergeable}",
  "mergeStateStatus": "{merge_state_status}",
  "reviewDecision": "{review_decision}",
  "headRefOid": "abc123",
  "headRefName": "feature",
  "baseRefName": "main",
  "baseRefOid": "base123",
  "statusCheckRollup": [],
  "labels": [{{"name": "ready"}}]
}}"#
        ),
    )
    .expect("write pr view");
}

fn write_pr_checks_fixture(source_dir: &std::path::Path, bucket: &str, state: &str, link: &str) {
    std::fs::write(
        source_dir.join("gh-pr-checks.json"),
        format!(
            r#"[
  {{"bucket": "{bucket}", "completedAt": "2026-05-09T05:01:00Z", "link": "{link}", "name": "ci", "state": "{state}"}}
]"#
        ),
    )
    .expect("write checks");
}

fn write_review_threads_fixture(source_dir: &std::path::Path, unresolved: bool) {
    let is_resolved = if unresolved { "false" } else { "true" };
    std::fs::write(
        source_dir.join("gh-review-threads.json"),
        format!(
            r#"[
  {{
    "data": {{
      "repository": {{
        "pullRequest": {{
          "reviewThreads": {{
            "nodes": [
              {{"id": "thread-1", "isResolved": {is_resolved}, "isOutdated": false}}
            ],
            "pageInfo": {{"hasNextPage": false, "endCursor": null}}
          }}
        }}
      }}
    }}
  }}
]"#
        ),
    )
    .expect("write threads");
}

#[cfg(unix)]
fn write_fake_gh(bin_dir: &std::path::Path) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let script = bin_dir.join("gh");
    std::fs::write(
        &script,
        r#"#!/bin/sh
printf '%s\n' "$*" >> "$GH_LOG"
fixture_root="$GH_FIXTURES"
if [ -n "$GH_REFRESH_FIXTURES" ] && [ -f "$GH_LOG.refresh" ]; then
  fixture_root="$GH_REFRESH_FIXTURES"
fi
case "$*" in
  pr\ view*)
    count_file="$GH_LOG.pr_view_count"
    count=0
    if [ -f "$count_file" ]; then
      count="$(cat "$count_file")"
    fi
    count=$((count + 1))
    printf '%s\n' "$count" > "$count_file"
    if [ -n "$GH_REFRESH_FIXTURES" ] && [ "$count" -gt 1 ]; then
      touch "$GH_LOG.refresh"
      fixture_root="$GH_REFRESH_FIXTURES"
    fi
    cat "$fixture_root/gh-pr-view.json"
    ;;
  pr\ checks*) cat "$fixture_root/gh-pr-checks.json" ;;
  api\ --paginate\ --slurp\ repos/*/pulls/*/reviews*) cat "$fixture_root/gh-reviews.json" ;;
  api\ --paginate\ --slurp\ repos/*/pulls/*/comments*) cat "$fixture_root/gh-review-comments.json" ;;
  api\ graphql*)
    if [ -n "$FAIL_THREADS" ]; then
      echo "thread capture failed" >&2
      exit 2
    fi
    cat "$fixture_root/gh-review-threads.json"
    ;;
  api\ rate_limit*) cat "$fixture_root/gh-rate-limit.json" ;;
  api\ --paginate\ --slurp\ repos/*/issues/*/comments*)
    if [ -n "$DUPLICATE_MARKER" ]; then
      printf '[{"id":99,"html_url":"https://example.test/duplicate","body":"<!-- %s -->"}]\n' "$DUPLICATE_MARKER"
    else
      echo '[]'
    fi
    ;;
  api\ --method\ POST\ repos/*/issues/*/comments*) echo '{"id":123,"html_url":"https://example.test/comment"}' ;;
  api\ --method\ POST\ repos/*/pulls/*/comments/*/replies*) echo '{"id":124,"html_url":"https://example.test/reply"}' ;;
  api\ --method\ POST\ repos/*/actions/runs/*/rerun-failed-jobs*) echo '{"ok":true}' ;;
  pr\ merge*) echo '{"merged":true}' ;;
  api\ repos/*/actions/runs/*)
    run_id="$(printf '%s\n' "$*" | sed -n 's#.*actions/runs/\([0-9][0-9]*\).*#\1#p')"
    test -n "$run_id" || run_id=456
    conclusion="${RUN_CONCLUSION:-success}"
    status="${RUN_STATUS:-completed}"
    event="${RUN_EVENT:-pull_request}"
    repository="${RUN_REPOSITORY:-BjornMelin/dev-skills}"
    head_repository="${RUN_HEAD_REPOSITORY:-BjornMelin/dev-skills}"
    head_branch="${RUN_HEAD_BRANCH:-feature}"
    fork="${RUN_FORK:-false}"
    pull_requests="${RUN_PULL_REQUESTS:-}"
    if [ -z "$pull_requests" ]; then
      pull_requests='[{"number":48},{"number":49}]'
    fi
    printf '{"id":%s,"head_sha":"abc123","head_branch":"%s","event":"%s","status":"%s","conclusion":"%s","html_url":"https://github.com/%s/actions/runs/%s","repository":{"full_name":"%s"},"head_repository":{"full_name":"%s","fork":%s},"pull_requests":%s}\n' "$run_id" "$head_branch" "$event" "$status" "$conclusion" "$repository" "$run_id" "$repository" "$head_repository" "$fork" "$pull_requests"
    ;;
  issue\ edit*) echo '{"ok":true}' ;;
  *) echo "unexpected gh args: $*" >&2; exit 2 ;;
esac
"#,
    )
    .expect("fake gh");
    let mut permissions = std::fs::metadata(&script)
        .expect("fake gh metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&script, permissions).expect("fake gh executable");
    script
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
                .any(|part| part == "--paginate")
            && command["command"]
                .as_array()
                .expect("command argv")
                .iter()
                .any(|part| part == "--slurp")
            && command["command"]
                .as_array()
                .expect("command argv")
                .iter()
                .any(|part| {
                    part.as_str().is_some_and(|part| {
                        part.contains("endCursor") && part.contains("after:$endCursor")
                    })
                })
    }));
    assert!(plan_commands.iter().any(|command| {
        command["id"] == "gh-pr-checks"
            && command["command"]
                .as_array()
                .expect("command argv")
                .iter()
                .any(|part| {
                    part == "bucket,completedAt,description,event,link,name,startedAt,state,workflow"
                })
    }));
    assert!(plan_commands.iter().any(|command| {
        command["id"] == "gh-pr-view"
            && command["command"]
                .as_array()
                .expect("command argv")
                .iter()
                .any(|part| {
                    part == "number,url,state,isDraft,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup,headRefOid,headRefName,baseRefName,baseRefOid,updatedAt,labels"
                })
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
fn pr_agent_replays_sources_records_state_and_recommendations() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    std::fs::create_dir_all(&source_dir).expect("source dir");

    let init_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "PR agent state",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "pr-agent-state",
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

    std::fs::write(
        source_dir.join("gh-pr-view.json"),
        r#"{
  "number": 47,
  "url": "https://github.com/BjornMelin/dev-skills/pull/47",
  "state": "OPEN",
  "isDraft": false,
  "mergeable": "MERGEABLE",
  "reviewDecision": "APPROVED",
  "headRefOid": "abc123",
  "statusCheckRollup": []
}"#,
    )
    .expect("write pr view");
    std::fs::write(
        source_dir.join("gh-pr-checks.json"),
        r#"[
  {"bucket": "pass", "completedAt": "2026-05-09T05:01:00Z", "link": "https://example.test/ci", "name": "ci", "state": "SUCCESS"}
]"#,
    )
    .expect("write checks");
    std::fs::write(
        source_dir.join("gh-reviews.json"),
        r#"[
  {"id": 1, "user": {"login": "coderabbitai"}, "state": "APPROVED", "submitted_at": "2026-05-09T05:00:00Z"}
]"#,
    )
    .expect("write reviews");
    std::fs::write(source_dir.join("gh-review-comments.json"), "[]").expect("write comments");
    std::fs::write(
        source_dir.join("gh-review-threads.json"),
        r#"[
  {
    "data": {
      "repository": {
        "pullRequest": {
          "reviewThreads": {
            "nodes": [
              {"id": "resolved", "isResolved": true, "isOutdated": false}
            ],
            "pageInfo": {"hasNextPage": false, "endCursor": null}
          }
        }
      }
    }
  }
]"#,
    )
    .expect("write threads");
    std::fs::write(
        source_dir.join("gh-rate-limit.json"),
        r#"{"resources":{"core":{"limit":5000,"remaining":4999,"reset":1770000000}}}"#,
    )
    .expect("write rate limit");

    let agent_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "agent",
            "--capsule",
            capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "47",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let agent_json: Value = serde_json::from_slice(&agent_output).expect("agent json");
    assert_eq!(agent_json["command"], "pr agent");
    assert_eq!(
        agent_json["result"]["schema"],
        "codex-dev.pr-agent-state.v1"
    );
    assert_eq!(agent_json["result"]["dry_run"], true);
    assert_eq!(agent_json["result"]["pr"]["number"], 47);
    assert_eq!(
        agent_json["result"]["pr"]["review_threads"]["authoritative"],
        true
    );
    assert!(
        agent_json["result"]["actions"]
            .as_array()
            .expect("actions")
            .iter()
            .any(|action| action["id"] == "merge_when_policy_allows")
    );
    assert!(
        agent_json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["source"] == "gh-rate-limit")
    );

    let capsule_path = std::path::Path::new(capsule);
    assert!(capsule_path.join("pr-agent-state.json").is_file());
    let pr_json: Value = serde_json::from_str(
        &std::fs::read_to_string(capsule_path.join("pr.json")).expect("pr json"),
    )
    .expect("pr json parse");
    assert!(
        pr_json["sources"]
            .as_array()
            .expect("sources")
            .iter()
            .any(|source| source["kind"] == "gh-review-threads")
    );
    let evidence = std::fs::read_to_string(capsule_path.join("evidence.jsonl")).expect("evidence");
    assert!(evidence.contains("PR agent dry-run state recorded"));

    std::fs::write(
        source_dir.join("gh-review-threads.json"),
        r#"[
  {
    "data": {
      "repository": {
        "pullRequest": {
          "reviewThreads": {
            "nodes": [
              {"id": "current", "isResolved": false, "isOutdated": false}
            ],
            "pageInfo": {"hasNextPage": true, "endCursor": "next"}
          }
        }
      }
    }
  }
]"#,
    )
    .expect("write incomplete threads");
    let incomplete_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "agent",
            "--capsule",
            capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "47",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--checked-at",
            "2026-05-09T05:06:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let incomplete_json: Value =
        serde_json::from_slice(&incomplete_output).expect("incomplete json");
    assert!(
        incomplete_json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| {
                diagnostic["source"] == "gh-review-threads"
                    && diagnostic["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("pagination"))
            })
    );
    assert!(
        incomplete_json["result"]["actions"]
            .as_array()
            .expect("actions")
            .iter()
            .any(|action| action["id"] == "refresh_review_threads")
    );
}

#[test]
fn pr_agent_action_dry_run_plans_hosted_write_without_apply() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    write_pr_agent_source_fixtures(&source_dir, 48);
    let capsule = init_capsule_fixture(&root, "pr-agent-action", "PR hosted action");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "agent-action",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "48",
            "--plan-id",
            "reply-cr-001",
            "--action",
            "reply-review-comment",
            "--review-comment-id",
            "12345",
            "--body",
            "Verified against current code; this is stale.",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("action json");
    assert_eq!(json["command"], "pr agent-action");
    assert_eq!(
        json["result"]["schema"],
        "codex-dev.pr-agent-hosted-action.v1"
    );
    assert_eq!(json["result"]["dry_run"], true);
    assert_eq!(json["result"]["apply_requested"], false);
    assert_eq!(json["result"]["action"]["kind"], "reply-review-comment");
    assert!(
        json["result"]["action"]["command"]
            .as_array()
            .expect("command")
            .iter()
            .any(|arg| {
                arg.as_str()
                    .is_some_and(|arg| arg.contains("codex-dev-pr-agent:sha256:"))
            })
    );

    let action_dir =
        std::path::Path::new(json["result"]["action_dir"].as_str().expect("action dir"));
    assert!(action_dir.join("plan.json").is_file());
    assert!(action_dir.join("before-state.json").is_file());
    assert!(!action_dir.join("after-state.json").exists());
    let evidence = std::fs::read_to_string(std::path::Path::new(&capsule).join("evidence.jsonl"))
        .expect("evidence");
    assert!(evidence.contains("PR agent hosted action reply-cr-001"));
}

#[test]
fn pr_agent_action_rejects_apply_with_replay_sources() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    write_pr_agent_source_fixtures(&source_dir, 48);
    let capsule = init_capsule_fixture(&root, "pr-agent-action-apply-replay", "PR hosted action");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "agent-action",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "48",
            "--plan-id",
            "unsafe-replay",
            "--action",
            "post-issue-comment",
            "--body",
            "This must not post from replayed state.",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--checked-at",
            "2026-05-09T05:05:00Z",
            "--apply",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("error json");
    assert_eq!(json["command"], "pr agent-action");
    assert!(
        json["result"]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("--apply must capture live state")
    );
}

#[cfg(unix)]
#[test]
fn pr_agent_action_apply_uses_live_gh_and_records_before_after_state() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let fixtures = temp.path().join("fixtures");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    write_pr_agent_source_fixtures(&fixtures, 48);
    write_fake_gh(&bin);
    let log = temp.path().join("gh.log");
    let capsule = init_capsule_fixture(&root, "pr-agent-action-apply", "PR hosted action apply");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let test_path = format!("{}:{old_path}", bin.display());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", test_path)
        .env("GH_FIXTURES", &fixtures)
        .env("GH_LOG", &log)
        .env("GH_TOKEN", "test-token")
        .args([
            "--json",
            "pr",
            "agent-action",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "48",
            "--plan-id",
            "issue-comment-001",
            "--action",
            "post-issue-comment",
            "--body",
            "Posting verified evidence.",
            "--checked-at",
            "2026-05-09T05:06:00Z",
            "--apply",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("action json");
    assert_eq!(json["result"]["dry_run"], false);
    assert_eq!(json["result"]["execution"]["status"], "applied");
    let action_dir =
        std::path::Path::new(json["result"]["action_dir"].as_str().expect("action dir"));
    assert!(action_dir.join("plan.json").is_file());
    assert!(action_dir.join("before-state.json").is_file());
    assert!(action_dir.join("after-state.json").is_file());

    let gh_log = std::fs::read_to_string(log).expect("gh log");
    assert!(gh_log.contains("pr view 48 --repo BjornMelin/dev-skills"));
    assert!(gh_log.contains(
        "api --paginate --slurp repos/BjornMelin/dev-skills/issues/48/comments?per_page=100"
    ));
    assert!(gh_log.contains("api --method POST repos/BjornMelin/dev-skills/issues/48/comments"));
    let evidence = std::fs::read_to_string(std::path::Path::new(&capsule).join("evidence.jsonl"))
        .expect("evidence");
    assert!(evidence.contains("PR agent hosted action issue-comment-001"));
    assert!(evidence.contains("Applied"));
}

#[cfg(unix)]
#[test]
fn pr_agent_action_apply_replies_to_review_comment_path() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let fixtures = temp.path().join("fixtures");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    write_pr_agent_source_fixtures(&fixtures, 48);
    write_fake_gh(&bin);
    let log = temp.path().join("gh.log");
    let capsule = init_capsule_fixture(&root, "pr-agent-action-review-reply", "PR review reply");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let test_path = format!("{}:{old_path}", bin.display());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", test_path)
        .env("GH_FIXTURES", &fixtures)
        .env("GH_LOG", &log)
        .env("GH_TOKEN", "test-token")
        .args([
            "--json",
            "pr",
            "agent-action",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "48",
            "--plan-id",
            "review-reply-001",
            "--action",
            "reply-review-comment",
            "--review-comment-id",
            "98765",
            "--body",
            "Verified against current code.",
            "--checked-at",
            "2026-05-09T05:05:00Z",
            "--apply",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("reply json");
    assert_eq!(json["result"]["execution"]["status"], "applied");
    let gh_log = std::fs::read_to_string(log).expect("gh log");
    assert!(gh_log.contains(
        "api --paginate --slurp repos/BjornMelin/dev-skills/pulls/48/comments?per_page=100"
    ));
    assert!(
        gh_log.contains(
            "api --method POST repos/BjornMelin/dev-skills/pulls/48/comments/98765/replies"
        )
    );
}

#[cfg(unix)]
#[test]
fn pr_agent_action_apply_skips_duplicate_comment_marker() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let fixtures = temp.path().join("fixtures");
    let source_dir = temp.path().join("sources");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    write_pr_agent_source_fixtures(&fixtures, 48);
    write_pr_agent_source_fixtures(&source_dir, 48);
    write_fake_gh(&bin);
    let log = temp.path().join("gh.log");
    let capsule = init_capsule_fixture(
        &root,
        "pr-agent-action-duplicate",
        "PR hosted action duplicate",
    );

    let dry_run = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "agent-action",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "48",
            "--plan-id",
            "issue-comment-dup",
            "--action",
            "post-issue-comment",
            "--body",
            "Posting verified evidence.",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let dry_run_json: Value = serde_json::from_slice(&dry_run).expect("dry run json");
    let marker = dry_run_json["result"]["action"]["idempotency_key"]
        .as_str()
        .expect("idempotency key");

    let old_path = std::env::var("PATH").unwrap_or_default();
    let test_path = format!("{}:{old_path}", bin.display());
    let apply = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", test_path)
        .env("GH_FIXTURES", &fixtures)
        .env("GH_LOG", &log)
        .env("GH_TOKEN", "test-token")
        .env("DUPLICATE_MARKER", marker)
        .args([
            "--json",
            "pr",
            "agent-action",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "48",
            "--plan-id",
            "issue-comment-dup",
            "--action",
            "post-issue-comment",
            "--body",
            "Posting verified evidence.",
            "--checked-at",
            "2026-05-09T05:06:00Z",
            "--apply",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let apply_json: Value = serde_json::from_slice(&apply).expect("apply json");
    assert_eq!(
        apply_json["result"]["execution"]["status"],
        "skipped_duplicate"
    );
    assert_eq!(
        apply_json["result"]["execution"]["duplicate_of"],
        "https://example.test/duplicate"
    );
    let gh_log = std::fs::read_to_string(log).expect("gh log");
    assert!(gh_log.contains(
        "api --paginate --slurp repos/BjornMelin/dev-skills/issues/48/comments?per_page=100"
    ));
    assert!(!gh_log.contains("api --method POST repos/BjornMelin/dev-skills/issues/48/comments"));
}

#[cfg(unix)]
#[test]
fn pr_agent_action_apply_blocks_failed_before_state_capture() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let fixtures = temp.path().join("fixtures");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    write_pr_agent_source_fixtures(&fixtures, 48);
    write_fake_gh(&bin);
    let log = temp.path().join("gh.log");
    let capsule = init_capsule_fixture(&root, "pr-agent-action-blocked", "PR hosted action block");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let test_path = format!("{}:{old_path}", bin.display());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", test_path)
        .env("GH_FIXTURES", &fixtures)
        .env("GH_LOG", &log)
        .env("GH_TOKEN", "test-token")
        .env("FAIL_THREADS", "1")
        .args([
            "--json",
            "pr",
            "agent-action",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "48",
            "--plan-id",
            "issue-comment-blocked",
            "--action",
            "post-issue-comment",
            "--body",
            "This must not post when state capture fails.",
            "--checked-at",
            "2026-05-09T05:05:00Z",
            "--apply",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("blocked json");
    assert_eq!(json["result"]["execution"]["status"], "failed");
    assert!(
        json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| {
                diagnostic["source"] == "pr-agent-preflight"
                    && diagnostic["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("before-state capture"))
            })
    );
    let gh_log = std::fs::read_to_string(log).expect("gh log");
    assert!(!gh_log.contains("api --method POST repos/BjornMelin/dev-skills/issues/48/comments"));
}

#[cfg(unix)]
#[test]
fn pr_agent_action_apply_skips_already_resolved_thread() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let fixtures = temp.path().join("fixtures");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    write_pr_agent_source_fixtures(&fixtures, 48);
    write_fake_gh(&bin);
    let log = temp.path().join("gh.log");
    let capsule = init_capsule_fixture(&root, "pr-agent-action-thread", "PR hosted action thread");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let test_path = format!("{}:{old_path}", bin.display());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", test_path)
        .env("GH_FIXTURES", &fixtures)
        .env("GH_LOG", &log)
        .env("GH_TOKEN", "test-token")
        .args([
            "--json",
            "pr",
            "agent-action",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "48",
            "--plan-id",
            "resolve-thread-001",
            "--action",
            "resolve-review-thread",
            "--thread-id",
            "resolved",
            "--checked-at",
            "2026-05-09T05:05:00Z",
            "--apply",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("thread json");
    assert_eq!(json["result"]["execution"]["status"], "skipped_duplicate");
    let gh_log = std::fs::read_to_string(log).expect("gh log");
    assert!(!gh_log.contains("resolveReviewThread"));
}

#[cfg(unix)]
#[test]
fn pr_agent_action_apply_skips_existing_label() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let fixtures = temp.path().join("fixtures");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    write_pr_agent_source_fixtures(&fixtures, 48);
    write_fake_gh(&bin);
    let log = temp.path().join("gh.log");
    let capsule = init_capsule_fixture(&root, "pr-agent-action-label", "PR hosted action label");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let test_path = format!("{}:{old_path}", bin.display());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", test_path)
        .env("GH_FIXTURES", &fixtures)
        .env("GH_LOG", &log)
        .env("GH_TOKEN", "test-token")
        .args([
            "--json",
            "pr",
            "agent-action",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "48",
            "--plan-id",
            "label-ready-001",
            "--action",
            "add-labels",
            "--label",
            "ready",
            "--checked-at",
            "2026-05-09T05:05:00Z",
            "--apply",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("label json");
    assert_eq!(json["result"]["execution"]["status"], "skipped_duplicate");
    let gh_log = std::fs::read_to_string(log).expect("gh log");
    assert!(!gh_log.contains("issue edit"));
}

#[cfg(unix)]
#[test]
fn pr_agent_action_apply_skips_non_failed_workflow_rerun() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let fixtures = temp.path().join("fixtures");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    write_pr_agent_source_fixtures(&fixtures, 48);
    write_fake_gh(&bin);
    let log = temp.path().join("gh.log");
    let capsule = init_capsule_fixture(&root, "pr-agent-action-rerun", "PR hosted action rerun");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let test_path = format!("{}:{old_path}", bin.display());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", test_path)
        .env("GH_FIXTURES", &fixtures)
        .env("GH_LOG", &log)
        .env("GH_TOKEN", "test-token")
        .env("RUN_CONCLUSION", "success")
        .args([
            "--json",
            "pr",
            "agent-action",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "48",
            "--plan-id",
            "rerun-001",
            "--action",
            "rerun-failed-jobs",
            "--run-id",
            "456",
            "--checked-at",
            "2026-05-09T05:05:00Z",
            "--apply",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("rerun json");
    assert_eq!(json["result"]["execution"]["status"], "skipped_duplicate");
    let gh_log = std::fs::read_to_string(log).expect("gh log");
    assert!(gh_log.contains("api repos/BjornMelin/dev-skills/actions/runs/456"));
    assert!(!gh_log.contains("rerun-failed-jobs"));
}

#[test]
fn pr_readiness_replays_green_state_and_plans_merge_report() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    write_pr_agent_source_fixtures(&source_dir, 49);
    let capsule = init_capsule_fixture(&root, "pr-readiness-green", "PR readiness green");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--poll-attempts",
            "1",
            "--poll-interval-seconds",
            "0",
            "--merge",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("readiness json");
    assert_eq!(json["command"], "pr readiness");
    assert_eq!(json["result"]["schema"], "codex-dev.pr-agent-readiness.v1");
    assert_eq!(json["result"]["final_status"], "ready");
    assert_eq!(json["result"]["ready"], true);
    assert_eq!(json["result"]["actions"][0]["kind"], "merge");
    assert_eq!(json["result"]["actions"][0]["status"], "planned");
    let capsule_path = std::path::Path::new(&capsule);
    assert!(capsule_path.join("pr-readiness.json").is_file());
    let markdown = std::fs::read_to_string(capsule_path.join("pr-readiness.md")).expect("markdown");
    assert!(markdown.contains("# PR Readiness: BjornMelin/dev-skills#49"));
    assert!(markdown.contains("- Status: Ready"));
    let evidence = std::fs::read_to_string(capsule_path.join("evidence.jsonl")).expect("evidence");
    assert!(evidence.contains("PR readiness for BjornMelin/dev-skills#49 finished as Ready"));
}

#[test]
fn pr_readiness_blocks_missing_pr_identity_and_still_writes_report() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    write_pr_agent_source_fixtures(&source_dir, 49);
    std::fs::write(
        source_dir.join("gh-pr-view.json"),
        r#"{
  "number": 49,
  "url": "https://github.com/BjornMelin/dev-skills/pull/49",
  "state": "OPEN",
  "isDraft": false,
  "mergeable": "MERGEABLE",
  "mergeStateStatus": "CLEAN",
  "reviewDecision": "APPROVED",
  "statusCheckRollup": [],
  "labels": [{"name": "ready"}]
}"#,
    )
    .expect("write pr view without identity");
    let capsule = init_capsule_fixture(
        &root,
        "pr-readiness-missing-identity",
        "PR readiness missing identity",
    );

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--poll-interval-seconds",
            "0",
            "--merge",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("readiness json");
    assert_eq!(json["result"]["final_status"], "blocked");
    assert_eq!(json["result"]["actions"][0]["status"], "skipped");
    let blockers = json["result"]["attempts"][0]["blockers"]
        .as_array()
        .expect("blockers");
    assert!(blockers.iter().any(|blocker| {
        blocker
            .as_str()
            .is_some_and(|blocker| blocker.contains("head SHA"))
    }));
    let capsule_path = std::path::Path::new(&capsule);
    assert!(capsule_path.join("pr-readiness.json").is_file());
    assert!(capsule_path.join("pr-readiness.md").is_file());
}

#[test]
fn pr_readiness_blocks_missing_merge_state_status() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    write_pr_agent_source_fixtures(&source_dir, 49);
    std::fs::write(
        source_dir.join("gh-pr-view.json"),
        r#"{
  "number": 49,
  "url": "https://github.com/BjornMelin/dev-skills/pull/49",
  "state": "OPEN",
  "isDraft": false,
  "mergeable": "MERGEABLE",
  "reviewDecision": "APPROVED",
  "headRefOid": "abc123",
  "headRefName": "feature",
  "baseRefName": "main",
  "baseRefOid": "base123",
  "statusCheckRollup": [],
  "labels": [{"name": "ready"}]
}"#,
    )
    .expect("write pr view without merge state");
    let capsule = init_capsule_fixture(
        &root,
        "pr-readiness-missing-merge-state",
        "PR readiness missing merge state",
    );

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--poll-interval-seconds",
            "0",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("readiness json");
    assert_eq!(json["result"]["final_status"], "blocked");
    assert!(
        json["result"]["attempts"][0]["blockers"]
            .as_array()
            .expect("blockers")
            .iter()
            .any(|blocker| blocker
                .as_str()
                .is_some_and(|blocker| blocker.contains("merge state was not captured")))
    );
}

#[test]
fn pr_readiness_distinguishes_stale_review_decision_from_clean_threads() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    write_pr_agent_source_fixtures(&source_dir, 49);
    write_pr_view_fixture(
        &source_dir,
        49,
        false,
        "MERGEABLE",
        "CLEAN",
        "CHANGES_REQUESTED",
    );
    write_review_threads_fixture(&source_dir, false);
    let capsule = init_capsule_fixture(&root, "pr-readiness-stale", "PR readiness stale");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--poll-interval-seconds",
            "0",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("readiness json");
    assert_eq!(json["result"]["final_status"], "ready");
    let warnings = json["result"]["attempts"][0]["warnings"]
        .as_array()
        .expect("warnings");
    assert!(warnings.iter().any(|warning| {
        warning
            .as_str()
            .is_some_and(|warning| warning.contains("reviewDecision is changes_requested"))
    }));
}

#[test]
fn pr_readiness_reports_outdated_review_comments_as_warning() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    write_pr_agent_source_fixtures(&source_dir, 49);
    std::fs::write(
        source_dir.join("gh-review-comments.json"),
        r#"[
  {"id": 7, "body": "stale feedback", "outdated": true}
]"#,
    )
    .expect("write outdated comments");
    let capsule = init_capsule_fixture(
        &root,
        "pr-readiness-outdated-comments",
        "PR readiness outdated comments",
    );

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--poll-interval-seconds",
            "0",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("readiness json");
    assert_eq!(json["result"]["final_status"], "ready");
    assert_eq!(json["result"]["attempts"][0]["outdated_review_comments"], 1);
    assert!(
        json["result"]["attempts"][0]["warnings"]
            .as_array()
            .expect("warnings")
            .iter()
            .any(|warning| warning
                .as_str()
                .is_some_and(|warning| warning.contains("outdated review comment")))
    );
}

#[test]
fn pr_readiness_fails_closed_for_missing_or_unknown_check_evidence() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    write_pr_agent_source_fixtures(&source_dir, 49);
    std::fs::write(source_dir.join("gh-pr-checks.json"), "[]").expect("write empty checks");
    let empty_checks_capsule = init_capsule_fixture(
        &root,
        "pr-readiness-empty-checks",
        "PR readiness empty checks",
    );

    let empty_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &empty_checks_capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--poll-interval-seconds",
            "0",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let empty_json: Value = serde_json::from_slice(&empty_output).expect("empty checks json");
    assert_eq!(empty_json["result"]["final_status"], "blocked");
    assert!(
        empty_json["result"]["attempts"][0]["blockers"]
            .as_array()
            .expect("blockers")
            .iter()
            .any(|blocker| blocker
                .as_str()
                .is_some_and(|blocker| blocker.contains("cannot prove CI passed")))
    );

    std::fs::write(
        source_dir.join("gh-pr-checks.json"),
        r#"[
  {"name": "ci", "status": "COMPLETED", "conclusion": "STALE", "link": "https://github.com/BjornMelin/dev-skills/actions/runs/778"}
]"#,
    )
    .expect("write stale check");
    let unknown_checks_capsule = init_capsule_fixture(
        &root,
        "pr-readiness-unknown-check",
        "PR readiness unknown check",
    );
    let unknown_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &unknown_checks_capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--poll-interval-seconds",
            "0",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let unknown_json: Value = serde_json::from_slice(&unknown_output).expect("unknown check json");
    assert_eq!(unknown_json["result"]["final_status"], "blocked");
    assert_eq!(
        unknown_json["result"]["attempts"][0]["failing_checks"][0]["run_id"],
        778
    );
}

#[test]
fn pr_readiness_blocks_unresolved_review_threads() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    write_pr_agent_source_fixtures(&source_dir, 49);
    write_review_threads_fixture(&source_dir, true);
    let capsule = init_capsule_fixture(&root, "pr-readiness-unresolved", "PR readiness unresolved");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--poll-interval-seconds",
            "0",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("readiness json");
    assert_eq!(json["result"]["final_status"], "blocked");
    assert_eq!(
        json["result"]["attempts"][0]["pr"]["review_threads"]["unresolved"],
        1
    );
}

#[test]
fn pr_readiness_reports_failing_pending_and_draft_states() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    write_pr_agent_source_fixtures(&source_dir, 49);
    write_pr_checks_fixture(
        &source_dir,
        "fail",
        "FAILURE",
        "https://github.com/BjornMelin/dev-skills/actions/runs/777/jobs/888",
    );
    write_pr_view_fixture(&source_dir, 49, false, "MERGEABLE", "UNSTABLE", "APPROVED");
    let failing_capsule =
        init_capsule_fixture(&root, "pr-readiness-failing", "PR readiness failing");

    let failing_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &failing_capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--poll-interval-seconds",
            "0",
            "--rerun-failed",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let failing_json: Value = serde_json::from_slice(&failing_output).expect("failing json");
    assert_eq!(failing_json["result"]["final_status"], "blocked");
    assert_eq!(
        failing_json["result"]["attempts"][0]["failing_checks"][0]["run_id"],
        777
    );
    assert_eq!(failing_json["result"]["actions"][0]["status"], "planned");

    write_pr_checks_fixture(
        &source_dir,
        "pending",
        "PENDING",
        "https://example.test/pending",
    );
    write_pr_view_fixture(&source_dir, 49, false, "MERGEABLE", "UNKNOWN", "APPROVED");
    let pending_capsule =
        init_capsule_fixture(&root, "pr-readiness-pending", "PR readiness pending");
    let pending_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &pending_capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--poll-interval-seconds",
            "0",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let pending_json: Value = serde_json::from_slice(&pending_output).expect("pending json");
    assert_eq!(pending_json["result"]["final_status"], "waiting");
    assert_eq!(
        pending_json["result"]["attempts"][0]["pending_checks"][0]["diagnostic_command"],
        "https://example.test/pending"
    );

    write_pr_checks_fixture(&source_dir, "pass", "SUCCESS", "https://example.test/ci");
    write_pr_view_fixture(&source_dir, 49, true, "MERGEABLE", "DRAFT", "APPROVED");
    let draft_capsule = init_capsule_fixture(&root, "pr-readiness-draft", "PR readiness draft");
    let draft_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &draft_capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--poll-interval-seconds",
            "0",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let draft_json: Value = serde_json::from_slice(&draft_output).expect("draft json");
    assert_eq!(draft_json["result"]["final_status"], "blocked");
    assert!(
        draft_json["result"]["attempts"][0]["blockers"]
            .as_array()
            .expect("blockers")
            .iter()
            .any(|blocker| blocker
                .as_str()
                .is_some_and(|blocker| blocker.contains("draft")))
    );
}

#[cfg(unix)]
#[test]
fn pr_readiness_apply_merge_requires_clean_latest_state() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let fixtures = temp.path().join("fixtures");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    write_pr_agent_source_fixtures(&fixtures, 49);
    write_fake_gh(&bin);
    let log = temp.path().join("gh.log");
    let capsule = init_capsule_fixture(&root, "pr-readiness-merge", "PR readiness merge");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let test_path = format!("{}:{old_path}", bin.display());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", test_path)
        .env("GH_FIXTURES", &fixtures)
        .env("GH_LOG", &log)
        .env("GH_TOKEN", "test-token")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--poll-interval-seconds",
            "0",
            "--merge",
            "--apply",
            "--delete-branch",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("merge json");
    assert_eq!(json["result"]["final_status"], "merged");
    assert_eq!(json["result"]["actions"][0]["status"], "applied");
    let gh_log = std::fs::read_to_string(log).expect("gh log");
    assert!(
        gh_log
            .matches("pr view 49 --repo BjornMelin/dev-skills")
            .count()
            >= 2
    );
    assert!(
        gh_log.contains("pr merge 49 --repo BjornMelin/dev-skills --squash --match-head-commit abc123 --delete-branch")
    );
}

#[cfg(unix)]
#[test]
fn pr_readiness_apply_merge_rechecks_live_state_before_merge() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let fixtures = temp.path().join("fixtures");
    let refresh_fixtures = temp.path().join("refresh-fixtures");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    write_pr_agent_source_fixtures(&fixtures, 49);
    write_pr_agent_source_fixtures(&refresh_fixtures, 49);
    write_pr_view_fixture(
        &refresh_fixtures,
        49,
        true,
        "MERGEABLE",
        "DRAFT",
        "APPROVED",
    );
    write_fake_gh(&bin);
    let log = temp.path().join("gh.log");
    let capsule = init_capsule_fixture(
        &root,
        "pr-readiness-merge-refresh",
        "PR readiness merge refresh",
    );
    let old_path = std::env::var("PATH").unwrap_or_default();
    let test_path = format!("{}:{old_path}", bin.display());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", test_path)
        .env("GH_FIXTURES", &fixtures)
        .env("GH_REFRESH_FIXTURES", &refresh_fixtures)
        .env("GH_LOG", &log)
        .env("GH_TOKEN", "test-token")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--poll-interval-seconds",
            "0",
            "--merge",
            "--apply",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("merge refresh json");
    assert_eq!(json["result"]["final_status"], "blocked");
    assert_eq!(json["result"]["actions"][0]["status"], "failed");
    assert!(
        json["result"]["actions"][0]["reason"]
            .as_str()
            .expect("reason")
            .contains("pre-merge readiness refresh")
    );
    let gh_log = std::fs::read_to_string(log).expect("gh log");
    assert!(!gh_log.contains("pr merge 49"));
}

#[cfg(unix)]
#[test]
fn pr_readiness_apply_rejects_replay_source() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let source_dir = temp.path().join("sources");
    write_pr_agent_source_fixtures(&source_dir, 49);
    let capsule = init_capsule_fixture(
        &root,
        "pr-readiness-apply-replay",
        "PR readiness apply replay",
    );

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--source-dir",
            source_dir.to_str().expect("utf8 source dir"),
            "--apply",
            "--merge",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("apply replay json");
    assert_eq!(json["ok"], false);
    assert!(
        json["result"]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("--source-dir")
    );
}

#[cfg(unix)]
#[test]
fn pr_readiness_apply_rerun_delegates_to_hosted_action_preflight() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let fixtures = temp.path().join("fixtures");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    write_pr_agent_source_fixtures(&fixtures, 49);
    write_pr_checks_fixture(
        &fixtures,
        "fail",
        "FAILURE",
        "https://github.com/BjornMelin/dev-skills/actions/runs/777/jobs/888",
    );
    write_pr_view_fixture(&fixtures, 49, false, "MERGEABLE", "UNSTABLE", "APPROVED");
    write_fake_gh(&bin);
    let log = temp.path().join("gh.log");
    let capsule = init_capsule_fixture(&root, "pr-readiness-rerun", "PR readiness rerun");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let test_path = format!("{}:{old_path}", bin.display());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", test_path)
        .env("GH_FIXTURES", &fixtures)
        .env("GH_LOG", &log)
        .env("GH_TOKEN", "test-token")
        .env("RUN_CONCLUSION", "failure")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--poll-interval-seconds",
            "0",
            "--rerun-failed",
            "--apply",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("rerun json");
    assert_eq!(json["result"]["actions"][0]["kind"], "rerun_failed_jobs");
    assert_eq!(json["result"]["actions"][0]["status"], "applied");
    let gh_log = std::fs::read_to_string(log).expect("gh log");
    assert!(gh_log.contains("api repos/BjornMelin/dev-skills/actions/runs/777"));
    assert!(gh_log.contains(
        "api --method POST repos/BjornMelin/dev-skills/actions/runs/777/rerun-failed-jobs"
    ));
}

#[cfg(unix)]
#[test]
fn pr_readiness_apply_rerun_rejects_mismatched_workflow_run_identity() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let fixtures = temp.path().join("fixtures");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    write_pr_agent_source_fixtures(&fixtures, 49);
    write_pr_checks_fixture(
        &fixtures,
        "fail",
        "FAILURE",
        "https://github.com/BjornMelin/dev-skills/actions/runs/779/jobs/889",
    );
    write_pr_view_fixture(&fixtures, 49, false, "MERGEABLE", "UNSTABLE", "APPROVED");
    write_fake_gh(&bin);
    let log = temp.path().join("gh.log");
    let capsule = init_capsule_fixture(
        &root,
        "pr-readiness-rerun-mismatch",
        "PR readiness rerun mismatch",
    );
    let old_path = std::env::var("PATH").unwrap_or_default();
    let test_path = format!("{}:{old_path}", bin.display());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", test_path)
        .env("GH_FIXTURES", &fixtures)
        .env("GH_LOG", &log)
        .env("GH_TOKEN", "test-token")
        .env("RUN_CONCLUSION", "failure")
        .env("RUN_REPOSITORY", "Other/repo")
        .args([
            "--json",
            "pr",
            "readiness",
            "--capsule",
            &capsule,
            "--repo",
            "BjornMelin/dev-skills",
            "--number",
            "49",
            "--poll-interval-seconds",
            "0",
            "--rerun-failed",
            "--apply",
            "--checked-at",
            "2026-05-09T05:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("rerun mismatch json");
    assert_eq!(json["result"]["final_status"], "blocked");
    assert_eq!(json["result"]["actions"][0]["status"], "failed");
    assert!(
        json["result"]["actions"][0]["stderr"]
            .as_str()
            .expect("stderr")
            .contains("workflow run repository")
    );
    let gh_log = std::fs::read_to_string(log).expect("gh log");
    assert!(gh_log.contains("api repos/BjornMelin/dev-skills/actions/runs/779"));
    assert!(!gh_log.contains("rerun-failed-jobs"));
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
    let checks = record_json["result"]["pr"]["checks"]
        .as_array()
        .expect("checks array");
    let lint = checks
        .iter()
        .find(|check| check["name"] == "lint")
        .expect("lint check");
    assert_eq!(lint["status"], "completed");
    assert_eq!(lint["conclusion"], "failure");
    assert_eq!(
        record_json["result"]["pr"]["sources"][0]["kind"],
        "gh-pr-checks"
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
