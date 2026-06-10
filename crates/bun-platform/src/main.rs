use anyhow::{Context, Result};
use bun_platform_core::{
    PlatformPaths, Severity, SkillContext, apply_safe_fixes, check_skill_integrity,
    create_release_sync_report, format_findings_md, format_findings_text, format_fixes_text,
    load_audit_config, plan_safe_fixes, preview_release_sync, run_audit, run_release_sync,
    should_fail,
};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use std::{
    io,
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    time::Instant,
};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Audit and maintain Bun-first repo posture and bun-dev skill references.",
    long_about = "bun-platform is the standalone execution boundary for the bun-dev skill. It audits Bun-first repos, plans and applies bounded safe fixes, validates repo state, benchmarks the hot path, and maintains vendor-backed bun-dev references.",
    propagate_version = true,
    after_long_help = "Examples:\n  bun-platform audit --root .\n  bun-platform plan-fixes --root .\n  bun-platform benchmark --root ./fixtures/github-actions --format json\n  bun-platform release-sync --status\n  bun-platform release-sync --dry-run"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(
        about = "Audit a repository for Bun platform findings.",
        long_about = "Scan a repository for lockfile drift, Bun runtime mismatches, TypeScript posture, Vercel Bun runtime issues, and other bun-dev rules.",
        after_long_help = "Example:\n  bun-platform audit --root . --format text --fail-on warn"
    )]
    Audit {
        #[command(flatten)]
        scope: ScopeArgs,
        #[command(flatten)]
        output: OutputArgs,
    },
    #[command(
        about = "List bun-dev rule identifiers.",
        long_about = "Print the canonical bun-dev rule IDs discovered from the skill directory so operators can inspect or explain them.",
        after_long_help = "Example:\n  bun-platform list-rules"
    )]
    ListRules {
        #[command(flatten)]
        skill: SkillArgs,
    },
    #[command(
        about = "Print the full markdown for one bun-dev rule.",
        long_about = "Load the requested rule markdown from the bun-dev skill and print it verbatim.",
        after_long_help = "Example:\n  bun-platform explain pm-no-mixed-lockfiles"
    )]
    Explain {
        #[arg(help = "Rule identifier to print, such as `pm-no-mixed-lockfiles`.")]
        rule_id: String,
        #[command(flatten)]
        skill: SkillArgs,
    },
    #[command(
        about = "Plan deterministic safe fixes without mutating the repo.",
        long_about = "Evaluate the current repo posture and print the safe package.json fixes bun-platform can apply without introducing compatibility layers.",
        after_long_help = "Example:\n  bun-platform plan-fixes --root . --format json"
    )]
    PlanFixes {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(
            long,
            value_enum,
            default_value_t = OutputMode::Text,
            help = "Output format for the planned fixes."
        )]
        format: OutputMode,
    },
    #[command(
        about = "Apply deterministic safe fixes.",
        long_about = "Apply the bounded safe fixes bun-platform knows how to perform. Rollback artifacts are written to external state, never inside the repo or skill tree.",
        after_long_help = "Example:\n  bun-platform apply-safe-fixes --root ."
    )]
    ApplySafeFixes {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(
            long,
            value_enum,
            default_value_t = OutputMode::Text,
            help = "Output format for the applied fixes."
        )]
        format: OutputMode,
    },
    #[command(
        about = "Audit the repo and then run validation commands.",
        long_about = "Run the audit first, fail on the requested severity threshold, and if clean enough execute repo-native validation commands.",
        after_long_help = "Example:\n  bun-platform validate --root . --fail-on warn"
    )]
    Validate {
        #[command(flatten)]
        scope: ScopeArgs,
        #[command(flatten)]
        output: OutputArgs,
    },
    #[command(
        about = "Measure hot-path performance for audit and fix planning.",
        long_about = "Benchmark the local hot path by timing audit and plan-fixes over repeated iterations. This is intended for fixture or scoped-repo performance tracking, not vendor-backed release-sync work.",
        after_long_help = "Example:\n  bun-platform benchmark --root ./fixtures/github-actions --iterations 5 --format json"
    )]
    Benchmark {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(
            long,
            default_value_t = 3,
            help = "Number of repeated iterations to measure."
        )]
        iterations: u32,
        #[arg(
            long,
            value_enum,
            default_value_t = OutputMode::Text,
            help = "Output format for benchmark results."
        )]
        format: OutputMode,
    },
    #[command(
        about = "Maintain vendor-backed bun-dev references and indexes.",
        long_about = "Refresh Bun release notes and Vercel Bun runtime snapshots for the bun-dev skill, regenerate indexes, and write the resulting report to external state. Use --status for a local report or --dry-run for a non-mutating preview.",
        after_long_help = "Examples:\n  bun-platform release-sync\n  bun-platform release-sync --status --format json\n  bun-platform release-sync --dry-run"
    )]
    ReleaseSync {
        #[command(flatten)]
        skill: SkillArgs,
        #[arg(
            long,
            help = "Print the current local release-sync report and integrity state without fetching remote docs."
        )]
        status: bool,
        #[arg(
            long = "dry-run",
            conflicts_with = "status",
            help = "Fetch remote docs and report which skill files would change without mutating them."
        )]
        dry_run: bool,
        #[arg(
            long,
            value_enum,
            default_value_t = OutputMode::Text,
            help = "Output format for status, preview, or sync reporting."
        )]
        format: OutputMode,
    },
    #[command(
        about = "Inspect bun-platform installation, state paths, and skill integrity.",
        long_about = "Print the discovered skill root, XDG config/state/cache locations, current release report path, Bun version pin, and bun-dev integrity status.",
        after_long_help = "Example:\n  bun-platform doctor --format json"
    )]
    Doctor {
        #[command(flatten)]
        skill: SkillArgs,
        #[arg(
            long,
            value_enum,
            default_value_t = OutputMode::Text,
            help = "Output format for doctor data."
        )]
        format: OutputMode,
    },
    #[command(
        about = "Generate shell completions.",
        long_about = "Print shell completion scripts for the requested shell.",
        after_long_help = "Example:\n  bun-platform completions zsh"
    )]
    Completions {
        #[arg(value_enum, help = "Shell to generate completions for.")]
        shell: Shell,
    },
}

