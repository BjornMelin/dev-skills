use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

use anyhow::{Context, Result};
use bun_platform_core::{
    AuditConfig, CliOverrides, PlatformPaths, Severity, SkillContext, apply_safe_fixes,
    check_skill_integrity, create_release_sync_report, format_findings_text, format_fixes_text,
    load_audit_config, plan_safe_fixes, preview_release_sync, run_audit, run_release_sync,
};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand, ValueEnum};
use codex_dev_core::{
    AppendEvidenceArgs, EVIDENCE_SCHEMA, EvidenceKind, EvidenceRecord, append_evidence,
};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::CommandOutput;

const BUN_AUDIT_SCHEMA: &str = "codex-dev.bun-audit.v1";
const BUN_RULES_SCHEMA: &str = "codex-dev.bun-rules.v1";
const BUN_RULE_SCHEMA: &str = "codex-dev.bun-rule.v1";
const BUN_FIXES_SCHEMA: &str = "codex-dev.bun-fixes.v1";
const BUN_VALIDATE_SCHEMA: &str = "codex-dev.bun-validate.v1";
const BUN_REFERENCES_SCHEMA: &str = "codex-dev.bun-references.v1";
const BUN_DOCTOR_SCHEMA: &str = "codex-dev.bun-doctor.v1";
const BUN_BENCHMARK_SCHEMA: &str = "codex-dev.bun-benchmark.v1";
const EXTERNAL_TOOL_REPORT_IMPORT_SCHEMA: &str = "external_tool_report_import.v1";

#[derive(Subcommand, Debug)]
pub(crate) enum BunCommand {
    /// Audit a repository for Bun platform findings.
    Audit(BunAuditArgs),
    /// List or show bun-dev rule files.
    Rules {
        #[command(subcommand)]
        command: BunRulesCommand,
    },
    /// Plan or apply deterministic Bun safe fixes.
    Fixes {
        #[command(subcommand)]
        command: BunFixesCommand,
    },
    /// Plan or run Bun validation commands.
    Validate {
        #[command(subcommand)]
        command: BunValidateCommand,
    },
    /// Inspect or refresh vendor-backed bun-dev references.
    References {
        #[command(subcommand)]
        command: BunReferencesCommand,
    },
    /// Inspect Bun platform paths, version pins, and skill integrity.
    Doctor(BunDoctorArgs),
    /// Measure Bun platform audit and fix-planning hot paths.
    Benchmark(BunBenchmarkArgs),
}

#[derive(Subcommand, Debug)]
pub(crate) enum BunRulesCommand {
    /// Print rule identifiers from the bun-dev skill.
    List(BunSkillArgs),
    /// Print one rule file.
    Show(BunRuleShowArgs),
}

#[derive(Subcommand, Debug)]
pub(crate) enum BunFixesCommand {
    /// Plan safe fixes without mutating the repository.
    Plan(BunFixesArgs),
    /// Apply safe fixes and write rollback artifacts to external state.
    Apply(BunFixesArgs),
}

#[derive(Subcommand, Debug)]
pub(crate) enum BunValidateCommand {
    /// Print validation commands without executing them.
    Plan(BunScopeArgs),
    /// Audit and run validation commands.
    Run(BunValidateRunArgs),
}

#[derive(Subcommand, Debug)]
pub(crate) enum BunReferencesCommand {
    /// Print local reference and integrity status.
    Status(BunSkillArgs),
    /// Fetch remote docs and report changed reference files without writing.
    Plan(BunSkillArgs),
    /// Refresh tracked reference snapshots and rule indexes.
    Sync(BunSkillArgs),
}

#[derive(Subcommand, Debug)]
pub(crate) enum ToolCommand {
    /// Import a JSON report from an external tool into a task capsule.
    Import(ToolImportArgs),
}

#[derive(Args, Debug, Default)]
pub(crate) struct BunSkillArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Override bun-dev skill root. Defaults to tracked skills/bun-dev when present, then BUN_PLATFORM_SKILL_ROOT or ~/.agents/skills/bun-dev."
    )]
    skill_root: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub(crate) struct BunRuleShowArgs {
    #[arg(value_name = "RULE_ID")]
    rule_id: String,
    #[command(flatten)]
    skill: BunSkillArgs,
}

