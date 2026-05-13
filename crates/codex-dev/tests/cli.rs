use assert_cmd::Command;
use serde_json::{Value, json};
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

fn write_research_bundle_fixture(root: &std::path::Path, schema: &str) -> std::path::PathBuf {
    let path = root.join("evidence-bundle.json");
    std::fs::write(
        &path,
        format!(
            r#"{{
  "schema": "{schema}",
  "generated_at": "2026-05-11T12:00:00Z",
  "status": "passed",
  "strict": true,
  "run": {{
    "path": ".codex/research/run.json",
    "query": "verify package upgrade behavior",
    "profile": "deep",
    "topic": "dependency",
    "status": "closed",
    "cache_source_ids": ["src-official-docs"]
  }},
  "budget": {{
    "by_provider": [
      {{"provider": "github", "budget": 8, "spent": 2, "remaining": 6}},
      {{"provider": "context7", "budget": 4, "spent": 1, "remaining": 3}}
    ]
  }},
  "provider_errors": [],
  "ledger": {{
    "path": ".codex/research/ledger.jsonl",
    "source_count": 2,
    "claim_count": 2,
    "source_ids": ["src-official-docs", "src-github-source"],
    "claim_ids": ["claim-official-docs-first", "claim-source-hydrated"]
  }},
  "citation_coverage": {{
    "cited_claims": 2,
    "uncited_claims": 0,
    "uncited_claim_ids": [],
    "missing_source_refs": [],
    "coverage": 1.0
  }},
  "source_freshness": {{
    "by_status": {{"current": 2}},
    "unknown_source_ids": []
  }},
  "report": {{
    "path": ".codex/research/report.md",
    "exists": true
  }},
  "artifacts": [".codex/research/evidence-bundle.json"],
  "warnings": [],
  "failures": []
}}"#
        ),
    )
    .expect("write research bundle fixture");
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

#[test]
fn task_commands_emit_stable_json_reports() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let alpha = init_capsule_fixture(&root, "alpha-task", "Alpha task");
    init_capsule_fixture(&root, "beta-task", "Beta task");

    let list_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "task",
            "list",
            "--root",
            root.to_str().expect("task root"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list_json: Value = serde_json::from_slice(&list_output).expect("task list json");
    assert_eq!(list_json["ok"], true);
    assert_eq!(list_json["command"], "task list");
    assert_eq!(list_json["result"]["schema"], "task_index.v1");
    assert_eq!(list_json["result"]["root_status"], "ready");
    assert_eq!(list_json["result"]["total"], 2);
    assert_eq!(list_json["result"]["valid"], 2);
    assert_eq!(
        list_json["result"]["tasks"][0]["capsule"]["id"],
        "alpha-task"
    );

    let show_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "task",
            "show",
            "--root",
            root.to_str().expect("task root"),
            "alpha-task",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let show_json: Value = serde_json::from_slice(&show_output).expect("task show json");
    assert_eq!(show_json["ok"], true);
    assert_eq!(show_json["command"], "task show");
    assert_eq!(
        show_json["result"]["task"]["capsule"]["title"],
        "Alpha task"
    );

    let export_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args(["--json", "task", "export", &alpha])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let export_json: Value = serde_json::from_slice(&export_output).expect("task export json");
    assert_eq!(export_json["ok"], true);
    assert_eq!(export_json["command"], "task export");
    assert_eq!(export_json["result"]["schema"], "task_index.v1");
    assert_eq!(export_json["result"]["capsule"]["id"], "alpha-task");
    assert_eq!(
        export_json["result"]["evidence"].as_array().unwrap().len(),
        1
    );
    assert!(
        export_json["result"]["markdown"]["plan.md"]
            .as_str()
            .expect("plan markdown")
            .contains("# Plan")
    );

    let file_root = temp.path().join("task-root-file");
    std::fs::write(&file_root, "not a directory\n").expect("file root");
    let file_root_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "task",
            "list",
            "--root",
            file_root.to_str().expect("file root"),
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let file_root_json: Value =
        serde_json::from_slice(&file_root_output).expect("task list file-root json");
    assert_eq!(file_root_json["ok"], false);
    assert_eq!(file_root_json["result"]["root_status"], "unusable");
    assert!(
        file_root_json["result"]["diagnostics"][0]
            .as_str()
            .expect("diagnostic")
            .contains("task root is not a directory")
    );
}

#[cfg(unix)]
fn write_fake_local_tool(bin_dir: &std::path::Path, name: &str) {
    use std::os::unix::fs::PermissionsExt;

    let script = bin_dir.join(name);
    let body = if name == "git" {
        r#"#!/bin/sh
if [ -n "$GH_TOKEN" ] || [ -n "$GITHUB_TOKEN" ] || [ -n "$GH_ENTERPRISE_TOKEN" ] || [ -n "$GITHUB_ENTERPRISE_TOKEN" ]; then
  echo "token env leaked into git probe" >&2
  exit 7
fi
case "$*" in
  *"check-ignore"*) exit 0 ;;
  *"--version"*) printf 'git version fixture\n' ;;
  *) exit 0 ;;
esac
"#
    } else {
        r#"#!/bin/sh
if [ -n "$GH_TOKEN" ] || [ -n "$GITHUB_TOKEN" ] || [ -n "$GH_ENTERPRISE_TOKEN" ] || [ -n "$GITHUB_ENTERPRISE_TOKEN" ]; then
  echo "token env leaked into probe" >&2
  exit 7
fi
if [ "$1" = "--version" ]; then
  name="${0##*/}"
  printf '%s fixture\n' "$name"
fi
"#
    };
    std::fs::write(&script, body).expect("write fake local tool");
    let mut perms = std::fs::metadata(&script)
        .expect("fake local tool metadata")
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms).expect("fake local tool executable");
}

#[cfg(unix)]
fn write_local_doctor_fixture(root: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let repo = root.join("repo");
    std::fs::create_dir_all(repo.join("docs/runbooks")).expect("repo docs");
    std::fs::write(repo.join("Cargo.toml"), "[workspace]\n").expect("repo Cargo.toml");
    std::fs::write(repo.join("docs/runbooks/validation.md"), "# Validation\n")
        .expect("repo validation");
    std::fs::write(
        repo.join(".gitignore"),
        ".codex/tasks/\n.codex/research/\n/target/\n",
    )
    .expect("repo gitignore");

    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).expect("fake bin");
    for name in [
        "codex-dev",
        "codex-dev-tui",
        "codex-research",
        "cargo",
        "rustc",
        "git",
        "gh",
        "python3",
        "cargo-deny",
        "cargo-audit",
    ] {
        write_fake_local_tool(&bin, name);
    }
    (repo, bin)
}

fn write_skill_inventory_repo(root: &std::path::Path) -> std::path::PathBuf {
    let repo = root.join("repo");
    std::fs::create_dir_all(repo.join("docs/runbooks")).expect("repo docs");
    std::fs::create_dir_all(repo.join("docs")).expect("docs");
    std::fs::create_dir_all(repo.join("skills/dist")).expect("dist");
    std::fs::write(repo.join("Cargo.toml"), "[workspace]\n").expect("repo Cargo.toml");
    std::fs::write(repo.join("docs/runbooks/validation.md"), "# Validation\n")
        .expect("repo validation");
    std::fs::write(
        repo.join("README.md"),
        "| Skill | Description | Source |\n| --- | --- | --- |\n| `alpha-skill` | Alpha. | [skills/alpha-skill/SKILL.md](skills/alpha-skill/SKILL.md) |\n| `beta-skill` | Beta. | [skills/beta-skill/SKILL.md](skills/beta-skill/SKILL.md) |\n",
    )
    .expect("readme");
    std::fs::write(
        repo.join("docs/index.md"),
        "- [Alpha](../skills/alpha-skill/SKILL.md)\n",
    )
    .expect("docs index");
    std::fs::write(repo.join("skills/dist/alpha-skill.skill"), "zip fixture\n")
        .expect("dist bundle");

    let alpha = repo.join("skills/alpha-skill");
    std::fs::create_dir_all(alpha.join("references/deep")).expect("alpha references");
    std::fs::create_dir_all(alpha.join("scripts")).expect("alpha scripts");
    std::fs::create_dir_all(alpha.join("templates")).expect("alpha templates");
    std::fs::write(
        alpha.join("SKILL.md"),
        r#"---
name: alpha-skill
description: |
  Alpha skill description.
license: MIT
allowed-tools:
  - web.run
  - mcp.github
metadata:
  category: test
---

# Alpha
"#,
    )
    .expect("alpha skill");
    std::fs::write(alpha.join("references/deep/guide.md"), "# Guide\n").expect("alpha ref");
    std::fs::write(alpha.join("scripts/check.sh"), "#!/bin/sh\n").expect("alpha script");
    std::fs::write(alpha.join("templates/prompt.md"), "Prompt\n").expect("alpha template");

    let beta = repo.join("skills/beta-skill");
    std::fs::create_dir_all(&beta).expect("beta dir");
    std::fs::write(
        beta.join("SKILL.md"),
        r#"---
name: beta-skill
description: Beta skill description.
---

# Beta
"#,
    )
    .expect("beta skill");
    repo
}