#[derive(Args, Debug, Default)]
struct SkillArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Override the bun-dev skill root. Defaults to $BUN_PLATFORM_SKILL_ROOT or ~/.agents/skills/bun-dev."
    )]
    skill_root: Option<PathBuf>,
}

#[derive(Args, Debug, Default)]
struct ScopeArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Repository root to inspect. Defaults to the current working directory."
    )]
    root: Option<PathBuf>,
    #[arg(
        long,
        value_name = "PATH",
        help = "Path to bun-platform.config.json. If provided, the file must exist and parse successfully."
    )]
    config: Option<PathBuf>,
    #[arg(
        long,
        value_name = "PATH",
        help = "Optional baseline file override. Accepts an array of suppression keys or an object with suppressionKeys."
    )]
    baseline: Option<PathBuf>,
    #[arg(
        long = "include",
        value_name = "PATH",
        help = "Restrict the scan to one or more included paths."
    )]
    include_paths: Vec<PathBuf>,
    #[arg(
        long = "exclude",
        value_name = "DIR",
        help = "Exclude an additional directory name from recursive scanning."
    )]
    exclude_dirs: Vec<String>,
    #[arg(
        long = "adapter",
        value_name = "ID",
        help = "Enable a specific adapter set instead of auto-detection."
    )]
    adapters: Vec<String>,
    #[arg(
        long = "max-files",
        value_name = "N",
        help = "Maximum number of files to scan before failing."
    )]
    max_files: Option<usize>,
    #[arg(
        long = "max-bytes",
        value_name = "N",
        help = "Maximum cumulative bytes to scan before failing."
    )]
    max_bytes: Option<u64>,
    #[arg(
        long = "write-cache",
        help = "Write scan-cache entries under the dev-skills Bun platform cache."
    )]
    write_cache: bool,
}