#[derive(Args, Debug, Default)]
pub(crate) struct BunScanScopeArgs {
    #[arg(long, value_name = "PATH", help = "Repository root to inspect")]
    root: Option<PathBuf>,
    #[arg(long, value_name = "PATH", help = "Path to bun-platform.config.json")]
    config: Option<PathBuf>,
    #[arg(long, value_name = "PATH", help = "Baseline suppression file")]
    baseline: Option<PathBuf>,
    #[arg(long = "include", value_name = "PATH")]
    include_paths: Vec<PathBuf>,
    #[arg(long = "exclude", value_name = "DIR")]
    exclude_dirs: Vec<String>,
    #[arg(long = "adapter", value_name = "ID")]
    adapters: Vec<String>,
    #[arg(long = "max-files", value_name = "N")]
    max_files: Option<usize>,
    #[arg(long = "max-bytes", value_name = "N")]
    max_bytes: Option<u64>,
}

#[derive(Args, Debug, Default)]
pub(crate) struct BunScopeArgs {
    #[command(flatten)]
    scan: BunScanScopeArgs,
    #[arg(
        long = "write-cache",
        help = "Write scan-cache entries under the dev-skills Bun platform cache"
    )]
    write_cache: bool,
}

#[derive(Args, Debug)]
pub(crate) struct BunAuditArgs {
    #[command(flatten)]
    scope: BunScopeArgs,
    #[arg(long = "fail-on", value_enum)]
    fail_on: Option<BunSeverityArg>,
}

#[derive(Args, Debug)]
pub(crate) struct BunFixesArgs {
    #[command(flatten)]
    scope: BunScopeArgs,
    #[arg(
        long = "full-content",
        help = "Include complete before/after file content"
    )]
    full_content: bool,
}

#[derive(Args, Debug)]
pub(crate) struct BunValidateRunArgs {
    #[command(flatten)]
    scope: BunScopeArgs,
    #[arg(long = "fail-on", value_enum, default_value_t = BunSeverityArg::Warn)]
    fail_on: BunSeverityArg,
}

#[derive(Args, Debug)]
pub(crate) struct BunDoctorArgs {
    #[command(flatten)]
    skill: BunSkillArgs,
}

#[derive(Args, Debug)]
pub(crate) struct BunBenchmarkArgs {
    #[command(flatten)]
    scope: BunScanScopeArgs,
    #[arg(long, default_value_t = 3, value_name = "N")]
    iterations: u32,
}

#[derive(Args, Debug)]
pub(crate) struct ToolImportArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    capsule: PathBuf,
    #[arg(long, value_name = "TOOL_NAME")]
    tool: String,
    #[arg(long, value_name = "REPORT_JSON")]
    report: PathBuf,
    #[arg(long, value_name = "KIND", default_value_t = EvidenceKind::Output)]
    kind: EvidenceKind,
    #[arg(long)]
    summary: Option<String>,
    #[arg(long = "source-command", value_name = "COMMAND")]
    source_command: Option<String>,
    #[arg(
        long = "source-exit-code",
        value_name = "EXIT_CODE",
        requires = "source_command"
    )]
    source_exit_code: Option<i32>,
    #[arg(long = "imported-at", value_name = "RFC3339")]
    imported_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum BunSeverityArg {
    Error,
    Warn,
    Info,
}

pub(crate) fn bun_command_name(command: &BunCommand) -> &'static str {
    match command {
        BunCommand::Audit(_) => "bun audit",
        BunCommand::Rules { command } => match command {
            BunRulesCommand::List(_) => "bun rules list",
            BunRulesCommand::Show(_) => "bun rules show",
        },
        BunCommand::Fixes { command } => match command {
            BunFixesCommand::Plan(_) => "bun fixes plan",
            BunFixesCommand::Apply(_) => "bun fixes apply",
        },
        BunCommand::Validate { command } => match command {
            BunValidateCommand::Plan(_) => "bun validate plan",
            BunValidateCommand::Run(_) => "bun validate run",
        },
        BunCommand::References { command } => match command {
            BunReferencesCommand::Status(_) => "bun references status",
            BunReferencesCommand::Plan(_) => "bun references plan",
            BunReferencesCommand::Sync(_) => "bun references sync",
        },
        BunCommand::Doctor(_) => "bun doctor",
        BunCommand::Benchmark(_) => "bun benchmark",
    }
}