fn write_bootstrap_repo(root: &std::path::Path) -> std::path::PathBuf {
    let repo = root.join("bootstrap-repo");
    std::fs::create_dir_all(repo.join("docs/runbooks")).expect("repo docs");
    std::fs::create_dir_all(repo.join("bootstrap/packs")).expect("pack root");
    std::fs::create_dir_all(repo.join("bootstrap/templates/generic")).expect("generic templates");
    std::fs::create_dir_all(repo.join("bootstrap/templates/rust-cli")).expect("rust templates");
    std::fs::write(repo.join("Cargo.toml"), "[workspace]\n").expect("repo Cargo.toml");
    std::fs::write(repo.join("docs/runbooks/validation.md"), "# Validation\n")
        .expect("repo validation");
    for template in [
        "generic/AGENTS.md.tmpl",
        "generic/agent-bootstrap.md.tmpl",
        "generic/codex-agents-readme.md.tmpl",
        "generic/gitignore.tmpl",
        "rust-cli/rust-agent-workflow.md.tmpl",
    ] {
        std::fs::write(
            repo.join("bootstrap/templates").join(template),
            "template\n",
        )
        .expect("template fixture");
    }
    std::fs::write(
        repo.join("bootstrap/packs/codex-agent-repo.json"),
        r#"{
  "schema": "dev-skills.bootstrap-pack.v1",
  "name": "codex-agent-repo",
  "description": "Generic Codex-ready repository bootstrap.",
  "composes": {
    "skills": ["deep-researcher", "subspawn"],
    "subagent_sources": ["subagents/codex/agents/global"]
  },
  "files": [
    {"target": "AGENTS.md", "template": "generic/AGENTS.md.tmpl"},
    {"target": "docs/agent-bootstrap.md", "template": "generic/agent-bootstrap.md.tmpl"}
  ],
  "advisory_host_checks": ["git diff --check"]
}"#,
    )
    .expect("codex pack");
    std::fs::write(
        repo.join("bootstrap/packs/rust-cli-agent-repo.json"),
        r#"{
  "schema": "dev-skills.bootstrap-pack.v1",
  "name": "rust-cli-agent-repo",
  "description": "Rust CLI repository bootstrap.",
  "composes": {
    "skills": ["rust-expert", "rust-cli-clap"],
    "subagent_sources": ["subagents/codex/agents/global"]
  },
  "files": [
    {"target": "docs/rust-agent-workflow.md", "template": "rust-cli/rust-agent-workflow.md.tmpl"}
  ],
  "advisory_host_checks": ["cargo test", "git diff --check"]
}"#,
    )
    .expect("rust pack");
    repo
}

#[test]
fn skills_inventory_emits_stable_json_report() {
    let temp = tempdir().expect("tempdir");
    let repo = write_skill_inventory_repo(temp.path());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    assert_eq!(json["command"], "skills inventory");
    assert_eq!(json["result"]["schema"], "skill_inventory.v1");
    assert_eq!(json["result"]["checked_at"], "2026-05-12T08:00:00Z");
    assert_eq!(json["result"]["total"], 2);
    assert_eq!(json["result"]["valid"], 2);
    assert_eq!(json["result"]["invalid"], 0);
    let skills = json["result"]["skills"].as_array().expect("skills array");
    assert_eq!(skills[0]["name"], "alpha-skill");
    assert_eq!(skills[0]["path"], "skills/alpha-skill");
    assert_eq!(skills[0]["resources"]["references"]["files"], 1);
    assert_eq!(skills[0]["resources"]["references"]["capped"], false);
    assert_eq!(skills[0]["resources"]["scripts"]["files"], 1);
    assert_eq!(skills[0]["resources"]["templates"]["files"], 1);
    assert_eq!(skills[0]["exposure"]["readme_catalog"], true);
    assert_eq!(skills[0]["exposure"]["docs_index"], true);
    assert_eq!(skills[0]["package"]["present"], true);
    assert_eq!(skills[0]["package"]["rejected"], false);
    assert_eq!(skills[0]["metadata_present"], true);
    assert!(
        skills[0]["allowed_tools"]
            .as_array()
            .expect("allowed tools")
            .iter()
            .any(|tool| tool.as_str() == Some("web.run"))
    );
    assert_eq!(skills[1]["name"], "beta-skill");
    assert_eq!(skills[1]["package"]["present"], false);
    assert_eq!(skills[1]["package"]["rejected"], false);
    assert!(
        skills[1]["underbuilt_signals"]
            .as_array()
            .expect("signals")
            .iter()
            .any(|signal| signal.as_str() == Some("missing_docs_index_exposure"))
    );
}

#[test]
fn bootstrap_status_emits_stable_json_report() {
    let temp = tempdir().expect("tempdir");
    let repo = write_bootstrap_repo(temp.path());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "bootstrap",
            "status",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T09:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).expect("utf8 output");
    assert!(
        !output_text.contains(repo.to_str().expect("repo path")),
        "bootstrap status must redact local repo paths by default"
    );
    let json: Value = serde_json::from_slice(&output).expect("bootstrap status json");
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "bootstrap status");
    assert_eq!(json["result"]["schema"], "bootstrap_status.v1");
    assert_eq!(json["result"]["checked_at"], "2026-05-12T09:00:00Z");
    assert_eq!(json["result"]["pack_root"], "bootstrap/packs");
    assert_eq!(json["result"]["template_root"], "bootstrap/templates");
    assert_eq!(json["result"]["total"], 2);
    assert_eq!(json["result"]["valid"], 2);
    assert_eq!(json["result"]["invalid"], 0);
    assert_eq!(json["result"]["diagnostics"], json!([]));
    assert_eq!(
        json["result"]["policy_gates"]["profile"],
        "bootstrap_install"
    );
    assert!(
        json["result"]["policy_gates"]["gate_ids"]
            .as_array()
            .expect("gate ids")
            .iter()
            .any(|gate| gate.as_str() == Some("codex-dev-bootstrap-status"))
    );
    let packs = json["result"]["packs"].as_array().expect("packs");
    assert_eq!(packs[0]["name"], "codex-agent-repo");
    assert_eq!(packs[0]["path"], "bootstrap/packs/codex-agent-repo.json");
    assert_eq!(packs[0]["schema"], "dev-skills.bootstrap-pack.v1");
    assert_eq!(packs[0]["valid"], true);
    assert_eq!(packs[0]["file_count"], 2);
    assert_eq!(
        packs[0]["files"][0]["template"],
        "bootstrap/templates/generic/AGENTS.md.tmpl"
    );
    assert_eq!(packs[0]["composes"]["skills"][0], "deep-researcher");
    assert_eq!(packs[0]["advisory_host_checks"][0], "git diff --check");
}