#[derive(Args, Debug)]
struct OutputArgs {
    #[arg(
        long,
        value_enum,
        default_value_t = OutputMode::Text,
        help = "Output format."
    )]
    format: OutputMode,
    #[arg(
        long = "fail-on",
        value_enum,
        help = "Exit non-zero if any finding meets or exceeds this severity."
    )]
    fail_on: Option<SeverityArg>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputMode {
    Text,
    Json,
    Md,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum SeverityArg {
    Error,
    Warn,
    Info,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let paths = PlatformPaths::discover()?;
    paths.ensure()?;

    match cli.command {
        None => {
            Cli::command().print_help()?;
            println!();
        }
        Some(Commands::Audit { scope, output }) => {
            let root = resolve_root(scope.root.clone())?;
            let config = build_config(&root, &scope)?;
            let findings = run_audit(&root, &config, &paths)?;
            print_findings(&findings, output.format)?;
            if let Some(fail_on) = output.fail_on.map(map_severity)
                && should_fail(&findings, fail_on)
            {
                process::exit(1);
            }
        }
        Some(Commands::ListRules { skill }) => {
            let context = SkillContext::discover(skill.skill_root)?;
            for rule_id in context.list_rule_ids()? {
                println!("{rule_id}");
            }
        }
        Some(Commands::Explain { rule_id, skill }) => {
            let context = SkillContext::discover(skill.skill_root)?;
            println!("{}", context.explain_rule(&rule_id)?);
        }
        Some(Commands::PlanFixes { scope, format }) => {
            let root = resolve_root(scope.root.clone())?;
            let config = build_config(&root, &scope)?;
            let fixes = plan_safe_fixes(&root, &config)?;
            print_fixes(&fixes, format, false)?;
        }
        Some(Commands::ApplySafeFixes { scope, format }) => {
            let root = resolve_root(scope.root.clone())?;
            let config = build_config(&root, &scope)?;
            let fixes = apply_safe_fixes(&root, &config, &paths)?;
            print_fixes(&fixes, format, true)?;
        }
        Some(Commands::Validate { scope, output }) => {
            let root = resolve_root(scope.root.clone())?;
            let config = build_config(&root, &scope)?;
            let findings = run_audit(&root, &config, &paths)?;
            print_findings(&findings, output.format)?;
            let fail_on = output.fail_on.map(map_severity).unwrap_or(Severity::Warn);
            if should_fail(&findings, fail_on) {
                process::exit(1);
            }
            let commands = run_validation_commands(&root, &config.validation_commands)?;
            if !commands.is_empty() {
                println!("Validated {} command(s).", commands.len());
            }
        }
        Some(Commands::Benchmark {
            scope,
            iterations,
            format,
        }) => {
            let root = resolve_root(scope.root.clone())?;
            let config = build_config(&root, &scope)?;
            let report = benchmark_root(&root, &config, &paths, iterations)?;
            print_benchmark(&report, format)?;
        }
        Some(Commands::ReleaseSync {
            skill,
            status,
            dry_run,
            format,
        }) => {
            let context = SkillContext::discover(skill.skill_root)?;
            if status {
                let report = create_release_sync_report(&context)?;
                let integrity = check_skill_integrity(&context).is_ok();
                let payload = serde_json::json!({
                    "skill_root": context.skill_root.display().to_string(),
                    "report": report,
                    "integrity_ok": integrity,
                    "state_report_path": paths.reports_dir().join("release-sync-report.json").display().to_string(),
                    "state_report_exists": paths.reports_dir().join("release-sync-report.json").is_file(),
                });
                print_value(&payload, format)?;
            } else if dry_run {
                let preview = preview_release_sync(&context)?;
                let payload = serde_json::json!({
                    "skill_root": context.skill_root.display().to_string(),
                    "preview": preview,
                });
                print_value(&payload, format)?;
            } else {
                let report = run_release_sync(&context, &paths)?;
                let payload = serde_json::json!({
                    "skill_root": context.skill_root.display().to_string(),
                    "report": report,
                    "state_report_path": paths.reports_dir().join("release-sync-report.json").display().to_string(),
                });
                print_value(&payload, format)?;
            }
        }
        Some(Commands::Doctor { skill, format }) => {
            let context = SkillContext::discover(skill.skill_root)?;
            let report = create_release_sync_report(&context)?;
            let integrity = check_skill_integrity(&context);
            let payload = serde_json::json!({
                "skill_root": context.skill_root.display().to_string(),
                "config_dir": paths.config_dir.display().to_string(),
                "state_dir": paths.state_dir.display().to_string(),
                "cache_dir": paths.cache_dir.display().to_string(),
                "state_report_path": paths.reports_dir().join("release-sync-report.json").display().to_string(),
                "verified_bun_version": report.verified_bun_version,
                "integrity_ok": integrity.is_ok(),
                "integrity_error": integrity.err().map(|error| format!("{error:#}")),
            });
            print_value(&payload, format)?;
        }
        Some(Commands::Completions { shell }) => {
            let mut command = Cli::command();
            generate(shell, &mut command, "bun-platform", &mut io::stdout());
        }
    }

    Ok(())
}

fn build_config(root: &Path, scope: &ScopeArgs) -> Result<bun_platform_core::AuditConfig> {
    let overrides = bun_platform_core::CliOverrides {
        baseline_path: scope.baseline.clone(),
        include_paths: scope.include_paths.clone(),
        exclude_dirs: scope.exclude_dirs.clone(),
        adapters: scope.adapters.clone(),
        max_files: scope.max_files,
        max_bytes: scope.max_bytes,
        write_cache: scope.write_cache,
    };
    load_audit_config(root, scope.config.as_deref(), &overrides)
}

fn resolve_root(root: Option<PathBuf>) -> Result<PathBuf> {
    let path = root.unwrap_or(std::env::current_dir()?);
    path.canonicalize()
        .with_context(|| format!("failed to resolve {}", path.display()))
}

fn print_findings(findings: &[bun_platform_core::Finding], format: OutputMode) -> Result<()> {
    match format {
        OutputMode::Text => print!("{}", format_findings_text(findings)),
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(findings)?),
        OutputMode::Md => print!("{}", format_findings_md(findings)),
    }
    Ok(())
}