pub(crate) fn tool_command_name(command: &ToolCommand) -> &'static str {
    match command {
        ToolCommand::Import(_) => "tool import",
    }
}

pub(crate) fn handle_bun_command(command: BunCommand) -> Result<CommandOutput> {
    match command {
        BunCommand::Audit(args) => bun_audit(args),
        BunCommand::Rules { command } => match command {
            BunRulesCommand::List(args) => bun_rules_list(args),
            BunRulesCommand::Show(args) => bun_rules_show(args),
        },
        BunCommand::Fixes { command } => match command {
            BunFixesCommand::Plan(args) => bun_fixes(args, false),
            BunFixesCommand::Apply(args) => bun_fixes(args, true),
        },
        BunCommand::Validate { command } => match command {
            BunValidateCommand::Plan(args) => bun_validate_plan(args),
            BunValidateCommand::Run(args) => bun_validate_run(args),
        },
        BunCommand::References { command } => match command {
            BunReferencesCommand::Status(args) => bun_references_status(args),
            BunReferencesCommand::Plan(args) => bun_references_plan(args),
            BunReferencesCommand::Sync(args) => bun_references_sync(args),
        },
        BunCommand::Doctor(args) => bun_doctor(args),
        BunCommand::Benchmark(args) => bun_benchmark(args),
    }
}

pub(crate) fn handle_tool_command(command: ToolCommand) -> Result<CommandOutput> {
    match command {
        ToolCommand::Import(args) => tool_import(args),
    }
}

fn bun_audit(args: BunAuditArgs) -> Result<CommandOutput> {
    let (root, config) = build_bun_config(&args.scope.scan, args.scope.write_cache)?;
    let paths = PlatformPaths::discover()?;
    let findings = run_audit(&root, &config, &paths)?;
    let failed = args
        .fail_on
        .map(map_bun_severity)
        .map(|severity| bun_platform_core::should_fail(&findings, severity))
        .unwrap_or(false);
    Ok(CommandOutput {
        ok: !failed,
        command: "bun audit",
        human: format_findings_text(&findings),
        result: json!({
            "schema": BUN_AUDIT_SCHEMA,
            "root": root,
            "write_cache": config.write_cache,
            "finding_count": findings.len(),
            "findings": findings,
        }),
    })
}

fn bun_rules_list(args: BunSkillArgs) -> Result<CommandOutput> {
    let context = resolve_bun_skill_context(args.skill_root)?;
    let rule_ids = context.list_rule_ids()?;
    Ok(CommandOutput {
        ok: true,
        command: "bun rules list",
        human: format!("{}\n", rule_ids.join("\n")),
        result: json!({
            "schema": BUN_RULES_SCHEMA,
            "skill_root": context.skill_root,
            "rule_ids": rule_ids,
        }),
    })
}

fn bun_rules_show(args: BunRuleShowArgs) -> Result<CommandOutput> {
    let context = resolve_bun_skill_context(args.skill.skill_root)?;
    let markdown = context.explain_rule(&args.rule_id)?;
    Ok(CommandOutput {
        ok: true,
        command: "bun rules show",
        human: markdown.clone(),
        result: json!({
            "schema": BUN_RULE_SCHEMA,
            "skill_root": context.skill_root,
            "rule_id": args.rule_id,
            "markdown": markdown,
        }),
    })
}

fn bun_fixes(args: BunFixesArgs, apply: bool) -> Result<CommandOutput> {
    let (root, config) = build_bun_config(&args.scope.scan, args.scope.write_cache)?;
    let paths = PlatformPaths::discover()?;
    let fixes = if apply {
        apply_safe_fixes(&root, &config, &paths)?
    } else {
        plan_safe_fixes(&root, &config)?
    };
    let projected = fixes
        .iter()
        .map(|fix| ProjectedFix::from_fix(fix, args.full_content))
        .collect::<Result<Vec<_>>>()?;
    let command = if apply {
        "bun fixes apply"
    } else {
        "bun fixes plan"
    };
    Ok(CommandOutput {
        ok: true,
        command,
        human: format_fixes_text(&fixes, apply),
        result: json!({
            "schema": BUN_FIXES_SCHEMA,
            "root": root,
            "applied": apply,
            "full_content": args.full_content,
            "fix_count": projected.len(),
            "fixes": projected,
        }),
    })
}