#[test]
fn bootstrap_plan_emits_redacted_dry_run_actions() {
    let temp = tempdir().expect("tempdir");
    let repo = write_bootstrap_repo(temp.path());
    let out = temp.path().join("preview");
    std::fs::create_dir_all(&out).expect("preview root");
    std::fs::write(out.join("AGENTS.md"), "existing\n").expect("existing target");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "bootstrap",
            "plan",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--pack",
            "codex-agent-repo",
            "--out",
            out.to_str().expect("out path"),
            "--repo-name",
            "codex-smoke",
            "--primary-language",
            "rust",
            "--checked-at",
            "2026-05-12T09:30:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).expect("utf8 output");
    assert!(
        !output_text.contains(out.to_str().expect("out path")),
        "bootstrap plan must redact local output paths by default"
    );
    assert!(
        !output_text.contains(repo.to_str().expect("repo path")),
        "bootstrap plan must redact local repo paths by default"
    );
    let json: Value = serde_json::from_slice(&output).expect("bootstrap plan json");
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "bootstrap plan");
    assert_eq!(json["result"]["schema"], "bootstrap_plan.v1");
    assert_eq!(json["result"]["checked_at"], "2026-05-12T09:30:00Z");
    assert_eq!(json["result"]["pack"]["name"], "codex-agent-repo");
    assert_eq!(json["result"]["dry_run"], true);
    assert_eq!(json["result"]["output_root"], "<bootstrap-out>");
    assert_eq!(json["result"]["repo_name"], "codex-smoke");
    assert_eq!(json["result"]["primary_language"], "rust");
    assert_eq!(json["result"]["target_count"], 2);
    assert_eq!(json["result"]["action_counts"]["would_overwrite"], 1);
    assert_eq!(json["result"]["action_counts"]["would_write"], 1);
    assert_eq!(
        json["result"]["advisory_host_checks"][0],
        "git diff --check"
    );
    let files = json["result"]["files"].as_array().expect("planned files");
    assert_eq!(files[0]["target"], "AGENTS.md");
    assert_eq!(files[0]["action"], "would_overwrite");
    assert!(files[0].get("target_path").is_none());
    assert_eq!(files[1]["target"], "docs/agent-bootstrap.md");
    assert_eq!(files[1]["action"], "would_write");
}

#[test]
fn bootstrap_plan_expands_home_paths_like_renderer() {
    let temp = tempdir().expect("tempdir");
    let repo = write_bootstrap_repo(temp.path());
    let home = temp.path().join("home");
    let preview = home.join("preview");
    std::fs::create_dir_all(&preview).expect("preview root");
    std::fs::write(preview.join("AGENTS.md"), "existing\n").expect("existing target");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("HOME", &home)
        .args([
            "--json",
            "bootstrap",
            "plan",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--pack",
            "codex-agent-repo",
            "--out",
            "~/preview",
            "--checked-at",
            "2026-05-12T09:45:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).expect("utf8 output");
    assert!(
        !output_text.contains(home.to_str().expect("home path")),
        "bootstrap plan must redact expanded home paths by default"
    );
    let json: Value = serde_json::from_slice(&output).expect("bootstrap plan json");
    assert_eq!(json["ok"], true);
    assert_eq!(json["result"]["action_counts"]["would_overwrite"], 1);
    assert_eq!(json["result"]["files"][0]["action"], "would_overwrite");
}

#[cfg(unix)]
#[test]
fn bootstrap_status_rejects_symlinked_pack_root() {
    let temp = tempdir().expect("tempdir");
    let repo = write_bootstrap_repo(temp.path());
    let outside = temp.path().join("outside-packs");
    std::fs::create_dir_all(&outside).expect("outside packs");
    std::fs::remove_dir_all(repo.join("bootstrap/packs")).expect("remove pack root");
    std::os::unix::fs::symlink(&outside, repo.join("bootstrap/packs")).expect("pack symlink");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "bootstrap",
            "status",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T10:00:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("bootstrap status json");
    assert_eq!(json["ok"], false);
    assert_eq!(
        json["result"]["diagnostics"][0]["code"],
        "invalid_bootstrap_pack_root"
    );
    assert!(
        json["result"]["diagnostics"][0]["message"]
            .as_str()
            .expect("diagnostic")
            .contains("symlink")
    );
}

#[test]
fn bootstrap_status_redacts_malformed_pack_root_diagnostics() {
    let temp = tempdir().expect("tempdir");
    let repo = write_bootstrap_repo(temp.path());
    std::fs::remove_dir_all(repo.join("bootstrap/packs")).expect("remove pack root");
    std::fs::write(repo.join("bootstrap/packs"), "not a directory\n").expect("pack root file");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "bootstrap",
            "status",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T10:05:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).expect("utf8 output");
    assert!(
        !output_text.contains(repo.to_str().expect("repo path")),
        "malformed pack-root diagnostics must redact local repo paths by default"
    );
    let json: Value = serde_json::from_slice(&output).expect("bootstrap status json");
    assert_eq!(json["ok"], false);
    assert_eq!(
        json["result"]["diagnostics"][0]["message"],
        "bootstrap pack root is not a directory: bootstrap/packs"
    );
}

#[test]
fn bootstrap_plan_reports_output_root_errors_inside_contract() {
    let temp = tempdir().expect("tempdir");
    let repo = write_bootstrap_repo(temp.path());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env_remove("HOME")
        .args([
            "--json",
            "bootstrap",
            "plan",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--pack",
            "codex-agent-repo",
            "--out",
            "~/preview",
            "--checked-at",
            "2026-05-12T10:10:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).expect("utf8 output");
    assert!(
        !output_text.contains(repo.to_str().expect("repo path")),
        "output-root diagnostics must redact local repo paths by default"
    );
    let json: Value = serde_json::from_slice(&output).expect("bootstrap plan json");
    assert_eq!(json["ok"], false);
    assert_eq!(json["result"]["schema"], "bootstrap_plan.v1");
    assert_eq!(json["result"]["output_root"], "<bootstrap-out>");
    assert!(
        json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "invalid_bootstrap_output_root")
    );
}

#[cfg(unix)]
#[test]
fn bootstrap_plan_rejects_output_target_symlink_escape() {
    let temp = tempdir().expect("tempdir");
    let repo = write_bootstrap_repo(temp.path());
    let out = temp.path().join("preview");
    let outside = temp.path().join("outside");
    std::fs::create_dir_all(&out).expect("preview root");
    std::fs::create_dir_all(&outside).expect("outside root");
    std::os::unix::fs::symlink(&outside, out.join("docs")).expect("docs symlink");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "bootstrap",
            "plan",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--pack",
            "codex-agent-repo",
            "--out",
            out.to_str().expect("out path"),
            "--checked-at",
            "2026-05-12T10:15:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("bootstrap plan json");
    assert_eq!(json["ok"], false);
    assert!(
        json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "output_target_escape")
    );
}

#[test]
fn skills_inventory_reports_invalid_frontmatter() {
    let temp = tempdir().expect("tempdir");
    let repo = write_skill_inventory_repo(temp.path());
    let invalid = repo.join("skills/invalid-skill");
    std::fs::create_dir_all(&invalid).expect("invalid skill dir");
    std::fs::write(
        invalid.join("SKILL.md"),
        r#"---
name: different-name
description: Invalid skill.
extra: nope
---

# Invalid
"#,
    )
    .expect("invalid skill");
    let numeric = repo.join("skills/numeric-skill");
    std::fs::create_dir_all(&numeric).expect("numeric skill dir");
    std::fs::write(
        numeric.join("SKILL.md"),
        r#"---
name: 123
description: 456
---

# Numeric
"#,
    )
    .expect("numeric skill");
    let malformed = repo.join("skills/malformed-skill");
    std::fs::create_dir_all(&malformed).expect("malformed skill dir");
    std::fs::write(
        malformed.join("SKILL.md"),
        r#"---
name: malformed-skill
description: "unterminated
---

# Malformed
"#,
    )
    .expect("malformed skill");
    let traversal = repo.join("skills/traversal-skill");
    std::fs::create_dir_all(&traversal).expect("traversal skill dir");
    std::fs::write(
        traversal.join("SKILL.md"),
        r#"---
name: ../outside
description: Traversal skill.
---

# Traversal
"#,
    )
    .expect("traversal skill");
    let boolean_alias = repo.join("skills/boolean-alias-skill");
    std::fs::create_dir_all(&boolean_alias).expect("boolean alias skill dir");
    std::fs::write(
        boolean_alias.join("SKILL.md"),
        r#"---
name: on
description: yes
---

# Boolean Alias
"#,
    )
    .expect("boolean alias skill");
    let timestamp = repo.join("skills/timestamp-skill");
    std::fs::create_dir_all(&timestamp).expect("timestamp skill dir");
    std::fs::write(
        timestamp.join("SKILL.md"),
        r#"---
name: timestamp-skill
description: 2026-05-12
---

# Timestamp
"#,
    )
    .expect("timestamp skill");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    assert_eq!(json["ok"], false);
    assert_eq!(json["result"]["invalid"], 6);
    let invalid_skill = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "invalid-skill")
        .expect("invalid skill entry");
    let errors = invalid_skill["validation"]["errors"]
        .as_array()
        .expect("validation errors");
    assert!(errors.iter().any(|error| {
        error
            .as_str()
            .expect("error")
            .contains("unexpected frontmatter key")
    }));
    assert!(errors.iter().any(|error| {
        error
            .as_str()
            .expect("error")
            .contains("must match frontmatter name")
    }));
    let numeric_skill = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "numeric-skill")
        .expect("numeric skill entry");
    assert!(
        numeric_skill["validation"]["errors"]
            .as_array()
            .expect("numeric errors")
            .iter()
            .any(|error| error.as_str().expect("error").contains("must be a string"))
    );
    let malformed_skill = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "malformed-skill")
        .expect("malformed skill entry");
    assert!(
        malformed_skill["validation"]["errors"]
            .as_array()
            .expect("malformed errors")
            .iter()
            .any(|error| error.as_str().expect("error").contains("unterminated"))
    );
    let traversal_skill = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "traversal-skill")
        .expect("traversal skill entry");
    assert_eq!(
        traversal_skill["package"]["path"],
        "skills/dist/traversal-skill.skill"
    );
    assert_eq!(traversal_skill["package"]["present"], false);
    assert_eq!(traversal_skill["package"]["rejected"], false);
    let boolean_alias_skill = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "boolean-alias-skill")
        .expect("boolean alias skill entry");
    assert!(
        boolean_alias_skill["validation"]["errors"]
            .as_array()
            .expect("boolean alias errors")
            .iter()
            .any(|error| error.as_str().expect("error").contains("must be a string"))
    );
    let timestamp_skill = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "timestamp-skill")
        .expect("timestamp skill entry");
    assert!(
        timestamp_skill["validation"]["errors"]
            .as_array()
            .expect("timestamp errors")
            .iter()
            .any(|error| error.as_str().expect("error").contains("must be a string"))
    );
}

