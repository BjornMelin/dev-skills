use std::env;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Component;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, SecondsFormat, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const CAPSULE_SCHEMA: &str = "codex-dev.task-capsule.v1";
pub const EVIDENCE_SCHEMA: &str = "codex-dev.evidence.v1";
pub const VERIFICATION_SCHEMA: &str = "codex-dev.verification.v1";
pub const SUBAGENTS_SCHEMA: &str = "codex-dev.subagents.v1";
pub const PR_SCHEMA: &str = "codex-dev.pr.v1";
pub const PR_CONTROL_PLAN_SCHEMA: &str = "codex-dev.pr-control-plan.v1";
pub const OUTPUT_SCHEMA: &str = "codex-dev.output.v1";
pub const POLICY_GATES_SCHEMA: &str = "codex-dev.policy-gates.v1";

#[derive(Parser, Debug)]
#[command(name = "codex-dev")]
#[command(about = "Development operating-layer helper for Codex task capsules")]
pub struct Cli {
    #[arg(
        long,
        global = true,
        help = "Emit machine-readable JSON when supported"
    )]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    fn command_name(&self) -> &'static str {
        match &self.command {
            Commands::Capsule { command } => match command {
                CapsuleCommand::Init(_) => "capsule init",
                CapsuleCommand::Validate(_) => "capsule validate",
                CapsuleCommand::Status(_) => "capsule status",
                CapsuleCommand::Render(_) => "capsule render",
            },
            Commands::Policy { command } => match command {
                PolicyCommand::Manifest(_) => "policy manifest",
                PolicyCommand::Run(_) => "policy run",
            },
            Commands::Pr { command } => match command {
                PrCommand::Plan(_) => "pr plan",
                PrCommand::Record(_) => "pr record",
                PrCommand::Status(_) => "pr status",
            },
        }
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage local task capsules.
    Capsule {
        #[command(subcommand)]
        command: CapsuleCommand,
    },
    /// Plan or run repo-native validation policy gates.
    Policy {
        #[command(subcommand)]
        command: PolicyCommand,
    },
    /// Capture hosted PR evidence into task capsules.
    Pr {
        #[command(subcommand)]
        command: PrCommand,
    },
}

#[derive(Subcommand, Debug)]
enum CapsuleCommand {
    /// Create a new local task capsule.
    Init(Box<InitArgs>),
    /// Validate a task capsule directory.
    Validate(PathArgs),
    /// Print task capsule status.
    Status(PathArgs),
    /// Render a Markdown summary from capsule state.
    Render(PathArgs),
}

#[derive(Subcommand, Debug)]
enum PolicyCommand {
    /// Print a machine-readable gate manifest.
    Manifest(PolicyManifestArgs),
    /// Plan or execute gates and record capsule evidence.
    Run(PolicyRunArgs),
}

#[derive(Subcommand, Debug)]
enum PrCommand {
    /// Print the live-command plan for PR evidence capture.
    Plan(PrPlanArgs),
    /// Record a normalized PR snapshot into a task capsule.
    Record(PrRecordArgs),
    /// Print the PR snapshot currently stored in a task capsule.
    Status(PrStatusArgs),
}

#[derive(Args, Debug)]
pub struct PrPlanArgs {
    #[arg(long, value_name = "OWNER/REPO")]
    pub repo: String,
    #[arg(long, value_name = "PR_NUMBER")]
    pub number: u64,
    #[arg(long, value_name = "RFC3339")]
    pub generated_at: Option<DateTime<Utc>>,
}

#[derive(Args, Debug)]
pub struct PrRecordArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
    #[arg(
        long,
        value_name = "SNAPSHOT_JSON",
        help = "Local normalized PR snapshot fixture to record"
    )]
    pub source: PathBuf,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
}

#[derive(Args, Debug)]
pub struct PrStatusArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
}

#[derive(Args, Debug)]
pub struct PolicyManifestArgs {
    #[arg(long, value_enum, default_value_t = PolicyProfile::CodexDev)]
    pub profile: PolicyProfile,
    #[arg(long, value_name = "RFC3339")]
    pub generated_at: Option<DateTime<Utc>>,
}

#[derive(Args, Debug)]
pub struct PolicyRunArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
    #[arg(
        long,
        value_name = "REPO_ROOT",
        help = "Repository root used when executing repo-native gates"
    )]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PolicyProfile::CodexDev)]
    pub profile: PolicyProfile,
    #[arg(long, help = "Execute gates instead of recording a dry-run plan")]
    pub execute: bool,
    #[arg(long, help = "Permit gates marked as network-using")]
    pub allow_network: bool,
    #[arg(long, help = "Continue executing after a failed required gate")]
    pub keep_going: bool,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(long)]
    title: String,
    #[arg(long)]
    objective: Option<String>,
    #[arg(long)]
    branch: Option<String>,
    #[arg(long, default_value = "main")]
    base_branch: String,
    #[arg(long = "issue")]
    issues: Vec<u64>,
    #[arg(long = "pr")]
    pull_requests: Vec<u64>,
    #[arg(long, default_value = ".codex/tasks")]
    root: PathBuf,
    #[arg(long)]
    slug: Option<String>,
    #[arg(long)]
    id: Option<String>,
    #[arg(long, value_enum, default_value_t = CapsuleStatus::Active)]
    status: CapsuleStatus,
    #[arg(long, value_name = "RFC3339")]
    created_at: Option<DateTime<Utc>>,
    #[arg(long)]
    force: bool,
}