fn print_fixes(
    fixes: &[bun_platform_core::PlannedFix],
    format: OutputMode,
    applied: bool,
) -> Result<()> {
    match format {
        OutputMode::Text | OutputMode::Md => print!("{}", format_fixes_text(fixes, applied)),
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(fixes)?),
    }
    Ok(())
}

fn print_value(value: &serde_json::Value, format: OutputMode) -> Result<()> {
    match format {
        OutputMode::Text => println!("{}", serde_json::to_string_pretty(value)?),
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(value)?),
        OutputMode::Md => {
            println!("```json");
            println!("{}", serde_json::to_string_pretty(value)?);
            println!("```");
        }
    }
    Ok(())
}

fn map_severity(value: SeverityArg) -> Severity {
    match value {
        SeverityArg::Error => Severity::Error,
        SeverityArg::Warn => Severity::Warn,
        SeverityArg::Info => Severity::Info,
    }
}

fn run_validation_commands(root: &Path, configured: &[String]) -> Result<Vec<String>> {
    let package_json_path = root.join("package.json");
    let package_json = if package_json_path.is_file() {
        let text = std::fs::read_to_string(&package_json_path)
            .with_context(|| format!("failed to read {}", package_json_path.display()))?;
        Some(
            serde_json::from_str::<serde_json::Value>(&text)
                .with_context(|| format!("failed to parse {}", package_json_path.display()))?,
        )
    } else {
        None
    };
    let scripts = package_json
        .as_ref()
        .and_then(|value| value.get("scripts"))
        .and_then(|value| value.as_object())
        .cloned()
        .unwrap_or_default();
    let commands = if configured.is_empty() {
        let mut defaults = Vec::new();
        if root.join("bun.lockb").is_file() || root.join("bun.lock").is_file() {
            defaults.push("bun install --frozen-lockfile".to_string());
        }
        for script in ["lint", "typecheck", "test", "build"] {
            if scripts.contains_key(script) {
                defaults.push(format!("bun run {script}"));
            }
        }
        defaults
    } else {
        configured.to_vec()
    };

    for command in &commands {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let status = Command::new(&shell)
            .arg("-c")
            .arg(command)
            .current_dir(root)
            .stdin(Stdio::null())
            .status()
            .with_context(|| format!("failed to run validation command `{command}`"))?;
        if !status.success() {
            anyhow::bail!("Validation command failed: {command}");
        }
    }
    Ok(commands)
}