#[test]
fn skills_inventory_accepts_yaml_inline_comments_in_scalars() {
    let temp = tempdir().expect("tempdir");
    let repo = write_skill_inventory_repo(temp.path());
    let commented = repo.join("skills/commented-skill");
    std::fs::create_dir_all(&commented).expect("commented skill dir");
    std::fs::write(
        commented.join("SKILL.md"),
        r#"---
name: commented-skill # catalog identity
description: Commented skill description. # human note
allowed-tools: [bash, python3] # common shells
---

# Commented
"#,
    )
    .expect("commented skill");
    let quoted = repo.join("skills/quoted-comment-skill");
    std::fs::create_dir_all(&quoted).expect("quoted commented skill dir");
    std::fs::write(
        quoted.join("SKILL.md"),
        r#"---
name: "quoted-comment-skill" # catalog identity
description: 'Quoted skill description.' # human note
---

# Quoted Commented
"#,
    )
    .expect("quoted commented skill");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    let commented_skill = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "commented-skill")
        .expect("commented skill entry");
    assert_eq!(commented_skill["name"], "commented-skill");
    assert_eq!(
        commented_skill["description"],
        "Commented skill description."
    );
    assert_eq!(
        commented_skill["allowed_tools"]
            .as_array()
            .expect("allowed tools"),
        &vec![
            Value::String("bash".into()),
            Value::String("python3".into())
        ]
    );
    assert_eq!(commented_skill["validation"]["valid"], true);
    let quoted_skill = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "quoted-comment-skill")
        .expect("quoted skill entry");
    assert_eq!(quoted_skill["name"], "quoted-comment-skill");
    assert_eq!(quoted_skill["description"], "Quoted skill description.");
    assert_eq!(quoted_skill["validation"]["valid"], true);
}

#[test]
fn skills_inventory_accepts_crlf_frontmatter_fences() {
    let temp = tempdir().expect("tempdir");
    let repo = write_skill_inventory_repo(temp.path());
    let crlf = repo.join("skills/crlf-skill");
    std::fs::create_dir_all(&crlf).expect("crlf skill dir");
    std::fs::write(
        crlf.join("SKILL.md"),
        "---\r\nname: crlf-skill\r\ndescription: CRLF skill description.\r\n---\r\n\r\n# CRLF\r\n",
    )
    .expect("crlf skill");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    let crlf_skill = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "crlf-skill")
        .expect("crlf skill entry");
    assert_eq!(crlf_skill["name"], "crlf-skill");
    assert_eq!(crlf_skill["validation"]["valid"], true);
}

#[test]
fn skills_inventory_accepts_bom_prefixed_frontmatter_fences() {
    let temp = tempdir().expect("tempdir");
    let repo = write_skill_inventory_repo(temp.path());
    let bom = repo.join("skills/bom-skill");
    std::fs::create_dir_all(&bom).expect("bom skill dir");
    std::fs::write(
        bom.join("SKILL.md"),
        "\u{feff}---\nname: bom-skill\ndescription: BOM skill description.\n---\n\n# BOM\n",
    )
    .expect("bom skill");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    let bom_skill = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "bom-skill")
        .expect("bom skill entry");
    assert_eq!(bom_skill["name"], "bom-skill");
    assert_eq!(bom_skill["validation"]["valid"], true);
}

#[test]
fn skills_inventory_accepts_indented_frontmatter_keys() {
    let temp = tempdir().expect("tempdir");
    let repo = write_skill_inventory_repo(temp.path());
    let indented = repo.join("skills/indented-skill");
    std::fs::create_dir_all(&indented).expect("indented skill dir");
    std::fs::write(
        indented.join("SKILL.md"),
        r#"---
  name: indented-skill
  description: Indented skill description.
  metadata:
    category: test
---

# Indented
"#,
    )
    .expect("indented skill");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    let indented_skill = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "indented-skill")
        .expect("indented skill entry");
    assert_eq!(indented_skill["name"], "indented-skill");
    assert_eq!(indented_skill["description"], "Indented skill description.");
    assert_eq!(indented_skill["validation"]["valid"], true);
    assert_eq!(indented_skill["metadata_present"], true);
}