#[derive(Args, Debug)]
pub struct PathArgs {
    #[arg(value_name = "CAPSULE_DIR")]
    path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
pub enum CapsuleStatus {
    Active,
    Blocked,
    ReadyForPr,
    InReview,
    Merged,
    Closed,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
pub enum PolicyProfile {
    CodexDev,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capsule {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub status: CapsuleStatus,
    pub objective: String,
    pub branch: String,
    pub base_branch: String,
    pub issues: Vec<u64>,
    pub pull_requests: Vec<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub schema: String,
    pub kind: EvidenceKind,
    pub at: DateTime<Utc>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub artifacts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Command,
    Subagent,
    Review,
    Ci,
    Decision,
    Research,
    Manual,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Verification {
    pub schema: String,
    pub required: Vec<GateRecord>,
    pub optional: Vec<GateRecord>,
    pub last_checked_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GateRecord {
    pub name: String,
    pub command: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subagents {
    pub schema: String,
    pub batches: Vec<SubagentBatch>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentBatch {
    pub id: String,
    pub status: String,
    pub agents: Vec<SubagentRecord>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentRecord {
    pub role: String,
    pub task: String,
    pub status: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrEvidence {
    pub schema: String,
    pub repository: Option<String>,
    pub number: Option<u64>,
    pub url: Option<String>,
    pub state: String,
    pub checks: Vec<CheckRecord>,
    pub review_threads: ReviewThreadSummary,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckRecord {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub url: Option<String>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewThreadSummary {
    pub unresolved: u64,
    pub last_checked_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrControlPlan {
    pub schema: String,
    pub repository: String,
    pub number: u64,
    pub generated_at: DateTime<Utc>,
    pub commands: Vec<PrControlCommand>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrControlCommand {
    pub id: String,
    pub name: String,
    pub command: Vec<String>,
    pub source: String,
    pub required: bool,
    pub network: bool,
    pub secrets: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_input: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrRecordResult {
    pub capsule: PathBuf,
    pub pr_path: PathBuf,
    pub evidence_path: PathBuf,
    pub pr: PrEvidence,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrStatusResult {
    pub capsule: PathBuf,
    pub pr: PrEvidence,
}

#[derive(Debug, Deserialize)]
struct PrSnapshotInput {
    #[allow(dead_code)]
    schema: Option<String>,
    repository: Option<String>,
    number: Option<u64>,
    url: Option<String>,
    state: String,
    #[serde(default)]
    checks: Vec<CheckSnapshotInput>,
    review_threads: ReviewThreadSnapshotInput,
}

#[derive(Debug, Deserialize)]
struct CheckSnapshotInput {
    name: String,
    status: String,
    #[serde(default)]
    conclusion: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    checked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct ReviewThreadSnapshotInput {
    unresolved: u64,
    #[serde(default)]
    last_checked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyManifest {
    pub schema: String,
    pub profile: PolicyProfile,
    pub generated_at: DateTime<Utc>,
    pub gates: Vec<PolicyGate>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyGate {
    pub id: String,
    pub name: String,
    pub command: Vec<String>,
    pub source: String,
    pub required: bool,
    pub network: bool,
    pub secrets: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyRunResult {
    pub capsule: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_root: Option<PathBuf>,
    pub profile: PolicyProfile,
    pub dry_run: bool,
    pub passed: bool,
    pub gates: Vec<PolicyGateResult>,
    pub verification_path: PathBuf,
    pub evidence_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyGateResult {
    pub id: String,
    pub name: String,
    pub command: String,
    pub required: bool,
    pub status: GateStatus,
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    Planned,
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct CommandEnvelope {
    schema: &'static str,
    ok: bool,
    command: &'static str,
    result: Value,
}

#[derive(Debug)]
struct CommandOutput {
    ok: bool,
    command: &'static str,
    human: String,
    result: Value,
}

impl CommandOutput {
    fn error(command: &'static str, message: String) -> Self {
        Self {
            ok: false,
            command,
            human: format!("error: {message}"),
            result: json!({
                "error": {
                    "message": message,
                },
            }),
        }
    }
}

pub fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    let json = cli.json;
    let command = cli.command_name();
    let output = match handle_cli(cli) {
        Ok(output) => output,
        Err(error) if json => CommandOutput::error(command, format!("{error:#}")),
        Err(error) => return Err(error),
    };
    let ok = output.ok;
    let rendered = render_output(output, json)?;
    print!("{rendered}");
    if ok {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

pub fn run_from<I, T>(args: I) -> Result<String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    let json = cli.json;
    let command = cli.command_name();
    let output = match handle_cli(cli) {
        Ok(output) => output,
        Err(error) if json => CommandOutput::error(command, format!("{error:#}")),
        Err(error) => return Err(error),
    };
    render_output(output, json)
}

fn handle_cli(cli: Cli) -> Result<CommandOutput> {
    match cli.command {
        Commands::Capsule { command } => match command {
            CapsuleCommand::Init(args) => {
                let result = init_capsule(*args)?;
                Ok(CommandOutput {
                    ok: true,
                    command: "capsule init",
                    human: format!("created capsule at {}", result.path.display()),
                    result: serde_json::to_value(result)?,
                })
            }
            CapsuleCommand::Validate(args) => {
                let result = validate_capsule(&args.path)?;
                let human = if result.valid {
                    format!("valid capsule at {}", result.path.display())
                } else {
                    format!(
                        "invalid capsule at {}: {} issue(s)",
                        result.path.display(),
                        result.errors.len()
                    )
                };
                Ok(CommandOutput {
                    ok: result.valid,
                    command: "capsule validate",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            CapsuleCommand::Status(args) => {
                let result = capsule_status(&args.path)?;
                Ok(CommandOutput {
                    ok: true,
                    command: "capsule status",
                    human: format!("{} [{}] on {}", result.title, result.status, result.branch),
                    result: serde_json::to_value(result)?,
                })
            }
            CapsuleCommand::Render(args) => {
                let result = render_capsule(&args.path)?;
                Ok(CommandOutput {
                    ok: true,
                    command: "capsule render",
                    human: result.markdown.clone(),
                    result: serde_json::to_value(result)?,
                })
            }
        },
        Commands::Policy { command } => match command {
            PolicyCommand::Manifest(args) => {
                let generated_at = args.generated_at.unwrap_or_else(Utc::now);
                let result = policy_manifest(args.profile, generated_at);
                Ok(CommandOutput {
                    ok: true,
                    command: "policy manifest",
                    human: format!(
                        "generated {} policy gate(s) for {}",
                        result.gates.len(),
                        result.profile
                    ),
                    result: serde_json::to_value(result)?,
                })
            }
            PolicyCommand::Run(args) => {
                let checked_at = args.checked_at.unwrap_or_else(Utc::now);
                let result = run_policy_gates(args, checked_at)?;
                let failed = result
                    .gates
                    .iter()
                    .filter(|gate| gate.required && gate.status == GateStatus::Failed)
                    .count();
                let human = if result.dry_run {
                    format!(
                        "planned {} policy gate(s) for {}",
                        result.gates.len(),
                        result.capsule.display()
                    )
                } else if failed == 0 {
                    format!(
                        "passed {} policy gate(s) for {}",
                        result.gates.len(),
                        result.capsule.display()
                    )
                } else {
                    format!(
                        "failed {} required policy gate(s) for {}",
                        failed,
                        result.capsule.display()
                    )
                };
                Ok(CommandOutput {
                    ok: result.passed,
                    command: "policy run",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
        },
        Commands::Pr { command } => match command {
            PrCommand::Plan(args) => {
                let generated_at = args.generated_at.unwrap_or_else(Utc::now);
                let result = pr_control_plan(args.repo, args.number, generated_at);
                Ok(CommandOutput {
                    ok: true,
                    command: "pr plan",
                    human: format!(
                        "planned {} PR evidence command(s) for {}#{}",
                        result.commands.len(),
                        result.repository,
                        result.number
                    ),
                    result: serde_json::to_value(result)?,
                })
            }
            PrCommand::Record(args) => {
                let checked_at = args.checked_at.unwrap_or_else(Utc::now);
                let result = record_pr_snapshot(args, checked_at)?;
                let unresolved = result.pr.review_threads.unresolved;
                let human = format!(
                    "recorded PR snapshot for {} with {} unresolved thread(s)",
                    render_pr_label(&result.pr),
                    unresolved
                );
                Ok(CommandOutput {
                    ok: true,
                    command: "pr record",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            PrCommand::Status(args) => {
                let result = pr_status(&args.capsule)?;
                Ok(CommandOutput {
                    ok: true,
                    command: "pr status",
                    human: render_pr_status(&result.pr),
                    result: serde_json::to_value(result)?,
                })
            }
        },
    }
}

fn render_output(output: CommandOutput, json_output: bool) -> Result<String> {
    if json_output {
        let envelope = CommandEnvelope {
            schema: OUTPUT_SCHEMA,
            ok: output.ok,
            command: output.command,
            result: output.result,
        };
        Ok(format!("{}\n", serde_json::to_string_pretty(&envelope)?))
    } else {
        Ok(format!("{}\n", output.human))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitResult {
    pub path: PathBuf,
    pub capsule: Capsule,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationResult {
    pub path: PathBuf,
    pub valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusResult {
    pub path: PathBuf,
    pub id: String,
    pub title: String,
    pub status: CapsuleStatus,
    pub objective: String,
    pub branch: String,
    pub base_branch: String,
    pub issues: Vec<u64>,
    pub pull_requests: Vec<u64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenderResult {
    pub path: PathBuf,
    pub markdown: String,
}

pub fn policy_manifest(profile: PolicyProfile, generated_at: DateTime<Utc>) -> PolicyManifest {
    PolicyManifest {
        schema: POLICY_GATES_SCHEMA.to_string(),
        profile,
        generated_at,
        gates: built_in_gates(profile),
    }
}

pub fn run_policy_gates(args: PolicyRunArgs, checked_at: DateTime<Utc>) -> Result<PolicyRunResult> {
    let validation = validate_capsule(&args.capsule)?;
    if !validation.valid {
        bail!(
            "invalid capsule at {}: {}",
            args.capsule.display(),
            validation.errors.join("; ")
        );
    }

    let manifest = policy_manifest(args.profile, checked_at);
    let dry_run = !args.execute;
    let repo_root = if dry_run {
        args.repo_root
            .as_deref()
            .map(canonicalize_repo_root)
            .transpose()?
    } else {
        Some(resolve_repo_root(&args.capsule, args.repo_root.as_deref())?)
    };
    let mut results = Vec::new();

    for (index, gate) in manifest.gates.iter().enumerate() {
        let result = if dry_run {
            plan_gate(gate)
        } else if gate.network && !args.allow_network {
            skip_gate(
                gate,
                "gate requires network and --allow-network was not set",
            )
        } else {
            execute_gate(gate, repo_root.as_deref())
        };
        let should_stop = result.required
            && result.status == GateStatus::Failed
            && args.execute
            && !args.keep_going;
        results.push(result);
        if should_stop {
            for remaining in &manifest.gates[index + 1..] {
                results.push(skip_gate(remaining, "previous required gate failed"));
            }
            break;
        }
    }

    let passed = results.iter().all(|gate| {
        !gate.required || matches!(gate.status, GateStatus::Planned | GateStatus::Passed)
    });
    record_policy_run(&args.capsule, &results, checked_at)?;

    Ok(PolicyRunResult {
        verification_path: args.capsule.join("verification.json"),
        evidence_path: args.capsule.join("evidence.jsonl"),
        capsule: args.capsule,
        repo_root,
        profile: args.profile,
        dry_run,
        passed,
        gates: results,
    })
}

pub fn pr_control_plan(
    repository: String,
    number: u64,
    generated_at: DateTime<Utc>,
) -> PrControlPlan {
    PrControlPlan {
        schema: PR_CONTROL_PLAN_SCHEMA.to_string(),
        repository: repository.clone(),
        number,
        generated_at,
        commands: vec![
            pr_control_command(
                "gh-pr-view",
                "GitHub PR metadata snapshot",
                [
                    "gh",
                    "pr",
                    "view",
                    &number.to_string(),
                    "--repo",
                    &repository,
                    "--json",
                    "number,url,state,statusCheckRollup,reviewDecision,headRefOid",
                ],
            ),
            pr_control_command(
                "gh-pr-checks",
                "GitHub PR check summary",
                [
                    "gh",
                    "pr",
                    "checks",
                    &number.to_string(),
                    "--repo",
                    &repository,
                ],
            ),
            pr_control_command(
                "review-pack-start",
                "Fresh hosted review-thread bundle",
                [
                    "review-pack",
                    "start",
                    "--repo",
                    &repository,
                    "--pr",
                    &number.to_string(),
                    "--fresh",
                ],
            ),
            pr_control_command_with_manual_input(
                "review-pack-remaining",
                "Unresolved review-thread count from bundle",
                [
                    "review-pack",
                    "remaining",
                    "--repo",
                    &repository,
                    "--pr",
                    &number.to_string(),
                    "--previous",
                    "<bundle.json>",
                ],
                "replace <bundle.json> with the bundle path produced by review-pack start",
            ),
            pr_control_command(
                "gh-pr-review-fix",
                "Verify-first hosted review remediation workflow",
                ["gh-pr-review-fix", "pr", &number.to_string()],
            ),
        ],
    }
}

pub fn record_pr_snapshot(args: PrRecordArgs, checked_at: DateTime<Utc>) -> Result<PrRecordResult> {
    validate_capsule_for_pr_record(&args.capsule)?;

    let snapshot: PrSnapshotInput = read_json(&args.source)?;
    let pr = snapshot.into_pr_evidence(checked_at);
    write_json(args.capsule.join("pr.json"), &pr)?;

    let mut capsule: Capsule = read_json(&args.capsule.join("capsule.json"))?;
    if let Some(number) = pr.number
        && !capsule.pull_requests.contains(&number)
    {
        capsule.pull_requests.push(number);
    }
    capsule.updated_at = checked_at;
    write_json(args.capsule.join("capsule.json"), &capsule)?;

    append_jsonl(
        args.capsule.join("evidence.jsonl"),
        &EvidenceRecord {
            schema: EVIDENCE_SCHEMA.to_string(),
            kind: EvidenceKind::Review,
            at: checked_at,
            summary: format!(
                "PR snapshot recorded for {}; {} unresolved review thread(s); {} check(s)",
                render_pr_label(&pr),
                pr.review_threads.unresolved,
                pr.checks.len()
            ),
            command: Some(render_pr_record_command(
                &args.capsule,
                &args.source,
                checked_at,
            )),
            exit_code: Some(0),
            artifacts: vec!["pr.json".to_string()],
        },
    )?;

    Ok(PrRecordResult {
        pr_path: args.capsule.join("pr.json"),
        evidence_path: args.capsule.join("evidence.jsonl"),
        capsule: args.capsule,
        pr,
    })
}

pub fn pr_status(capsule_path: &Path) -> Result<PrStatusResult> {
    let validation = validate_capsule(capsule_path)?;
    if !validation.valid {
        bail!(
            "invalid capsule at {}: {}",
            capsule_path.display(),
            validation.errors.join("; ")
        );
    }
    let pr: PrEvidence = read_json(&capsule_path.join("pr.json"))?;
    Ok(PrStatusResult {
        capsule: capsule_path.to_path_buf(),
        pr,
    })
}

fn validate_capsule_for_pr_record(capsule_path: &Path) -> Result<()> {
    let validation = validate_capsule(capsule_path)?;
    if !validation.valid {
        bail!(
            "invalid capsule at {}: {}",
            capsule_path.display(),
            validation.errors.join("; ")
        );
    }
    Ok(())
}

pub fn init_capsule(args: InitArgs) -> Result<InitResult> {
    let created_at = args.created_at.unwrap_or_else(Utc::now);
    let slug = args.slug.unwrap_or_else(|| slugify(&args.title));
    let id = args
        .id
        .unwrap_or_else(|| format!("{}-{}", created_at.format("%Y%m%d-%H%M%S"), slug));
    validate_capsule_id(&id)?;
    let branch = match args.branch {
        Some(branch) => branch,
        None => current_git_branch().unwrap_or_else(|| "unknown".to_string()),
    };
    let objective = args.objective.unwrap_or_else(|| args.title.clone());
    let path = args.root.join(&id);

    if path.exists() {
        if !args.force {
            bail!("capsule already exists: {}", path.display());
        }
        let metadata = path
            .symlink_metadata()
            .with_context(|| format!("failed to inspect {}", path.display()))?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            bail!(
                "refusing to replace symlinked capsule path: {}",
                path.display()
            );
        }
        if file_type.is_dir() {
            fs::remove_dir_all(&path).with_context(|| {
                format!("failed to replace capsule directory {}", path.display())
            })?;
        } else {
            fs::remove_file(&path)
                .with_context(|| format!("failed to replace capsule file {}", path.display()))?;
        }
    }
    fs::create_dir_all(&path)
        .with_context(|| format!("failed to create capsule directory {}", path.display()))?;

    let capsule = Capsule {
        schema: CAPSULE_SCHEMA.to_string(),
        id,
        title: args.title,
        status: args.status,
        objective,
        branch,
        base_branch: args.base_branch,
        issues: args.issues,
        pull_requests: args.pull_requests,
        created_at,
        updated_at: created_at,
    };

    write_json(path.join("capsule.json"), &capsule)?;

    let evidence = EvidenceRecord {
        schema: EVIDENCE_SCHEMA.to_string(),
        kind: EvidenceKind::Manual,
        at: created_at,
        summary: "Task capsule initialized".to_string(),
        command: None,
        exit_code: None,
        artifacts: Vec::new(),
    };
    append_jsonl(path.join("evidence.jsonl"), &evidence)?;

    write_json(
        path.join("verification.json"),
        &Verification {
            schema: VERIFICATION_SCHEMA.to_string(),
            required: Vec::new(),
            optional: Vec::new(),
            last_checked_at: created_at,
        },
    )?;
    write_json(
        path.join("subagents.json"),
        &Subagents {
            schema: SUBAGENTS_SCHEMA.to_string(),
            batches: Vec::new(),
        },
    )?;
    write_json(
        path.join("pr.json"),
        &PrEvidence {
            schema: PR_SCHEMA.to_string(),
            repository: None,
            number: None,
            url: None,
            state: "not_created".to_string(),
            checks: Vec::new(),
            review_threads: ReviewThreadSummary {
                unresolved: 0,
                last_checked_at: created_at,
            },
        },
    )?;
    write_json(
        path.join("policy.json"),
        &policy_manifest(PolicyProfile::CodexDev, created_at),
    )?;

    write_markdown(
        path.join("plan.md"),
        &format!("# Plan\n\n{}\n", capsule.objective),
    )?;
    write_markdown(path.join("decisions.md"), "# Decisions\n\n")?;
    write_markdown(path.join("output.md"), "# Output\n\n")?;
    write_markdown(path.join("retrospective.md"), "# Retrospective\n\n")?;

    let validation = validate_capsule(&path)?;
    if !validation.valid {
        bail!(
            "created capsule failed validation: {}",
            validation.errors.join("; ")
        );
    }

    Ok(InitResult {
        path,
        capsule,
        files: REQUIRED_FILES
            .iter()
            .map(|file| (*file).to_string())
            .collect(),
    })
}

const REQUIRED_FILES: &[&str] = &[
    "capsule.json",
    "plan.md",
    "decisions.md",
    "evidence.jsonl",
    "verification.json",
    "subagents.json",
    "pr.json",
    "policy.json",
    "output.md",
    "retrospective.md",
];

pub fn validate_capsule(path: &Path) -> Result<ValidationResult> {
    validate_capsule_files(path)
}

fn validate_capsule_files(path: &Path) -> Result<ValidationResult> {
    let mut errors = Vec::new();
    if !path.is_dir() {
        errors.push(format!(
            "capsule path is not a directory: {}",
            path.display()
        ));
        return Ok(ValidationResult {
            path: path.to_path_buf(),
            valid: false,
            errors,
        });
    }

    let missing_files = REQUIRED_FILES
        .iter()
        .copied()
        .filter(|file| !path.join(file).is_file())
        .collect::<Vec<_>>();
    for file in &missing_files {
        errors.push(format!("missing required file: {file}"));
    }

    if !missing_files.contains(&"capsule.json") {
        match read_json::<Capsule>(&path.join("capsule.json")) {
            Ok(capsule) => {
                if capsule.schema != CAPSULE_SCHEMA {
                    errors.push(format!("capsule.json schema must be {CAPSULE_SCHEMA}"));
                }
            }
            Err(error) => errors.push(format!("invalid capsule.json: {error:#}")),
        }
    }

    if !missing_files.contains(&"evidence.jsonl") {
        match validate_evidence(&path.join("evidence.jsonl")) {
            Ok(file_errors) => errors.extend(file_errors),
            Err(error) => errors.push(format!("invalid evidence.jsonl: {error:#}")),
        }
    }

    if !missing_files.contains(&"verification.json") {
        validate_schema_file::<Verification, _>(
            &path.join("verification.json"),
            VERIFICATION_SCHEMA,
            |value| &value.schema,
            &mut errors,
        );
    }
    if !missing_files.contains(&"subagents.json") {
        validate_schema_file::<Subagents, _>(
            &path.join("subagents.json"),
            SUBAGENTS_SCHEMA,
            |value| &value.schema,
            &mut errors,
        );
    }
    if !missing_files.contains(&"pr.json") {
        validate_schema_file::<PrEvidence, _>(
            &path.join("pr.json"),
            PR_SCHEMA,
            |value| &value.schema,
            &mut errors,
        );
    }
    if !missing_files.contains(&"policy.json") {
        validate_schema_file::<PolicyManifest, _>(
            &path.join("policy.json"),
            POLICY_GATES_SCHEMA,
            |value| &value.schema,
            &mut errors,
        );
    }

    Ok(ValidationResult {
        path: path.to_path_buf(),
        valid: errors.is_empty(),
        errors,
    })
}

pub fn capsule_status(path: &Path) -> Result<StatusResult> {
    let validation = validate_capsule(path)?;
    if !validation.valid {
        bail!(
            "invalid capsule at {}: {}",
            path.display(),
            validation.errors.join("; ")
        );
    }
    let capsule: Capsule = read_json(&path.join("capsule.json"))?;
    Ok(StatusResult {
        path: path.to_path_buf(),
        id: capsule.id,
        title: capsule.title,
        status: capsule.status,
        objective: capsule.objective,
        branch: capsule.branch,
        base_branch: capsule.base_branch,
        issues: capsule.issues,
        pull_requests: capsule.pull_requests,
        updated_at: capsule.updated_at,
    })
}

pub fn render_capsule(path: &Path) -> Result<RenderResult> {
    let status = capsule_status(path)?;
    let mut markdown = String::new();
    writeln!(markdown, "# {}", status.title)?;
    writeln!(markdown)?;
    writeln!(markdown, "- Status: `{}`", status.status)?;
    writeln!(markdown, "- Capsule: `{}`", status.id)?;
    writeln!(markdown, "- Branch: `{}`", status.branch)?;
    writeln!(markdown, "- Base branch: `{}`", status.base_branch)?;
    writeln!(markdown, "- Issues: {}", render_numbers(&status.issues))?;
    writeln!(
        markdown,
        "- Pull requests: {}",
        render_numbers(&status.pull_requests)
    )?;
    writeln!(markdown)?;
    writeln!(markdown, "## Objective")?;
    writeln!(markdown)?;
    writeln!(markdown, "{}", status.objective)?;

    Ok(RenderResult {
        path: path.to_path_buf(),
        markdown,
    })
}

fn resolve_repo_root(capsule_path: &Path, explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(root) = explicit {
        return canonicalize_repo_root(root);
    }

    let current_dir = env::current_dir().context("failed to read current directory")?;
    if let Some(root) = find_repo_root(&current_dir) {
        return Ok(root);
    }

    let capsule_path =
        fs::canonicalize(capsule_path).unwrap_or_else(|_| capsule_path.to_path_buf());
    if let Some(parent) = capsule_path.parent()
        && let Some(root) = find_repo_root(parent)
    {
        return Ok(root);
    }

    bail!(
        "failed to discover repository root from current directory or capsule path; run from the repo or pass --repo-root"
    );
}

fn canonicalize_repo_root(root: &Path) -> Result<PathBuf> {
    let root = fs::canonicalize(root)
        .with_context(|| format!("failed to canonicalize repo root {}", root.display()))?;
    if !root.join("Cargo.toml").is_file() {
        bail!("repo root must contain Cargo.toml: {}", root.display());
    }
    if !root.join("docs/runbooks/validation.md").is_file() {
        bail!(
            "repo root must contain docs/runbooks/validation.md: {}",
            root.display()
        );
    }
    Ok(root)
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|path| {
            path.join("Cargo.toml").is_file() && path.join("docs/runbooks/validation.md").is_file()
        })
        .and_then(|path| fs::canonicalize(path).ok())
}

fn built_in_gates(profile: PolicyProfile) -> Vec<PolicyGate> {
    match profile {
        PolicyProfile::CodexDev => vec![
            policy_gate(
                "cargo-fmt",
                "Rust workspace formatting",
                ["cargo", "fmt", "--all", "--check"],
            ),
            policy_gate(
                "codex-dev-clippy",
                "codex-dev Clippy",
                [
                    "cargo",
                    "clippy",
                    "-p",
                    "codex-dev",
                    "--all-targets",
                    "--",
                    "-D",
                    "warnings",
                ],
            ),
            policy_gate(
                "codex-dev-check",
                "codex-dev cargo check",
                ["cargo", "check", "-p", "codex-dev"],
            ),
            policy_gate(
                "codex-dev-test",
                "codex-dev tests",
                ["cargo", "test", "-p", "codex-dev"],
            ),
            policy_gate(
                "codex-dev-help",
                "codex-dev help smoke",
                ["cargo", "run", "-q", "-p", "codex-dev", "--", "--help"],
            ),
            policy_gate(
                "docs-links",
                "documentation link check",
                [
                    "python3",
                    "tools/docs/check_links.py",
                    "docs",
                    "README.md",
                    "AGENTS.md",
                ],
            ),
            policy_gate(
                "diff-check",
                "git whitespace check",
                ["git", "diff", "--check"],
            ),
        ],
    }
}

fn policy_gate<const N: usize>(id: &str, name: &str, command: [&str; N]) -> PolicyGate {
    PolicyGate {
        id: id.to_string(),
        name: name.to_string(),
        command: command.iter().map(|part| (*part).to_string()).collect(),
        source: "docs/runbooks/validation.md#codex-dev-operating-layer".to_string(),
        required: true,
        network: false,
        secrets: false,
    }
}

fn pr_control_command<const N: usize>(
    id: &str,
    name: &str,
    command: [&str; N],
) -> PrControlCommand {
    PrControlCommand {
        id: id.to_string(),
        name: name.to_string(),
        command: command.iter().map(|part| (*part).to_string()).collect(),
        source: "gh-pr-review-fix / review-pack / gh".to_string(),
        required: true,
        network: true,
        secrets: true,
        manual_input: None,
    }
}

fn pr_control_command_with_manual_input<const N: usize>(
    id: &str,
    name: &str,
    command: [&str; N],
    manual_input: &str,
) -> PrControlCommand {
    PrControlCommand {
        required: false,
        manual_input: Some(manual_input.to_string()),
        ..pr_control_command(id, name, command)
    }
}

impl PrSnapshotInput {
    fn into_pr_evidence(self, checked_at: DateTime<Utc>) -> PrEvidence {
        PrEvidence {
            schema: PR_SCHEMA.to_string(),
            repository: self.repository,
            number: self.number,
            url: self.url,
            state: self.state.to_ascii_lowercase(),
            checks: self
                .checks
                .into_iter()
                .map(|check| CheckRecord {
                    name: check.name,
                    status: check.status.to_ascii_lowercase(),
                    conclusion: check.conclusion.map(|value| value.to_ascii_lowercase()),
                    url: check.url,
                    checked_at: check.checked_at.unwrap_or(checked_at),
                })
                .collect(),
            review_threads: ReviewThreadSummary {
                unresolved: self.review_threads.unresolved,
                last_checked_at: self.review_threads.last_checked_at.unwrap_or(checked_at),
            },
        }
    }
}

fn render_pr_status(pr: &PrEvidence) -> String {
    format!(
        "{} {}: {} unresolved review thread(s), {} check(s)",
        render_pr_label(pr),
        pr.state,
        pr.review_threads.unresolved,
        pr.checks.len()
    )
}

fn render_pr_label(pr: &PrEvidence) -> String {
    match (&pr.repository, pr.number) {
        (Some(repository), Some(number)) => format!("{repository}#{number}"),
        (None, Some(number)) => format!("#{number}"),
        (Some(repository), None) => repository.clone(),
        (None, None) => "unlinked PR".to_string(),
    }
}

fn render_pr_record_command(capsule: &Path, source: &Path, checked_at: DateTime<Utc>) -> String {
    let checked_at = checked_at.to_rfc3339_opts(SecondsFormat::AutoSi, true);
    format!(
        "codex-dev pr record --capsule {} --source {} --checked-at {}",
        shell_quote(&capsule.display().to_string()),
        shell_quote(&source.display().to_string()),
        shell_quote(&checked_at)
    )
}

fn plan_gate(gate: &PolicyGate) -> PolicyGateResult {
    gate_result(gate, GateStatus::Planned, None, None, None, None)
}

fn skip_gate(gate: &PolicyGate, reason: &str) -> PolicyGateResult {
    gate_result(
        gate,
        GateStatus::Skipped,
        None,
        Some(reason.to_string()),
        None,
        None,
    )
}

fn execute_gate(gate: &PolicyGate, repo_root: Option<&Path>) -> PolicyGateResult {
    let Some((program, args)) = gate.command.split_first() else {
        return gate_result(
            gate,
            GateStatus::Failed,
            None,
            Some("gate command is empty".to_string()),
            None,
            None,
        );
    };

    let mut command = Command::new(program);
    command.args(args);
    if let Some(repo_root) = repo_root {
        command.current_dir(repo_root);
    }

    match command.output() {
        Ok(output) => {
            let code = output.status.code();
            if output.status.success() {
                gate_result(gate, GateStatus::Passed, code, None, None, None)
            } else {
                gate_result(
                    gate,
                    GateStatus::Failed,
                    code,
                    Some(match code {
                        Some(code) => format!("command exited with status {code}"),
                        None => "command terminated by signal".to_string(),
                    }),
                    output_excerpt(&output.stdout),
                    output_excerpt(&output.stderr),
                )
            }
        }
        Err(error) => gate_result(
            gate,
            GateStatus::Failed,
            None,
            Some(format!("failed to start command: {error}")),
            None,
            None,
        ),
    }
}

fn gate_result(
    gate: &PolicyGate,
    status: GateStatus,
    exit_code: Option<i32>,
    error: Option<String>,
    stdout: Option<String>,
    stderr: Option<String>,
) -> PolicyGateResult {
    PolicyGateResult {
        id: gate.id.clone(),
        name: gate.name.clone(),
        command: render_command(&gate.command),
        required: gate.required,
        status,
        exit_code,
        error,
        stdout,
        stderr,
    }
}

fn record_policy_run(
    capsule_path: &Path,
    results: &[PolicyGateResult],
    checked_at: DateTime<Utc>,
) -> Result<()> {
    let mut verification: Verification = read_json(&capsule_path.join("verification.json"))?;
    verification.required = results
        .iter()
        .filter(|gate| gate.required)
        .map(|gate| GateRecord {
            name: gate.id.clone(),
            command: gate.command.clone(),
            status: gate.status.to_string(),
        })
        .collect();
    verification.optional = results
        .iter()
        .filter(|gate| !gate.required)
        .map(|gate| GateRecord {
            name: gate.id.clone(),
            command: gate.command.clone(),
            status: gate.status.to_string(),
        })
        .collect();
    verification.last_checked_at = checked_at;
    write_json(capsule_path.join("verification.json"), &verification)?;

    for gate in results {
        append_jsonl(
            capsule_path.join("evidence.jsonl"),
            &EvidenceRecord {
                schema: EVIDENCE_SCHEMA.to_string(),
                kind: match gate.status {
                    GateStatus::Planned | GateStatus::Skipped => EvidenceKind::Decision,
                    GateStatus::Passed | GateStatus::Failed => EvidenceKind::Command,
                },
                at: checked_at,
                summary: format!("Policy gate {} {}", gate.id, gate.status),
                command: Some(gate.command.clone()),
                exit_code: gate.exit_code,
                artifacts: vec!["verification.json".to_string()],
            },
        )?;
    }

    let mut capsule: Capsule = read_json(&capsule_path.join("capsule.json"))?;
    capsule.updated_at = checked_at;
    write_json(capsule_path.join("capsule.json"), &capsule)?;
    Ok(())
}

fn validate_schema_file<T, F>(
    path: &Path,
    expected_schema: &str,
    schema: F,
    errors: &mut Vec<String>,
) where
    T: for<'de> Deserialize<'de>,
    F: Fn(&T) -> &str,
{
    match read_json::<T>(path) {
        Ok(value) => {
            if schema(&value) != expected_schema {
                errors.push(format!(
                    "{} schema must be {expected_schema}",
                    path.file_name()
                        .and_then(|file| file.to_str())
                        .unwrap_or("json file")
                ));
            }
        }
        Err(error) => errors.push(format!(
            "invalid {}: {error:#}",
            path.file_name()
                .and_then(|file| file.to_str())
                .unwrap_or("json file")
        )),
    }
}

fn validate_evidence(path: &Path) -> Result<Vec<String>> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut errors = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: EvidenceRecord = serde_json::from_str(&line)
            .with_context(|| format!("line {} is not valid evidence JSON", index + 1))?;
        if record.schema != EVIDENCE_SCHEMA {
            errors.push(format!(
                "evidence.jsonl line {} schema must be {EVIDENCE_SCHEMA}",
                index + 1
            ));
        }
    }
    Ok(errors)
}

fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<()> {
    let mut file =
        File::create(&path).with_context(|| format!("failed to create {}", path.display()))?;
    serde_json::to_writer_pretty(&mut file, value)?;
    writeln!(file)?;
    Ok(())
}

fn append_jsonl<T: Serialize>(path: PathBuf, value: &T) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    serde_json::to_writer(&mut file, value)?;
    writeln!(file)?;
    Ok(())
}

fn write_markdown(path: PathBuf, content: &str) -> Result<()> {
    fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))
}

fn read_json<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    serde_json::from_reader(file).with_context(|| format!("failed to parse {}", path.display()))
}

fn current_git_branch() -> Option<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let branch = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if branch.is_empty() {
        None
    } else {
        Some(branch)
    }
}

fn validate_capsule_id(id: &str) -> Result<()> {
    if id.is_empty() {
        bail!("capsule id must not be empty");
    }
    if !id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        bail!("capsule id must contain only ASCII letters, numbers, '-' or '_': {id}");
    }

    let mut components = Path::new(id).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(component)), None) if component.to_str() == Some(id) => Ok(()),
        _ => bail!("capsule id must be a single safe path segment: {id}"),
    }
}

fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "task".to_string()
    } else {
        slug.to_string()
    }
}

fn render_numbers(numbers: &[u64]) -> String {
    if numbers.is_empty() {
        "none".to_string()
    } else {
        numbers
            .iter()
            .map(|number| format!("#{number}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn render_command(command: &[String]) -> String {
    command
        .iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(arg: &str) -> String {
    if arg.is_empty() {
        return "''".to_string();
    }
    if arg.bytes().all(|byte| {
        byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'_' | b'@' | b'%' | b'+' | b'=' | b':' | b',' | b'.' | b'/' | b'-'
            )
    }) {
        return arg.to_string();
    }

    format!("'{}'", arg.replace('\'', "'\\''"))
}

fn output_excerpt(bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes).trim().to_string();
    if text.is_empty() {
        return None;
    }

    const MAX_CHARS: usize = 2000;
    if text.chars().count() <= MAX_CHARS {
        return Some(text);
    }

    let mut truncated = text.chars().take(MAX_CHARS).collect::<String>();
    truncated.push_str("\n[truncated]");
    Some(truncated)
}

impl std::fmt::Display for CapsuleStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            CapsuleStatus::Active => "active",
            CapsuleStatus::Blocked => "blocked",
            CapsuleStatus::ReadyForPr => "ready_for_pr",
            CapsuleStatus::InReview => "in_review",
            CapsuleStatus::Merged => "merged",
            CapsuleStatus::Closed => "closed",
        };
        formatter.write_str(value)
    }
}

impl std::fmt::Display for PolicyProfile {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyProfile::CodexDev => formatter.write_str("codex_dev"),
        }
    }
}

impl std::fmt::Display for GateStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            GateStatus::Planned => "planned",
            GateStatus::Passed => "passed",
            GateStatus::Failed => "failed",
            GateStatus::Skipped => "skipped",
        };
        formatter.write_str(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn assert_json_keys(value: &Value, expected: &[&str]) {
        let mut actual = value
            .as_object()
            .expect("json object")
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        actual.sort();
        let mut expected = expected
            .iter()
            .map(|key| (*key).to_string())
            .collect::<Vec<_>>();
        expected.sort();
        assert_eq!(actual, expected);
    }

    fn init_args(root: PathBuf) -> InitArgs {
        InitArgs {
            title: "Build capsule CLI".to_string(),
            objective: Some("Create task capsules".to_string()),
            branch: Some("feat/codex-dev-task-capsules".to_string()),
            base_branch: "main".to_string(),
            issues: vec![22],
            pull_requests: Vec::new(),
            root,
            slug: Some("capsule-cli".to_string()),
            id: Some("20260509-040000-capsule-cli".to_string()),
            status: CapsuleStatus::Active,
            created_at: Some("2026-05-09T04:00:00Z".parse().expect("valid timestamp")),
            force: false,
        }
    }

    #[test]
    fn init_creates_valid_capsule_layout() {
        let temp = tempdir().expect("tempdir");
        let result = init_capsule(init_args(temp.path().to_path_buf())).expect("init capsule");

        assert_eq!(result.capsule.schema, CAPSULE_SCHEMA);
        for file in REQUIRED_FILES {
            assert!(result.path.join(file).exists(), "{file} exists");
        }

        let validation = validate_capsule(&result.path).expect("validate");
        assert!(validation.valid, "{:?}", validation.errors);
    }

    #[test]
    fn init_writes_golden_capsule_contract_files() {
        let temp = tempdir().expect("tempdir");
        let result = init_capsule(init_args(temp.path().to_path_buf())).expect("init capsule");

        let capsule: Value = read_json(&result.path.join("capsule.json")).expect("capsule json");
        assert_json_keys(
            &capsule,
            &[
                "schema",
                "id",
                "title",
                "status",
                "objective",
                "branch",
                "base_branch",
                "issues",
                "pull_requests",
                "created_at",
                "updated_at",
            ],
        );
        assert_eq!(capsule["schema"], CAPSULE_SCHEMA);
        assert_eq!(capsule["status"], "active");
        assert_eq!(capsule["created_at"], "2026-05-09T04:00:00Z");

        let evidence = fs::read_to_string(result.path.join("evidence.jsonl")).expect("evidence");
        let evidence: Value =
            serde_json::from_str(evidence.lines().next().expect("evidence line")).unwrap();
        assert_json_keys(&evidence, &["schema", "kind", "at", "summary", "artifacts"]);
        assert_eq!(evidence["schema"], EVIDENCE_SCHEMA);
        assert_eq!(evidence["kind"], "manual");
        assert_eq!(evidence["artifacts"], json!([]));

        let verification: Value =
            read_json(&result.path.join("verification.json")).expect("verification json");
        assert_json_keys(
            &verification,
            &["schema", "required", "optional", "last_checked_at"],
        );
        assert_eq!(verification["schema"], VERIFICATION_SCHEMA);
        assert_eq!(verification["last_checked_at"], "2026-05-09T04:00:00Z");

        let subagents: Value = read_json(&result.path.join("subagents.json")).expect("subagents");
        assert_json_keys(&subagents, &["schema", "batches"]);
        assert_eq!(subagents["schema"], SUBAGENTS_SCHEMA);

        let pr: Value = read_json(&result.path.join("pr.json")).expect("pr json");
        assert_json_keys(
            &pr,
            &[
                "schema",
                "repository",
                "number",
                "url",
                "state",
                "checks",
                "review_threads",
            ],
        );
        assert_eq!(pr["schema"], PR_SCHEMA);
        assert_eq!(pr["state"], "not_created");
        assert_eq!(
            pr["review_threads"]["last_checked_at"],
            "2026-05-09T04:00:00Z"
        );

        let policy: Value = read_json(&result.path.join("policy.json")).expect("policy json");
        assert_json_keys(&policy, &["schema", "profile", "generated_at", "gates"]);
        assert_eq!(policy["schema"], POLICY_GATES_SCHEMA);
        assert_eq!(policy["profile"], "codex_dev");
        assert_eq!(policy["generated_at"], "2026-05-09T04:00:00Z");
        assert_json_keys(
            &policy["gates"][0],
            &[
                "id", "name", "command", "source", "required", "network", "secrets",
            ],
        );

        let output = fs::read_to_string(result.path.join("output.md")).expect("output");
        assert_eq!(output, "# Output\n\n");
    }

    #[test]
    fn init_rejects_unsafe_capsule_ids() {
        let temp = tempdir().expect("tempdir");
        let mut args = init_args(temp.path().join("root"));
        args.id = Some("../escape".to_string());

        let error = init_capsule(args).expect_err("unsafe id rejected");
        assert!(error.to_string().contains("capsule id"));
        assert!(!temp.path().join("escape").exists());
    }

    #[test]
    fn force_replaces_existing_capsule() {
        let temp = tempdir().expect("tempdir");
        let args = init_args(temp.path().to_path_buf());
        let result = init_capsule(args).expect("init capsule");
        fs::write(result.path.join("stale.txt"), "old").expect("write stale marker");

        let mut replacement = init_args(temp.path().to_path_buf());
        replacement.force = true;
        let result = init_capsule(replacement).expect("replace capsule");

        assert!(!result.path.join("stale.txt").exists());
        let evidence = fs::read_to_string(result.path.join("evidence.jsonl")).expect("evidence");
        assert_eq!(evidence.lines().count(), 1);
    }

    #[test]
    fn force_replaces_file_at_capsule_path() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("20260509-040000-capsule-cli");
        fs::write(&path, "old").expect("write stale file");

        let mut replacement = init_args(temp.path().to_path_buf());
        replacement.force = true;
        let result = init_capsule(replacement).expect("replace file");

        assert!(result.path.is_dir());
        assert!(result.path.join("capsule.json").is_file());
    }

    #[test]
    fn status_and_render_use_capsule_contract() {
        let temp = tempdir().expect("tempdir");
        let result = init_capsule(init_args(temp.path().to_path_buf())).expect("init capsule");

        let status = capsule_status(&result.path).expect("status");
        assert_eq!(status.id, "20260509-040000-capsule-cli");
        assert_eq!(status.issues, vec![22]);

        let rendered = render_capsule(&result.path).expect("render");
        assert!(rendered.markdown.contains("# Build capsule CLI"));
        assert!(rendered.markdown.contains("- Issues: #22"));
    }

    #[test]
    fn validate_reports_missing_files() {
        let temp = tempdir().expect("tempdir");
        let validation = validate_capsule(temp.path()).expect("validate");

        assert!(!validation.valid);
        assert!(
            validation
                .errors
                .iter()
                .any(|error| error.contains("capsule.json"))
        );
        assert!(
            validation
                .errors
                .iter()
                .all(|error| !error.contains("failed to open"))
        );
    }

    #[test]
    fn run_from_emits_json_envelope() {
        let temp = tempdir().expect("tempdir");
        let output = run_from([
            "codex-dev",
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
            temp.path().to_str().expect("utf8 temp path"),
            "--id",
            "test-capsule",
            "--created-at",
            "2026-05-09T04:00:00Z",
        ])
        .expect("run");
        let value: Value = serde_json::from_str(&output).expect("json output");
        assert_eq!(value["schema"], OUTPUT_SCHEMA);
        assert_eq!(value["ok"], true);
        assert_eq!(value["command"], "capsule init");
        assert_eq!(value["result"]["capsule"]["issues"][0], 22);
    }

    #[test]
    fn run_from_emits_json_error_envelope() {
        let temp = tempdir().expect("tempdir");
        init_capsule(init_args(temp.path().to_path_buf())).expect("init capsule");

        let output = run_from([
            "codex-dev",
            "--json",
            "capsule",
            "init",
            "--title",
            "Build capsule CLI",
            "--root",
            temp.path().to_str().expect("utf8 temp path"),
            "--id",
            "20260509-040000-capsule-cli",
            "--created-at",
            "2026-05-09T04:00:00Z",
        ])
        .expect("json error envelope");
        let value: Value = serde_json::from_str(&output).expect("json output");
        assert_eq!(value["schema"], OUTPUT_SCHEMA);
        assert_eq!(value["ok"], false);
        assert_eq!(value["command"], "capsule init");
        assert!(
            value["result"]["error"]["message"]
                .as_str()
                .expect("message")
                .contains("already exists")
        );
    }

    #[test]
    fn policy_manifest_lists_repo_native_gates() {
        let manifest = policy_manifest(
            PolicyProfile::CodexDev,
            "2026-05-09T05:00:00Z".parse().unwrap(),
        );

        assert_eq!(manifest.schema, POLICY_GATES_SCHEMA);
        assert_eq!(manifest.profile, PolicyProfile::CodexDev);
        assert!(
            manifest
                .gates
                .iter()
                .any(|gate| gate.id == "codex-dev-test")
        );
        assert!(manifest.gates.iter().all(|gate| gate.required));
        assert!(manifest.gates.iter().all(|gate| !gate.network));
        assert!(manifest.gates.iter().all(|gate| !gate.secrets));
    }

    #[test]
    fn policy_dry_run_records_verification_and_evidence() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        let result = run_policy_gates(
            PolicyRunArgs {
                capsule: capsule.clone(),
                repo_root: None,
                profile: PolicyProfile::CodexDev,
                execute: false,
                allow_network: false,
                keep_going: false,
                checked_at: None,
            },
            "2026-05-09T05:00:00Z".parse().unwrap(),
        )
        .expect("policy dry run");

        assert!(result.passed);
        assert!(result.dry_run);
        assert!(
            result
                .gates
                .iter()
                .all(|gate| gate.status == GateStatus::Planned)
        );

        let verification: Verification =
            read_json(&capsule.join("verification.json")).expect("verification");
        assert_eq!(verification.required.len(), result.gates.len());
        assert_eq!(verification.required[0].status, "planned");

        let evidence = fs::read_to_string(capsule.join("evidence.jsonl")).expect("evidence");
        assert!(evidence.contains("Policy gate cargo-fmt planned"));
    }

    #[test]
    fn policy_execution_reports_failed_gate() {
        let missing = PolicyGate {
            id: "missing-command".to_string(),
            name: "missing command".to_string(),
            command: vec!["codex-dev-command-that-does-not-exist".to_string()],
            source: "test".to_string(),
            required: true,
            network: false,
            secrets: false,
        };

        let result = execute_gate(&missing, None);

        assert_eq!(result.status, GateStatus::Failed);
        assert!(
            result
                .error
                .expect("error")
                .contains("failed to start command")
        );
    }

    #[test]
    fn policy_execution_uses_repo_root() {
        let temp = tempdir().expect("tempdir");
        fs::write(temp.path().join("marker.txt"), "ok").expect("marker");
        let gate = PolicyGate {
            id: "repo-root-marker".to_string(),
            name: "repo root marker".to_string(),
            command: vec![
                "python3".to_string(),
                "-c".to_string(),
                "from pathlib import Path; raise SystemExit(0 if Path('marker.txt').is_file() else 7)"
                    .to_string(),
            ],
            source: "test".to_string(),
            required: true,
            network: false,
            secrets: false,
        };

        let result = execute_gate(&gate, Some(temp.path()));

        assert_eq!(result.status, GateStatus::Passed);
    }

    #[test]
    fn policy_failure_preserves_subprocess_output() {
        let gate = PolicyGate {
            id: "stderr-command".to_string(),
            name: "stderr command".to_string(),
            command: vec![
                "python3".to_string(),
                "-c".to_string(),
                "import sys; sys.stderr.write('boom'); raise SystemExit(9)".to_string(),
            ],
            source: "test".to_string(),
            required: true,
            network: false,
            secrets: false,
        };

        let result = execute_gate(&gate, None);

        assert_eq!(result.status, GateStatus::Failed);
        assert_eq!(result.exit_code, Some(9));
        assert_eq!(result.stderr.as_deref(), Some("boom"));
    }

    #[test]
    fn pr_plan_lists_existing_review_commands() {
        let plan = pr_control_plan(
            "BjornMelin/dev-skills".to_string(),
            25,
            "2026-05-09T05:00:00Z".parse().unwrap(),
        );

        assert_eq!(plan.schema, PR_CONTROL_PLAN_SCHEMA);
        assert_eq!(plan.repository, "BjornMelin/dev-skills");
        assert_eq!(plan.number, 25);
        assert!(plan.commands.iter().all(|command| command.network));
        assert!(plan.commands.iter().any(|command| {
            command.id == "review-pack-start" && command.command[0] == "review-pack"
        }));
        let remaining = plan
            .commands
            .iter()
            .find(|command| command.id == "review-pack-remaining")
            .expect("remaining command");
        assert!(!remaining.required);
        assert!(remaining.manual_input.is_some());
        assert!(plan.commands.iter().any(|command| {
            command.id == "gh-pr-review-fix" && command.command[0] == "gh-pr-review-fix"
        }));
    }

    #[test]
    fn pr_record_updates_capsule_contracts() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let source = temp.path().join("pr-snapshot.json");
        fs::write(
            &source,
            r#"{
  "repository": "BjornMelin/dev-skills",
  "number": 25,
  "url": "https://github.com/BjornMelin/dev-skills/pull/25",
  "state": "OPEN",
  "checks": [
    {
      "name": "CodeRabbit",
      "status": "COMPLETED",
      "conclusion": "SUCCESS",
      "url": "https://example.test/check"
    }
  ],
  "review_threads": {
    "unresolved": 0
  }
}"#,
        )
        .expect("write fixture");

        let result = record_pr_snapshot(
            PrRecordArgs {
                capsule: capsule.clone(),
                source,
                checked_at: None,
            },
            "2026-05-09T05:00:00Z".parse().unwrap(),
        )
        .expect("record pr");

        assert_eq!(result.pr.schema, PR_SCHEMA);
        assert_eq!(result.pr.state, "open");
        assert_eq!(result.pr.checks[0].status, "completed");
        assert_eq!(result.pr.checks[0].conclusion.as_deref(), Some("success"));
        assert_eq!(result.pr.review_threads.unresolved, 0);

        let pr: PrEvidence = read_json(&capsule.join("pr.json")).expect("pr json");
        assert_eq!(pr.number, Some(25));

        let capsule_state: Capsule = read_json(&capsule.join("capsule.json")).expect("capsule");
        assert_eq!(capsule_state.pull_requests, vec![25]);

        let evidence = fs::read_to_string(capsule.join("evidence.jsonl")).expect("evidence");
        assert!(evidence.contains("PR snapshot recorded"));
        assert!(evidence.contains("codex-dev pr record --capsule"));
        assert!(evidence.contains("--source"));
        assert!(evidence.contains("--checked-at 2026-05-09T05:00:00Z"));
    }

    #[test]
    fn validate_rejects_drifted_pr_schema_name() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let mut pr: Value = read_json(&capsule.join("pr.json")).expect("pr json");
        pr["schema"] = json!("codex-dev.pr-evidence.v1");
        write_json(capsule.join("pr.json"), &pr).expect("write drifted pr schema");

        let validation = validate_capsule(&capsule).expect("validate");

        assert!(!validation.valid);
        assert!(
            validation
                .errors
                .iter()
                .any(|error| { error == &format!("pr.json schema must be {PR_SCHEMA}") })
        );
    }

    #[test]
    fn pr_record_rejects_missing_pr_json() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        fs::remove_file(capsule.join("pr.json")).expect("remove placeholder");
        let source = temp.path().join("pr-snapshot.json");
        fs::write(
            &source,
            r#"{
  "repository": "BjornMelin/dev-skills",
  "number": 25,
  "state": "OPEN",
  "review_threads": {
    "unresolved": 0
  }
}"#,
        )
        .expect("write fixture");

        let error = record_pr_snapshot(
            PrRecordArgs {
                capsule: capsule.clone(),
                source,
                checked_at: None,
            },
            "2026-05-09T05:00:00Z".parse().unwrap(),
        )
        .expect_err("missing pr.json rejected");

        assert!(error.to_string().contains("missing required file: pr.json"));
        assert!(!capsule.join("pr.json").exists());
    }

    #[test]
    fn pr_record_command_preserves_timestamp_precision() {
        let command = render_pr_record_command(
            Path::new("/tmp/capsule"),
            Path::new("/tmp/pr-snapshot.json"),
            "2026-05-09T05:00:00.123456789Z".parse().unwrap(),
        );

        assert!(command.contains("--checked-at 2026-05-09T05:00:00.123456789Z"));
    }

    #[test]
    fn command_rendering_preserves_argument_boundaries() {
        let command = vec![
            "python3".to_string(),
            "-c".to_string(),
            "print('hello world')".to_string(),
        ];

        assert_eq!(
            render_command(&command),
            "python3 -c 'print('\\''hello world'\\'')'"
        );
    }
}