fn bun_validate_plan(scope: BunScopeArgs) -> Result<CommandOutput> {
    let (root, config) = build_bun_config(&scope.scan, scope.write_cache)?;
    let commands = validation_commands(&root, &config)?;
    Ok(CommandOutput {
        ok: true,
        command: "bun validate plan",
        human: format!("{}\n", commands.join("\n")),
        result: json!({
            "schema": BUN_VALIDATE_SCHEMA,
            "root": root,
            "dry_run": true,
            "commands": commands,
        }),
    })
}

fn bun_validate_run(args: BunValidateRunArgs) -> Result<CommandOutput> {
    let (root, config) = build_bun_config(&args.scope.scan, args.scope.write_cache)?;
    let paths = PlatformPaths::discover()?;
    let findings = run_audit(&root, &config, &paths)?;
    let fail_on = map_bun_severity(args.fail_on);
    let audit_failed = bun_platform_core::should_fail(&findings, fail_on);
    if audit_failed {
        return Ok(CommandOutput {
            ok: false,
            command: "bun validate run",
            human: format_findings_text(&findings),
            result: json!({
                "schema": BUN_VALIDATE_SCHEMA,
                "root": root,
                "dry_run": false,
                "audit_failed": true,
                "fail_on": severity_name(fail_on),
                "finding_count": findings.len(),
                "findings": findings,
                "commands": [],
                "command_results": [],
            }),
        });
    }
    let commands = validation_commands(&root, &config)?;
    let command_results = run_validation_commands(&root, &commands)?;
    let ok = command_results.iter().all(|result| result.exit_code == 0);
    Ok(CommandOutput {
        ok,
        command: "bun validate run",
        human: if ok {
            format!("Validated {} command(s).", command_results.len())
        } else {
            format!(
                "Failed {} of {} validation command(s).",
                command_results
                    .iter()
                    .filter(|result| result.exit_code != 0)
                    .count(),
                command_results.len()
            )
        },
        result: json!({
            "schema": BUN_VALIDATE_SCHEMA,
            "root": root,
            "dry_run": false,
            "audit_failed": false,
            "fail_on": severity_name(fail_on),
            "finding_count": findings.len(),
            "findings": findings,
            "commands": commands,
            "command_results": command_results,
        }),
    })
}

fn bun_references_status(args: BunSkillArgs) -> Result<CommandOutput> {
    let context = resolve_bun_skill_context(args.skill_root)?;
    let paths = PlatformPaths::discover()?;
    let report = create_release_sync_report(&context)?;
    let integrity = check_skill_integrity(&context);
    Ok(CommandOutput {
        ok: integrity.is_ok(),
        command: "bun references status",
        human: format!(
            "bun-dev references at {}; integrity {}",
            context.skill_root.display(),
            if integrity.is_ok() { "ok" } else { "failed" }
        ),
        result: json!({
            "schema": BUN_REFERENCES_SCHEMA,
            "action": "status",
            "skill_root": context.skill_root,
            "report": report,
            "integrity_ok": integrity.is_ok(),
            "integrity_error": integrity.err().map(|error| format!("{error:#}")),
            "state_report_path": paths.reports_dir().join("release-sync-report.json"),
            "state_report_exists": paths.reports_dir().join("release-sync-report.json").is_file(),
        }),
    })
}

fn bun_references_plan(args: BunSkillArgs) -> Result<CommandOutput> {
    let context = resolve_bun_skill_context(args.skill_root)?;
    let preview = preview_release_sync(&context)?;
    let changed = preview.would_update.len();
    Ok(CommandOutput {
        ok: preview.integrity_ok,
        command: "bun references plan",
        human: format!("{changed} bun-dev reference file(s) would change"),
        result: json!({
            "schema": BUN_REFERENCES_SCHEMA,
            "action": "plan",
            "skill_root": context.skill_root,
            "preview": preview,
        }),
    })
}