fn benchmark_root(
    root: &Path,
    config: &bun_platform_core::AuditConfig,
    paths: &PlatformPaths,
    iterations: u32,
) -> Result<serde_json::Value> {
    let iterations = iterations.max(1);
    let mut audit_ms = Vec::new();
    let mut plan_fix_ms = Vec::new();

    for _ in 0..iterations {
        let audit_started = Instant::now();
        let findings = run_audit(root, config, paths)?;
        let audit_elapsed = audit_started.elapsed().as_secs_f64() * 1000.0;
        audit_ms.push(audit_elapsed);

        let fix_started = Instant::now();
        let fixes = plan_safe_fixes(root, config)?;
        let fix_elapsed = fix_started.elapsed().as_secs_f64() * 1000.0;
        plan_fix_ms.push(fix_elapsed);

        let _ = (findings.len(), fixes.len());
    }

    Ok(serde_json::json!({
        "root": root.display().to_string(),
        "iterations": iterations,
        "audit_ms": audit_ms,
        "plan_fix_ms": plan_fix_ms,
        "summary": {
            "audit": summarize_timings(&audit_ms),
            "plan_fixes": summarize_timings(&plan_fix_ms),
        }
    }))
}

fn summarize_timings(values: &[f64]) -> serde_json::Value {
    if values.is_empty() {
        return serde_json::json!({
            "min_ms": 0.0,
            "median_ms": 0.0,
            "max_ms": 0.0,
            "mean_ms": 0.0,
        });
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let sum = sorted.iter().copied().sum::<f64>();
    let median = if sorted.len().is_multiple_of(2) {
        let upper = sorted.len() / 2;
        (sorted[upper - 1] + sorted[upper]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };

    serde_json::json!({
        "min_ms": round_ms(*sorted.first().unwrap_or(&0.0)),
        "median_ms": round_ms(median),
        "max_ms": round_ms(*sorted.last().unwrap_or(&0.0)),
        "mean_ms": round_ms(sum / sorted.len() as f64),
    })
}

fn round_ms(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn print_benchmark(report: &serde_json::Value, format: OutputMode) -> Result<()> {
    match format {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputMode::Md => {
            println!("# bun-platform Benchmark");
            println!();
            println!("```json");
            println!("{}", serde_json::to_string_pretty(report)?);
            println!("```");
        }
        OutputMode::Text => {
            println!(
                "Benchmark root: {}",
                report["root"].as_str().unwrap_or_default()
            );
            println!(
                "Iterations: {}",
                report["iterations"].as_u64().unwrap_or_default()
            );
            println!(
                "Audit ms: {}",
                report["audit_ms"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|value| value.as_f64())
                    .map(|value| format!("{value:.2}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!(
                "Plan-fixes ms: {}",
                report["plan_fix_ms"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|value| value.as_f64())
                    .map(|value| format!("{value:.2}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!(
                "Audit summary: min={:.2} median={:.2} max={:.2} mean={:.2}",
                report["summary"]["audit"]["min_ms"]
                    .as_f64()
                    .unwrap_or_default(),
                report["summary"]["audit"]["median_ms"]
                    .as_f64()
                    .unwrap_or_default(),
                report["summary"]["audit"]["max_ms"]
                    .as_f64()
                    .unwrap_or_default(),
                report["summary"]["audit"]["mean_ms"]
                    .as_f64()
                    .unwrap_or_default(),
            );
            println!(
                "Plan-fixes summary: min={:.2} median={:.2} max={:.2} mean={:.2}",
                report["summary"]["plan_fixes"]["min_ms"]
                    .as_f64()
                    .unwrap_or_default(),
                report["summary"]["plan_fixes"]["median_ms"]
                    .as_f64()
                    .unwrap_or_default(),
                report["summary"]["plan_fixes"]["max_ms"]
                    .as_f64()
                    .unwrap_or_default(),
                report["summary"]["plan_fixes"]["mean_ms"]
                    .as_f64()
                    .unwrap_or_default(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn summarize_timings_reports_expected_median() {
        let summary = summarize_timings(&[10.0, 30.0, 20.0]);
        assert_eq!(summary["median_ms"].as_f64().unwrap_or_default(), 20.0);
    }
}