#[cfg(unix)]
#[test]
fn skills_inventory_ignores_symlinked_skill_and_resource_paths() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().expect("tempdir");
    let repo = write_skill_inventory_repo(temp.path());
    let external_skill = temp.path().join("external-skill");
    std::fs::create_dir_all(&external_skill).expect("external skill dir");
    std::fs::write(
        external_skill.join("SKILL.md"),
        r#"---
name: linked-skill
description: Linked skill.
---

# Linked
"#,
    )
    .expect("external skill");
    symlink(&external_skill, repo.join("skills/linked-skill")).expect("skill symlink");

    let external_assets = temp.path().join("external-assets");
    std::fs::create_dir_all(&external_assets).expect("external assets dir");
    std::fs::write(external_assets.join("outside.txt"), "outside").expect("external asset");
    let alpha_assets = repo.join("skills/alpha-skill/assets");
    symlink(&external_assets, &alpha_assets).expect("resource symlink");
    let external_readme = temp.path().join("external-readme.md");
    std::fs::write(&external_readme, "`alpha-skill`").expect("external readme");
    std::fs::remove_file(repo.join("README.md")).expect("remove readme");
    symlink(&external_readme, repo.join("README.md")).expect("readme symlink");
    let external_package = temp.path().join("external-package.skill");
    std::fs::write(&external_package, "outside package").expect("external package");
    symlink(&external_package, repo.join("skills/dist/beta-skill.skill")).expect("package symlink");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    assert_eq!(json["result"]["total"], 2);
    assert!(
        json["result"]["skills"]
            .as_array()
            .expect("skills")
            .iter()
            .all(|skill| skill["directory"] != "linked-skill")
    );
    let alpha = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "alpha-skill")
        .expect("alpha skill entry");
    assert_eq!(alpha["resources"]["assets"]["present"], true);
    assert_eq!(alpha["resources"]["assets"]["files"], 0);
    assert_eq!(alpha["resources"]["assets"]["capped"], true);
    assert_eq!(alpha["exposure"]["readme_catalog"], false);
    assert!(
        !alpha["underbuilt_signals"]
            .as_array()
            .expect("alpha signals")
            .iter()
            .any(|signal| signal.as_str() == Some("missing_readme_catalog"))
    );
    let beta = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "beta-skill")
        .expect("beta skill entry");
    assert_eq!(beta["package"]["present"], false);
    assert_eq!(beta["package"]["rejected"], true);
    assert!(
        !beta["underbuilt_signals"]
            .as_array()
            .expect("beta signals")
            .iter()
            .any(|signal| signal.as_str() == Some("missing_dist_package"))
    );
    assert!(
        json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "catalog_input_symlink")
    );
    assert!(
        json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "skill_directory_symlink"
                && diagnostic["skill"] == "linked-skill")
    );
    assert!(
        json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(
                |diagnostic| diagnostic["code"] == "resource_directory_symlink"
                    && diagnostic["skill"] == "alpha-skill"
            )
    );
    assert!(
        json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "package_path_symlink"
                && diagnostic["skill"] == "beta-skill")
    );
}

#[cfg(unix)]
#[test]
fn skills_inventory_rejects_symlinked_skills_root_without_missing_root_signal() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let external_skills = temp.path().join("external-skills");
    std::fs::create_dir_all(repo.join("docs/runbooks")).expect("repo dir");
    std::fs::create_dir_all(&external_skills).expect("external skills dir");
    std::fs::write(repo.join("Cargo.toml"), "[workspace]\n").expect("repo Cargo.toml");
    std::fs::write(repo.join("docs/runbooks/validation.md"), "# Validation\n")
        .expect("repo validation");
    symlink(&external_skills, repo.join("skills")).expect("skills root symlink");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    assert_eq!(json["ok"], false);
    assert_eq!(json["result"]["ok"], false);
    assert_eq!(json["result"]["total"], 0);
    assert!(
        json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "unsafe_skills_root")
    );
    assert!(
        !json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "missing_skills_root")
    );
}

#[cfg(unix)]
#[test]
fn skills_inventory_reports_unreadable_skill_entrypoint_without_aborting() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().expect("tempdir");
    let repo = write_skill_inventory_repo(temp.path());
    let skill_md = repo.join("skills/beta-skill/SKILL.md");
    std::fs::set_permissions(&skill_md, std::fs::Permissions::from_mode(0o000))
        .expect("lock skill entrypoint");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    std::fs::set_permissions(&skill_md, std::fs::Permissions::from_mode(0o600))
        .expect("restore skill entrypoint");
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    assert_eq!(json["command"], "skills inventory");
    assert_eq!(json["result"]["schema"], "skill_inventory.v1");
    assert_eq!(json["result"]["invalid"], 1);
    let beta = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "beta-skill")
        .expect("beta skill entry");
    assert_eq!(beta["validation"]["valid"], false);
    assert!(
        beta["validation"]["errors"]
            .as_array()
            .expect("beta errors")
            .iter()
            .any(|error| error
                .as_str()
                .expect("error")
                .contains("failed to read skill entrypoint"))
    );
    assert!(
        json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(
                |diagnostic| diagnostic["code"] == "skill_entrypoint_read_error"
                    && diagnostic["skill"] == "beta-skill"
            )
    );
}

#[cfg(unix)]
#[test]
fn skills_inventory_reports_catalog_read_errors() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().expect("tempdir");
    let repo = write_skill_inventory_repo(temp.path());
    let docs_index = repo.join("docs/index.md");
    std::fs::set_permissions(&docs_index, std::fs::Permissions::from_mode(0o000))
        .expect("lock docs index");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    std::fs::set_permissions(&docs_index, std::fs::Permissions::from_mode(0o600))
        .expect("restore docs index");
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    assert!(
        json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "catalog_input_read_error")
    );
    for skill in json["result"]["skills"].as_array().expect("skills") {
        assert!(
            !skill["underbuilt_signals"]
                .as_array()
                .expect("signals")
                .iter()
                .any(|signal| signal.as_str() == Some("missing_docs_index_exposure"))
        );
    }
}

#[test]
fn skills_inventory_marks_capped_resource_counts() {
    let temp = tempdir().expect("tempdir");
    let repo = write_skill_inventory_repo(temp.path());
    let mut nested = repo.join("skills/beta-skill/references");
    for index in 0..17 {
        nested = nested.join(format!("level-{index}"));
        std::fs::create_dir_all(&nested).expect("nested resource dir");
    }

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    let beta = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "beta-skill")
        .expect("beta skill entry");
    assert_eq!(beta["resources"]["references"]["present"], true);
    assert_eq!(beta["resources"]["references"]["capped"], true);
}

#[cfg(unix)]
#[test]
fn skills_inventory_reports_unreadable_resource_counts() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().expect("tempdir");
    let repo = write_skill_inventory_repo(temp.path());
    let references = repo.join("skills/beta-skill/references");
    std::fs::create_dir_all(&references).expect("beta references dir");
    std::fs::set_permissions(&references, std::fs::Permissions::from_mode(0o000))
        .expect("lock beta references dir");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "skills",
            "inventory",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T08:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    std::fs::set_permissions(&references, std::fs::Permissions::from_mode(0o700))
        .expect("restore beta references dir");
    let json: Value = serde_json::from_slice(&output).expect("skills inventory json");
    let beta = json["result"]["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|skill| skill["directory"] == "beta-skill")
        .expect("beta skill entry");
    assert_eq!(beta["resources"]["references"]["present"], true);
    assert_eq!(beta["resources"]["references"]["capped"], true);
    assert!(
        json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "resource_count_failed"
                && diagnostic["skill"] == "beta-skill")
    );
}