fn bun_references_sync(args: BunSkillArgs) -> Result<CommandOutput> {
    let context = resolve_bun_skill_context(args.skill_root)?;
    let paths = PlatformPaths::discover()?;
    let report = run_release_sync(&context, &paths)?;
    Ok(CommandOutput {
        ok: true,
        command: "bun references sync",
        human: format!(
            "synced bun-dev references at {}",
            context.skill_root.display()
        ),
        result: json!({
            "schema": BUN_REFERENCES_SCHEMA,
            "action": "sync",
            "skill_root": context.skill_root,
            "report": report,
            "state_report_path": paths.reports_dir().join("release-sync-report.json"),
        }),
    })
}

fn bun_doctor(args: BunDoctorArgs) -> Result<CommandOutput> {
    let context = resolve_bun_skill_context(args.skill.skill_root)?;
    let paths = PlatformPaths::discover()?;
    let report = create_release_sync_report(&context)?;
    let integrity = check_skill_integrity(&context);
    let payload = json!({
        "schema": BUN_DOCTOR_SCHEMA,
        "skill_root": context.skill_root,
        "config_dir": paths.config_dir,
        "state_dir": paths.state_dir,
        "cache_dir": paths.cache_dir,
        "state_report_path": paths.reports_dir().join("release-sync-report.json"),
        "verified_bun_version": report.verified_bun_version,
        "integrity_ok": integrity.is_ok(),
        "integrity_error": integrity.err().map(|error| format!("{error:#}")),
    });
    Ok(CommandOutput {
        ok: payload["integrity_ok"].as_bool().unwrap_or(false),
        command: "bun doctor",
        human: serde_json::to_string_pretty(&payload)?,
        result: payload,
    })
}

fn bun_benchmark(args: BunBenchmarkArgs) -> Result<CommandOutput> {
    let (root, mut config) = build_bun_config(&args.scope, false)?;
    config.write_cache = false;
    let paths = PlatformPaths::discover()?;
    let iterations = args.iterations.max(1);
    let mut audit_ms = Vec::new();
    let mut plan_fix_ms = Vec::new();

    for _ in 0..iterations {
        let started = Instant::now();
        let findings = run_audit(&root, &config, &paths)?;
        audit_ms.push(started.elapsed().as_secs_f64() * 1000.0);

        let started = Instant::now();
        let fixes = plan_safe_fixes(&root, &config)?;
        plan_fix_ms.push(started.elapsed().as_secs_f64() * 1000.0);
        let _ = (findings.len(), fixes.len());
    }

    let result = json!({
        "schema": BUN_BENCHMARK_SCHEMA,
        "root": root,
        "iterations": iterations,
        "audit_ms": audit_ms,
        "plan_fix_ms": plan_fix_ms,
        "summary": {
            "audit": summarize_timings(&audit_ms),
            "plan_fixes": summarize_timings(&plan_fix_ms),
        },
    });
    Ok(CommandOutput {
        ok: true,
        command: "bun benchmark",
        human: serde_json::to_string_pretty(&result)?,
        result,
    })
}

fn tool_import(args: ToolImportArgs) -> Result<CommandOutput> {
    let imported_at = args.imported_at.unwrap_or_else(Utc::now);
    let text = fs::read_to_string(&args.report)
        .with_context(|| format!("failed to read report {}", args.report.display()))?;
    let report = serde_json::from_str::<Value>(&text)
        .with_context(|| format!("failed to parse report {}", args.report.display()))?;
    let report_hash = sha256_hex(text.as_bytes());
    let report_schema = report
        .get("schema")
        .and_then(Value::as_str)
        .or_else(|| {
            report
                .get("result")
                .and_then(|value| value.get("schema"))
                .and_then(Value::as_str)
        })
        .unwrap_or("unknown");
    let source_id = format!("external-tool:{}:{report_schema}:{report_hash}", args.tool);
    let summary = args.summary.unwrap_or_else(|| {
        format!(
            "Imported {} report {} from {}",
            args.tool,
            report_schema,
            args.report.display()
        )
    });
    let record = EvidenceRecord {
        schema: EVIDENCE_SCHEMA.to_string(),
        kind: args.kind,
        at: imported_at,
        summary,
        command: args.source_command,
        exit_code: args.source_exit_code,
        source_ids: vec![source_id],
        actor: None,
        tool: Some(args.tool.clone()),
        confidence: None,
        residual_risk: None,
        artifacts: vec![args.report.display().to_string()],
    };
    let append_result = append_evidence(AppendEvidenceArgs {
        capsule: args.capsule.clone(),
        record: record.clone(),
    })?;
    let result = json!({
        "schema": EXTERNAL_TOOL_REPORT_IMPORT_SCHEMA,
        "imported_at": imported_at,
        "capsule": args.capsule,
        "evidence_path": append_result.evidence_path,
        "tool": args.tool,
        "report_path": args.report,
        "report_schema": report_schema,
        "report_hash": report_hash,
        "record": record,
        "evidence": append_result.evidence,
    });
    Ok(CommandOutput {
        ok: true,
        command: "tool import",
        human: format!(
            "imported external tool report into {}",
            append_result.capsule.display()
        ),
        result,
    })
}

fn build_bun_config(scope: &BunScanScopeArgs, write_cache: bool) -> Result<(PathBuf, AuditConfig)> {
    let root = resolve_root(scope.root.clone())?;
    let overrides = CliOverrides {
        baseline_path: scope.baseline.clone(),
        include_paths: scope.include_paths.clone(),
        exclude_dirs: scope.exclude_dirs.clone(),
        adapters: scope.adapters.clone(),
        max_files: scope.max_files,
        max_bytes: scope.max_bytes,
        write_cache,
    };
    let config = load_audit_config(&root, scope.config.as_deref(), &overrides)?;
    Ok((root, config))
}

fn resolve_root(root: Option<PathBuf>) -> Result<PathBuf> {
    let path = root.unwrap_or(std::env::current_dir()?);
    path.canonicalize()
        .with_context(|| format!("failed to resolve {}", path.display()))
}

fn resolve_bun_skill_context(skill_root_override: Option<PathBuf>) -> Result<SkillContext> {
    if let Some(skill_root) = skill_root_override {
        return SkillContext::discover(Some(skill_root));
    }
    if let Some(repo_root) = find_repo_root(&std::env::current_dir()?) {
        let tracked = repo_root.join("skills/bun-dev");
        if tracked.is_dir() {
            return SkillContext::discover(Some(tracked));
        }
    }
    SkillContext::discover(None)
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut cargo_fallback = None;
    for ancestor in start.ancestors() {
        if ancestor.join("skills/bun-dev").is_dir() || ancestor.join(".git").exists() {
            return Some(ancestor.to_path_buf());
        }
        if cargo_fallback.is_none() && ancestor.join("Cargo.toml").is_file() {
            cargo_fallback = Some(ancestor.to_path_buf());
        }
    }
    cargo_fallback
}