#[cfg(unix)]
#[test]
fn local_doctor_emits_read_only_json_report() {
    let temp = tempdir().expect("tempdir");
    let (repo, bin) = write_local_doctor_fixture(temp.path());
    let home = temp.path().join("home");
    std::fs::create_dir_all(home.join(".config/gh")).expect("gh config dir");
    std::fs::write(home.join(".config/gh/hosts.yml"), "github.com: {}\n").expect("gh config");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", bin)
        .env("HOME", home)
        .env("GH_TOKEN", "fixture-secret-token")
        .env_remove("GITHUB_TOKEN")
        .env_remove("GH_ENTERPRISE_TOKEN")
        .env_remove("GITHUB_ENTERPRISE_TOKEN")
        .args([
            "--json",
            "local",
            "doctor",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("local doctor json");
    assert_eq!(json["command"], "local doctor");
    assert_eq!(json["result"]["schema"], "codex-dev.local-doctor.v1");
    assert_eq!(json["result"]["mode"], "doctor");
    assert_eq!(json["result"]["ok"], true);
    assert_eq!(json["result"]["github"]["auth_class"], "env_token");
    let token_sources = json["result"]["github"]["token_sources"]
        .as_array()
        .expect("token sources");
    assert!(
        token_sources
            .iter()
            .any(|source| source.as_str() == Some("GH_TOKEN")),
        "local doctor should report GH_TOKEN as the categorical token source"
    );
    assert!(
        !String::from_utf8_lossy(&output).contains("fixture-secret-token"),
        "local doctor output must not include token values"
    );
    assert_eq!(json["result"]["capsule_root"]["git_ignored"], true);
    assert!(
        json["result"]["policy_profiles"]
            .as_array()
            .expect("profiles")
            .iter()
            .any(|profile| profile["profile"] == "full_local")
    );
}

#[cfg(unix)]
#[test]
fn local_status_uses_same_contract_with_status_mode() {
    let temp = tempdir().expect("tempdir");
    let (repo, bin) = write_local_doctor_fixture(temp.path());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", bin)
        .env("HOME", temp.path().join("home"))
        .env("GH_TOKEN", "fixture-secret-token")
        .env_remove("GITHUB_TOKEN")
        .env_remove("GH_ENTERPRISE_TOKEN")
        .env_remove("GITHUB_ENTERPRISE_TOKEN")
        .args([
            "--json",
            "local",
            "status",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("local status json");
    assert_eq!(json["command"], "local status");
    assert_eq!(json["result"]["schema"], "codex-dev.local-doctor.v1");
    assert_eq!(json["result"]["mode"], "status");
    assert_eq!(json["result"]["github"]["auth_class"], "env_token");
    let token_sources = json["result"]["github"]["token_sources"]
        .as_array()
        .expect("token sources");
    assert!(
        token_sources
            .iter()
            .any(|source| source.as_str() == Some("GH_TOKEN")),
        "local status should report GH_TOKEN as the categorical token source"
    );
    assert!(
        !String::from_utf8_lossy(&output).contains("fixture-secret-token"),
        "local status output must not include token values"
    );
}

#[cfg(unix)]
#[test]
fn local_doctor_reports_enterprise_token_source_without_value() {
    let temp = tempdir().expect("tempdir");
    let (repo, bin) = write_local_doctor_fixture(temp.path());

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", bin)
        .env("HOME", temp.path().join("home"))
        .env_remove("GH_TOKEN")
        .env_remove("GITHUB_TOKEN")
        .env("GH_ENTERPRISE_TOKEN", "fixture-enterprise-secret-token")
        .env_remove("GITHUB_ENTERPRISE_TOKEN")
        .args([
            "--json",
            "local",
            "doctor",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("local doctor json");
    assert_eq!(json["result"]["github"]["auth_class"], "env_token");
    let token_sources = json["result"]["github"]["token_sources"]
        .as_array()
        .expect("token sources");
    assert!(
        token_sources
            .iter()
            .any(|source| source.as_str() == Some("GH_ENTERPRISE_TOKEN")),
        "local doctor should report GH_ENTERPRISE_TOKEN as the categorical token source"
    );
    assert!(
        !String::from_utf8_lossy(&output).contains("fixture-enterprise-secret-token"),
        "local doctor output must not include enterprise token values"
    );
}

#[cfg(unix)]
#[test]
fn local_doctor_strict_global_binaries_upgrades_missing_binary_to_error() {
    let temp = tempdir().expect("tempdir");
    let (repo, bin) = write_local_doctor_fixture(temp.path());
    std::fs::remove_file(bin.join("codex-dev-tui")).expect("remove global binary fixture");

    let default_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", &bin)
        .env("HOME", temp.path().join("home"))
        .args([
            "--json",
            "local",
            "doctor",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let default_json: Value =
        serde_json::from_slice(&default_output).expect("default local doctor json");
    let default_diagnostics = default_json["result"]["diagnostics"]
        .as_array()
        .expect("default diagnostics");
    let default_missing_binary = default_diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"] == "missing_optional_codex_dev_tui")
        .expect("default missing binary warning");
    assert_eq!(default_json["result"]["ok"], true);
    assert_eq!(default_missing_binary["severity"], "warning");

    let strict_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", &bin)
        .env("HOME", temp.path().join("home"))
        .args([
            "--json",
            "local",
            "doctor",
            "--strict-global-binaries",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T05:00:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let strict_json: Value =
        serde_json::from_slice(&strict_output).expect("strict local doctor json");
    let strict_diagnostics = strict_json["result"]["diagnostics"]
        .as_array()
        .expect("strict diagnostics");
    let strict_missing_binary = strict_diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"] == "missing_codex_dev_tui")
        .expect("strict missing binary error");
    assert_eq!(strict_json["result"]["ok"], false);
    assert_eq!(strict_missing_binary["severity"], "error");
}

#[cfg(unix)]
#[test]
fn local_doctor_reports_unknown_capsule_ignore_probe() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().expect("tempdir");
    let (repo, bin) = write_local_doctor_fixture(temp.path());
    let git = bin.join("git");
    std::fs::write(
        &git,
        r#"#!/bin/sh
if [ -n "$GH_TOKEN" ] || [ -n "$GITHUB_TOKEN" ]; then
  echo "token env leaked into git probe" >&2
  exit 7
fi
case "$*" in
  *"check-ignore"*) exit 2 ;;
  *"--version"*) printf 'git version fixture\n' ;;
  *) exit 0 ;;
esac
"#,
    )
    .expect("write failing git fixture");
    let mut perms = std::fs::metadata(&git)
        .expect("failing git metadata")
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&git, perms).expect("failing git executable");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", bin)
        .env("HOME", temp.path().join("home"))
        .args([
            "--json",
            "local",
            "doctor",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T05:00:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("local doctor json");
    let diagnostics = json["result"]["diagnostics"]
        .as_array()
        .expect("diagnostics");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic["code"] == "capsule_root_ignore_unknown")
    );
    assert!(
        !diagnostics
            .iter()
            .any(|diagnostic| diagnostic["code"] == "capsule_root_not_ignored")
    );
}

#[cfg(unix)]
#[test]
fn local_doctor_reports_unignored_cache_roots() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().expect("tempdir");
    let (repo, bin) = write_local_doctor_fixture(temp.path());
    let git = bin.join("git");
    std::fs::write(
        &git,
        r#"#!/bin/sh
if [ -n "$GH_TOKEN" ] || [ -n "$GITHUB_TOKEN" ] || [ -n "$GH_ENTERPRISE_TOKEN" ] || [ -n "$GITHUB_ENTERPRISE_TOKEN" ]; then
  echo "token env leaked into git probe" >&2
  exit 7
fi
case "$*" in
  *".codex/tasks/probe"*) exit 0 ;;
  *".codex/research/probe"*) exit 1 ;;
  *".local-cache/codex-research/probe"*) exit 1 ;;
  *"target/codex-dev-install-smoke/probe"*) exit 0 ;;
  *"check-ignore"*) exit 0 ;;
  *"--version"*) printf 'git version fixture\n' ;;
  *) exit 0 ;;
esac
"#,
    )
    .expect("write cache git fixture");
    let mut perms = std::fs::metadata(&git)
        .expect("cache git metadata")
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&git, perms).expect("cache git executable");
    let xdg_cache_home = std::path::PathBuf::from(".local-cache");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", bin)
        .env("HOME", temp.path().join("home"))
        .env("XDG_CACHE_HOME", &xdg_cache_home)
        .args([
            "--json",
            "local",
            "doctor",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T05:00:00Z",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("local doctor json");
    let diagnostics = json["result"]["diagnostics"]
        .as_array()
        .expect("diagnostics");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic["code"] == "research_cache_not_ignored")
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic["code"] == "global_codex_cache_not_ignored")
    );
}