fn validation_commands(root: &Path, config: &AuditConfig) -> Result<Vec<String>> {
    let package_json_path = root.join("package.json");
    let package_json = if package_json_path.is_file() {
        let text = fs::read_to_string(&package_json_path)
            .with_context(|| format!("failed to read {}", package_json_path.display()))?;
        Some(
            serde_json::from_str::<Value>(&text)
                .with_context(|| format!("failed to parse {}", package_json_path.display()))?,
        )
    } else {
        None
    };
    if !config.validation_commands.is_empty() {
        return Ok(config.validation_commands.clone());
    }
    let scripts = package_json
        .as_ref()
        .and_then(|value| value.get("scripts"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut commands = Vec::new();
    if root.join("bun.lockb").is_file() || root.join("bun.lock").is_file() {
        commands.push("bun install --frozen-lockfile".to_string());
    }
    for script in ["lint", "typecheck", "test", "build"] {
        if scripts.contains_key(script) {
            commands.push(format!("bun run {script}"));
        }
    }
    Ok(commands)
}

fn run_validation_commands(
    root: &Path,
    commands: &[String],
) -> Result<Vec<ValidationCommandResult>> {
    let mut results = Vec::new();
    let (shell, command_flag) = validation_shell();
    for command in commands {
        let status = Command::new(&shell)
            .arg(command_flag)
            .arg(command)
            .current_dir(root)
            .stdin(Stdio::null())
            .status()
            .with_context(|| format!("failed to run validation command `{command}`"))?;
        results.push(ValidationCommandResult {
            command: command.clone(),
            exit_code: status.code().unwrap_or(1),
            success: status.success(),
        });
    }
    Ok(results)
}

#[cfg(windows)]
fn validation_shell() -> (OsString, &'static str) {
    (
        std::env::var_os("COMSPEC").unwrap_or_else(|| OsString::from("cmd.exe")),
        "/C",
    )
}

#[cfg(not(windows))]
fn validation_shell() -> (OsString, &'static str) {
    (
        std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("/bin/sh")),
        "-c",
    )
}

#[derive(Debug, Serialize)]
struct ValidationCommandResult {
    command: String,
    exit_code: i32,
    success: bool,
}

#[derive(Debug, Serialize)]
struct ProjectedFix {
    rule_id: String,
    rule_ids: Vec<String>,
    kind: String,
    file: String,
    description: String,
    before_sha256: Option<String>,
    after_sha256: Option<String>,
    before_bytes: Option<usize>,
    after_bytes: Option<usize>,
    diff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<String>,
}

impl ProjectedFix {
    fn from_fix(fix: &bun_platform_core::PlannedFix, full_content: bool) -> Result<Self> {
        let before = fix.before.as_deref();
        let after = fix.after.as_deref();
        Ok(Self {
            rule_id: fix.rule_id.clone(),
            rule_ids: fix.rule_ids.clone(),
            kind: format!("{:?}", fix.kind).to_ascii_lowercase(),
            file: fix.file.clone(),
            description: fix.description.clone(),
            before_sha256: before.map(|text| sha256_hex(text.as_bytes())),
            after_sha256: after.map(|text| sha256_hex(text.as_bytes())),
            before_bytes: before.map(str::len),
            after_bytes: after.map(str::len),
            diff: match (before, after) {
                (Some(before), Some(after)) => Some(simple_unified_diff(&fix.file, before, after)),
                _ => None,
            },
            before: full_content.then(|| fix.before.clone()).flatten(),
            after: full_content.then(|| fix.after.clone()).flatten(),
        })
    }
}

fn simple_unified_diff(path: &str, before: &str, after: &str) -> String {
    if before == after {
        return String::new();
    }
    let mut out = format!("--- {path}\n+++ {path}\n");
    let before_lines = before.lines().collect::<Vec<_>>();
    let after_lines = after.lines().collect::<Vec<_>>();
    let max_len = before_lines.len().max(after_lines.len());
    for index in 0..max_len {
        match (before_lines.get(index), after_lines.get(index)) {
            (Some(left), Some(right)) if left == right => out.push_str(&format!(" {left}\n")),
            (Some(left), Some(right)) => {
                out.push_str(&format!("-{left}\n"));
                out.push_str(&format!("+{right}\n"));
            }
            (Some(left), None) => out.push_str(&format!("-{left}\n")),
            (None, Some(right)) => out.push_str(&format!("+{right}\n")),
            (None, None) => {}
        }
    }
    out
}

fn summarize_timings(values: &[f64]) -> Value {
    if values.is_empty() {
        return json!({
            "min_ms": 0.0,
            "median_ms": 0.0,
            "max_ms": 0.0,
            "mean_ms": 0.0,
        });
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let sum = sorted.iter().sum::<f64>();
    let median = if sorted.len().is_multiple_of(2) {
        let upper = sorted.len() / 2;
        (sorted[upper - 1] + sorted[upper]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };
    json!({
        "min_ms": round_ms(*sorted.first().unwrap_or(&0.0)),
        "median_ms": round_ms(median),
        "max_ms": round_ms(*sorted.last().unwrap_or(&0.0)),
        "mean_ms": round_ms(sum / sorted.len() as f64),
    })
}

fn round_ms(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn map_bun_severity(value: BunSeverityArg) -> Severity {
    match value {
        BunSeverityArg::Error => Severity::Error,
        BunSeverityArg::Warn => Severity::Warn,
        BunSeverityArg::Info => Severity::Info,
    }
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warn => "warn",
        Severity::Info => "info",
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