#[cfg(unix)]
#[test]
fn local_doctor_honors_gh_config_dir_and_xdg_cache_home() {
    let temp = tempdir().expect("tempdir");
    let (repo, bin) = write_local_doctor_fixture(temp.path());
    let gh_config = temp.path().join("gh-config");
    let xdg_cache_home = temp.path().join("xdg-cache");
    let codex_cache = xdg_cache_home.join("codex-research");
    std::fs::create_dir_all(&gh_config).expect("gh config dir");
    std::fs::create_dir_all(&codex_cache).expect("codex cache dir");
    std::fs::write(gh_config.join("hosts.yml"), "github.com: {}\n").expect("gh hosts");

    let output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .env("PATH", bin)
        .env("GH_CONFIG_DIR", &gh_config)
        .env("XDG_CACHE_HOME", &xdg_cache_home)
        .env_remove("HOME")
        .env_remove("GH_TOKEN")
        .env_remove("GITHUB_TOKEN")
        .env_remove("GH_ENTERPRISE_TOKEN")
        .env_remove("GITHUB_ENTERPRISE_TOKEN")
        .args([
            "--json",
            "local",
            "doctor",
            "--repo-root",
            repo.to_str().expect("repo path"),
            "--checked-at",
            "2026-05-12T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).expect("local doctor json");
    assert_eq!(json["result"]["github"]["auth_class"], "gh_config");
    let cache_roots = json["result"]["cache_roots"]
        .as_array()
        .expect("cache roots");
    let global_cache = cache_roots
        .iter()
        .find(|root| root["name"] == "global_codex_cache")
        .expect("global cache root");
    assert_eq!(
        global_cache["path"].as_str().expect("global cache path"),
        codex_cache.to_str().expect("cache path utf8")
    );
    assert_eq!(global_cache["exists"], true);
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

    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("repo root");
    let explain_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "policy",
            "explain",
            "--profile",
            "codex_dev",
            "--repo-root",
            repo_root.to_str().expect("utf8 repo root"),
            "--checked-at",
            "2026-05-09T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let explain_json: Value = serde_json::from_slice(&explain_output).expect("explain json");
    assert_eq!(explain_json["command"], "policy explain");
    assert_eq!(explain_json["result"]["schema"], "policy_explain.v1");
    assert_eq!(explain_json["result"]["docs_mirror"]["status"], "current");
    assert!(
        explain_json["result"]["gates"]
            .as_array()
            .expect("explain gates")
            .iter()
            .any(|gate| gate["id"] == "codex-dev-policy-explain")
    );

    let bad_repo_root = repo_root.join("no-such-policy-explain-root");
    let bad_repo_root_arg = bad_repo_root.to_str().expect("utf8 bad repo root");
    let explain_error_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "policy",
            "explain",
            "--profile",
            "codex_dev",
            "--repo-root",
            bad_repo_root_arg,
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let explain_error_json: Value =
        serde_json::from_slice(&explain_error_output).expect("explain error json");
    let explain_error_message = explain_error_json["result"]["error"]["message"]
        .as_str()
        .expect("explain error message");
    assert!(explain_error_message.contains("failed to canonicalize repo root"));
    assert!(explain_error_message.contains("--include-local-paths"));
    assert!(!explain_error_message.contains(bad_repo_root_arg));

    let explain_error_with_paths_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "policy",
            "explain",
            "--profile",
            "codex_dev",
            "--repo-root",
            bad_repo_root_arg,
            "--include-local-paths",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let explain_error_with_paths_json: Value =
        serde_json::from_slice(&explain_error_with_paths_output)
            .expect("explain error with paths json");
    let explain_error_with_paths_message =
        explain_error_with_paths_json["result"]["error"]["message"]
            .as_str()
            .expect("explain error with paths message");
    assert!(explain_error_with_paths_message.contains(bad_repo_root_arg));

    let missing_manifest_root = temp.path().join("policy-explain-missing-manifest-root");
    std::fs::create_dir(&missing_manifest_root).expect("missing manifest root");
    let missing_manifest_arg = missing_manifest_root
        .to_str()
        .expect("utf8 missing manifest root");
    let missing_manifest_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "policy",
            "explain",
            "--profile",
            "codex_dev",
            "--repo-root",
            missing_manifest_arg,
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let missing_manifest_json: Value =
        serde_json::from_slice(&missing_manifest_output).expect("missing manifest error json");
    let missing_manifest_message = missing_manifest_json["result"]["error"]["message"]
        .as_str()
        .expect("missing manifest error message");
    assert!(missing_manifest_message.contains("repo root must contain Cargo.toml"));
    assert!(missing_manifest_message.contains("--include-local-paths"));
    assert!(!missing_manifest_message.contains(missing_manifest_arg));

    let missing_manifest_with_paths_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "policy",
            "explain",
            "--profile",
            "codex_dev",
            "--repo-root",
            missing_manifest_arg,
            "--include-local-paths",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let missing_manifest_with_paths_json: Value =
        serde_json::from_slice(&missing_manifest_with_paths_output)
            .expect("missing manifest with paths json");
    let missing_manifest_with_paths_message =
        missing_manifest_with_paths_json["result"]["error"]["message"]
            .as_str()
            .expect("missing manifest with paths message");
    assert!(missing_manifest_with_paths_message.contains(missing_manifest_arg));

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
fn research_import_bundle_records_sanitized_evidence() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let bundle = write_research_bundle_fixture(temp.path(), "codex-research.evidence-bundle.v1");

    let init_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "Research import smoke",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "research-import-smoke",
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

    let import_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "research",
            "import-bundle",
            "--capsule",
            capsule,
            "--bundle",
            bundle.to_str().expect("bundle path"),
            "--source-command",
            "codex-research --json bundle --strict",
            "--source-exit-code",
            "0",
            "--imported-at",
            "2026-05-11T13:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let import_json: Value = serde_json::from_slice(&import_output).expect("import json");
    assert_eq!(import_json["command"], "research import-bundle");
    assert_eq!(
        import_json["result"]["schema"],
        "research_evidence_import.v1"
    );
    assert_eq!(import_json["result"]["bundle"]["status"], "passed");
    assert_eq!(import_json["result"]["bundle"]["budget"]["status"], "spent");
    assert_eq!(import_json["result"]["bundle"]["source_count"], 2);
    assert_eq!(import_json["result"]["bundle"]["claim_count"], 2);
    assert_eq!(import_json["result"]["record"]["kind"], "research");
    assert_eq!(import_json["result"]["record"]["tool"], "codex-research");
    assert_eq!(import_json["result"]["record"]["confidence"], 100);
    assert_eq!(
        import_json["result"]["record"]["source_ids"][0],
        "codex-research:claim:claim-official-docs-first"
    );
    assert!(
        import_json["result"]["record"]["source_ids"]
            .as_array()
            .expect("source ids")
            .iter()
            .any(|source| source == "codex-research:source:src-github-source")
    );
    assert!(
        import_json["result"]["record"]["artifacts"]
            .as_array()
            .expect("artifacts")
            .iter()
            .any(|artifact| artifact == ".codex/research/report.md")
    );
    assert_eq!(import_json["result"]["evidence"]["total"], 2);

    let evidence = std::fs::read_to_string(std::path::Path::new(capsule).join("evidence.jsonl"))
        .expect("evidence");
    let research_line = evidence
        .lines()
        .find(|line| line.contains("\"kind\":\"research\""))
        .expect("research evidence line");
    let research_record: Value = serde_json::from_str(research_line).expect("research record");
    assert!(
        research_record["summary"]
            .as_str()
            .expect("summary")
            .contains("Research bundle passed")
    );
}

#[test]
fn research_import_bundle_rejects_wrong_schema_before_writing() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let bundle = write_research_bundle_fixture(
        temp.path(),
        "unexpected.v1 token=wrong-schema-secret sk-proj-wrong-schema-secret",
    );

    let init_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "Research import invalid schema",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "research-import-invalid",
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
    let evidence_before =
        std::fs::read_to_string(std::path::Path::new(capsule).join("evidence.jsonl"))
            .expect("evidence before");

    let import_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "research",
            "import-bundle",
            "--capsule",
            capsule,
            "--bundle",
            bundle.to_str().expect("bundle path"),
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let import_json: Value = serde_json::from_slice(&import_output).expect("import error json");
    assert_eq!(import_json["ok"], false);
    assert_eq!(import_json["command"], "research import-bundle");
    assert!(
        import_json["result"]["error"]["message"]
            .as_str()
            .expect("error message")
            .contains("unsupported evidence bundle schema")
    );
    let output_text = String::from_utf8(import_output).expect("utf8 error output");
    assert!(!output_text.contains("wrong-schema-secret"));
    assert!(output_text.contains("[redacted]"));
    assert_eq!(
        std::fs::read_to_string(std::path::Path::new(capsule).join("evidence.jsonl"))
            .expect("evidence after"),
        evidence_before
    );
}

#[test]
fn research_import_bundle_redacts_secrets_and_caps_untrusted_arrays() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join("tasks");
    let bundle = temp.path().join("large-evidence-bundle.json");
    let source_ids = (0..140)
        .map(|index| format!("src-{index:03}-token=source-secret"))
        .collect::<Vec<_>>();
    let claim_ids = (0..140)
        .map(|index| format!("claim-{index:03}-api_key=claim-secret"))
        .collect::<Vec<_>>();
    let artifacts = (0..80)
        .map(|index| format!(".codex/research/artifact-{index:03}.json?api_key=artifact-secret"))
        .collect::<Vec<_>>();
    let providers = (0..40)
        .map(|index| {
            json!({
                "provider": format!("provider-{index:02}"),
                "budget": 10,
                "spent": 1,
                "remaining": 9
            })
        })
        .collect::<Vec<_>>();
    let freshness = (0..40)
        .map(|index| (format!("status-{index:02}-token=fresh-secret"), json!(1)))
        .collect::<serde_json::Map<_, _>>();
    let warnings = (0..25)
        .map(|index| format!("warning {index}: Authorization: Bearer ghp_warningsecret"))
        .collect::<Vec<_>>();
    let failures = (0..25)
        .map(|index| format!("failure {index}: body=raw-provider-payload"))
        .collect::<Vec<_>>();
    std::fs::write(
        &bundle,
        serde_json::to_vec_pretty(&json!({
            "schema": "codex-research.evidence-bundle.v1",
            "generated_at": "2026-05-11T12:00:00Z",
            "status": "failed",
            "strict": true,
            "run": {
                "query": "audit token=super-secret sk-proj-openai-secret",
                "profile": "deep",
                "topic": "dependency",
                "status": "closed",
                "cache_source_ids": source_ids.clone()
            },
            "budget": {"by_provider": providers},
            "provider_errors": [
                {"provider": "github", "message": "body=private-provider-payload OPENAI_API_KEY=openai-secret"}
            ],
            "ledger": {
                "source_count": 140,
                "claim_count": 140,
                "source_ids": source_ids.clone(),
                "claim_ids": claim_ids.clone()
            },
            "citation_coverage": {
                "cited_claims": 120,
                "uncited_claims": 20,
                "uncited_claim_ids": claim_ids.clone(),
                "missing_source_refs": source_ids.clone(),
                "coverage": 0.85
            },
            "source_freshness": {
                "by_status": freshness,
                "unknown_source_ids": source_ids
            },
            "report": {
                "path": ".codex/research/report.md?access_token=report-secret",
                "exists": true
            },
            "artifacts": artifacts,
            "warnings": warnings,
            "failures": failures
        }))
        .expect("serialize large bundle"),
    )
    .expect("write large bundle");

    let init_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "capsule",
            "init",
            "--title",
            "Research import untrusted bundle",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "research-import-untrusted",
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

    let import_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "research",
            "import-bundle",
            "--capsule",
            capsule,
            "--bundle",
            bundle.to_str().expect("bundle path"),
            "--imported-at",
            "2026-05-11T13:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(import_output.clone()).expect("utf8 output");
    for secret in [
        "super-secret",
        "sk-proj-openai-secret",
        "source-secret",
        "claim-secret",
        "artifact-secret",
        "fresh-secret",
        "ghp_warningsecret",
        "raw-provider-payload",
        "private-provider-payload",
        "openai-secret",
        "report-secret",
    ] {
        assert!(
            !output_text.contains(secret),
            "output leaked secret fragment: {secret}"
        );
    }
    assert!(output_text.contains("[redacted]"));

    let import_json: Value = serde_json::from_slice(&import_output).expect("import json");
    assert!(
        import_json["result"]["record"]["source_ids"]
            .as_array()
            .expect("source ids")
            .len()
            <= 200
    );
    assert!(
        import_json["result"]["record"]["artifacts"]
            .as_array()
            .expect("artifacts")
            .len()
            <= 50
    );
    assert!(
        import_json["result"]["bundle"]["budget"]["providers"]
            .as_array()
            .expect("providers")
            .len()
            <= 32
    );
    assert!(
        import_json["result"]["bundle"]["source_freshness"]
            .as_object()
            .expect("freshness")
            .len()
            <= 32
    );
    assert_eq!(
        import_json["result"]["bundle"]["warnings"]
            .as_array()
            .expect("warnings")
            .len(),
        20
    );
    assert_eq!(
        import_json["result"]["bundle"]["failures"]
            .as_array()
            .expect("failures")
            .len(),
        20
    );
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
fn orchestration_plan_record_close_and_verify() {
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
            "Orchestration ledger smoke",
            "--root",
            root.to_str().expect("utf8 temp path"),
            "--id",
            "orchestration-smoke",
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
            "orchestration",
            "plan",
            "--capsule",
            capsule,
            "--batch-id",
            "pre-pr-review",
            "--source",
            plan.to_str().expect("utf8 plan path"),
            "--recorded-at",
            "2026-05-09T05:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let plan_json: Value = serde_json::from_slice(&plan_output).expect("plan json");
    assert_eq!(plan_json["command"], "orchestration plan");
    assert_eq!(plan_json["result"]["schema"], "orchestration_run.v1");
    assert_eq!(plan_json["result"]["completion"]["expected"], 2);
    assert!(
        plan_json["result"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "incomplete_agent")
    );

    Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "orchestration",
            "verify",
            "--capsule",
            capsule,
            "--batch-id",
            "pre-pr-review",
            "--checked-at",
            "2026-05-09T06:00:00Z",
        ])
        .assert()
        .failure();

    for (role, agent_id) in [
        ("reviewer", "agent-reviewer-1"),
        ("test_runner", "agent-test-runner-1"),
    ] {
        Command::cargo_bin("codex-dev")
            .expect("binary")
            .args([
                "--json",
                "orchestration",
                "record",
                "--capsule",
                capsule,
                "--batch-id",
                "pre-pr-review",
                "--role",
                role,
                "--agent-id",
                agent_id,
                "--status",
                "completed",
                "--wait-status",
                "completed",
                "--wait-elapsed-ms",
                "1500",
                "--summary",
                "no blocking findings",
                "--disposition",
                "accepted",
                "--human-verified",
                "--source-id",
                role,
                "--artifact",
                "review-notes.md",
                "--recorded-at",
                "2026-05-09T05:10:00Z",
            ])
            .assert()
            .success();
    }

    Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "orchestration",
            "record",
            "--capsule",
            capsule,
            "--batch-id",
            "pre-pr-review",
            "--role",
            "reviewer",
            "--status",
            "completed",
            "--summary",
            "no blocking findings after follow-up",
            "--disposition",
            "accepted",
            "--human-verified",
            "--source-id",
            "reviewer",
            "--artifact",
            "review-notes.md",
            "--recorded-at",
            "2026-05-09T05:15:00Z",
        ])
        .assert()
        .success();

    let close_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "orchestration",
            "close",
            "--capsule",
            capsule,
            "--batch-id",
            "pre-pr-review",
            "--status",
            "completed",
            "--summary",
            "review batch clean",
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
    let close_json: Value = serde_json::from_slice(&close_output).expect("close json");
    assert_eq!(close_json["command"], "orchestration close");
    assert_eq!(close_json["result"]["completion"]["complete"], true);
    let reviewer = close_json["result"]["agents"]
        .as_array()
        .expect("agents")
        .iter()
        .find(|agent| agent["role"] == "reviewer")
        .expect("reviewer agent");
    assert_eq!(reviewer["agent_id"], "agent-reviewer-1");
    assert_eq!(reviewer["wait_status"], "completed");

    let verify_output = Command::cargo_bin("codex-dev")
        .expect("binary")
        .args([
            "--json",
            "orchestration",
            "verify",
            "--capsule",
            capsule,
            "--batch-id",
            "pre-pr-review",
            "--checked-at",
            "2026-05-09T06:00:00Z",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let verify_json: Value = serde_json::from_slice(&verify_output).expect("verify json");
    assert_eq!(verify_json["command"], "orchestration verify");
    assert_eq!(verify_json["result"]["diagnostics"], json!([]));
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
