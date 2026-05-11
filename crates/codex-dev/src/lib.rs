use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, SecondsFormat, Utc};
use clap::{Args, Parser, Subcommand};
use codex_dev_core::{
    AppendEvidenceArgs, Capsule, CapsuleStatus, EVIDENCE_SCHEMA, EvidenceKind, EvidenceKindSummary,
    EvidenceRecord, GateRecord, GateStatus, InitArgs, OUTPUT_SCHEMA, POLICY_GATES_SCHEMA,
    PR_CONTROL_PLAN_SCHEMA, PolicyGate, PolicyGateResult, PolicyManifest, PolicyProfile,
    PolicyRunResult, PrControlCommand, PrControlPlan, PrRecordArgs, RecordSubagentOutcomeArgs,
    RecordSubagentPlanArgs, RecordSubagentSynthesisArgs, SubagentDisposition,
    SubagentOutcomeStatus, SubagentSynthesisStatus, Verification, append_evidence, append_jsonl,
    capsule_status, ensure_regular_contract_files, init_capsule, pr_status, read_json,
    record_pr_snapshot, record_subagent_outcome, record_subagent_plan, record_subagent_synthesis,
    render_capsule, render_command, render_pr_label, render_pr_status, validate_capsule,
    write_json,
};
use serde::Serialize;
use serde_json::{Value, json};

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
            Commands::Evidence { command } => match command {
                EvidenceCommand::Append(_) => "evidence append",
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
            Commands::Subagents { command } => match command {
                SubagentsCommand::Plan(_) => "subagents record-plan",
                SubagentsCommand::Outcome(_) => "subagents record-outcome",
                SubagentsCommand::Synthesis(_) => "subagents record-synthesis",
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
    /// Append typed evidence records to task capsules.
    Evidence {
        #[command(subcommand)]
        command: EvidenceCommand,
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
    /// Record subspawn plans, outcomes, and synthesis into task capsules.
    Subagents {
        #[command(subcommand)]
        command: SubagentsCommand,
    },
}

#[derive(Subcommand, Debug)]
enum CapsuleCommand {
    /// Create a new local task capsule.
    Init(Box<CapsuleInitArgs>),
    /// Validate a task capsule directory.
    Validate(PathArgs),
    /// Print task capsule status.
    Status(PathArgs),
    /// Render a Markdown summary from capsule state.
    Render(PathArgs),
}

#[derive(Subcommand, Debug)]
enum EvidenceCommand {
    /// Append one typed evidence record to evidence.jsonl.
    Append(EvidenceAppendArgs),
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
    Record(PrRecordCliArgs),
    /// Print the PR snapshot currently stored in a task capsule.
    Status(PrStatusArgs),
}

#[derive(Subcommand, Debug)]
enum SubagentsCommand {
    /// Record a subspawn_plan.py JSON plan into subagents.json.
    #[command(name = "record-plan")]
    Plan(SubagentsRecordPlanArgs),
    /// Record one planned subagent's outcome and disposition.
    #[command(name = "record-outcome")]
    Outcome(SubagentsRecordOutcomeArgs),
    /// Record parent synthesis for a completed subagent batch.
    #[command(name = "record-synthesis")]
    Synthesis(SubagentsRecordSynthesisArgs),
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
pub struct PrRecordCliArgs {
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

impl PrRecordCliArgs {
    fn into_core(self) -> (PrRecordArgs, DateTime<Utc>) {
        let checked_at = self.checked_at.unwrap_or_else(Utc::now);
        let command = render_pr_record_command(&self.capsule, &self.source, checked_at);
        (
            PrRecordArgs {
                capsule: self.capsule,
                source: self.source,
                command: Some(command),
            },
            checked_at,
        )
    }
}

#[derive(Args, Debug)]
pub struct PrStatusArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
}

#[derive(Args, Debug)]
pub struct EvidenceAppendArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
    #[arg(long, value_name = "KIND")]
    pub kind: EvidenceKind,
    #[arg(long)]
    pub summary: String,
    #[arg(long, value_name = "RFC3339")]
    pub at: Option<DateTime<Utc>>,
    #[arg(long, value_name = "COMMAND")]
    pub command: Option<String>,
    #[arg(long, value_name = "EXIT_CODE")]
    pub exit_code: Option<i32>,
    #[arg(long = "source-id", value_name = "SOURCE_ID")]
    pub source_ids: Vec<String>,
    #[arg(long)]
    pub actor: Option<String>,
    #[arg(long)]
    pub tool: Option<String>,
    #[arg(long, value_name = "0_TO_100")]
    pub confidence: Option<u8>,
    #[arg(long = "residual-risk")]
    pub residual_risk: Option<String>,
    #[arg(long = "artifact", value_name = "ARTIFACT")]
    pub artifacts: Vec<String>,
}

impl EvidenceAppendArgs {
    fn into_core(self) -> AppendEvidenceArgs {
        let at = self.at.unwrap_or_else(Utc::now);
        AppendEvidenceArgs {
            capsule: self.capsule,
            record: EvidenceRecord {
                schema: EVIDENCE_SCHEMA.to_string(),
                kind: self.kind,
                at,
                summary: self.summary,
                command: self.command,
                exit_code: self.exit_code,
                source_ids: self.source_ids,
                actor: self.actor,
                tool: self.tool,
                confidence: self.confidence,
                residual_risk: self.residual_risk,
                artifacts: self.artifacts,
            },
        }
    }
}

#[derive(Args, Debug)]
pub struct SubagentsRecordPlanArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
    #[arg(long = "batch-id", value_name = "BATCH_ID")]
    pub batch_id: String,
    #[arg(long, value_name = "SUBSPAWN_PLAN_JSON")]
    pub source: PathBuf,
    #[arg(long, value_name = "COMMAND")]
    pub command: Option<String>,
    #[arg(long = "recorded-at", value_name = "RFC3339")]
    pub recorded_at: Option<DateTime<Utc>>,
}

impl SubagentsRecordPlanArgs {
    fn into_core(self) -> RecordSubagentPlanArgs {
        RecordSubagentPlanArgs {
            capsule: self.capsule,
            batch_id: self.batch_id,
            source: self.source,
            command: self.command,
            recorded_at: self.recorded_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Args, Debug)]
pub struct SubagentsRecordOutcomeArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
    #[arg(long = "batch-id", value_name = "BATCH_ID")]
    pub batch_id: String,
    #[arg(long, value_name = "ROLE")]
    pub role: String,
    #[arg(long, value_name = "STATUS")]
    pub status: SubagentOutcomeStatus,
    #[arg(long)]
    pub summary: String,
    #[arg(long, value_name = "DISPOSITION")]
    pub disposition: SubagentDisposition,
    #[arg(
        long = "human-verified",
        help = "Confirm a human parent session verified this outcome and disposition"
    )]
    pub human_verified: bool,
    #[arg(long = "source-id", value_name = "SOURCE_ID")]
    pub source_ids: Vec<String>,
    #[arg(long = "artifact", value_name = "ARTIFACT")]
    pub artifacts: Vec<String>,
    #[arg(long = "recorded-at", value_name = "RFC3339")]
    pub recorded_at: Option<DateTime<Utc>>,
}

impl SubagentsRecordOutcomeArgs {
    fn into_core(self) -> RecordSubagentOutcomeArgs {
        RecordSubagentOutcomeArgs {
            capsule: self.capsule,
            batch_id: self.batch_id,
            role: self.role,
            status: self.status,
            summary: self.summary,
            disposition: self.disposition,
            human_verified: self.human_verified,
            source_ids: self.source_ids,
            artifacts: self.artifacts,
            recorded_at: self.recorded_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Args, Debug)]
pub struct SubagentsRecordSynthesisArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
    #[arg(long = "batch-id", value_name = "BATCH_ID")]
    pub batch_id: String,
    #[arg(long, value_name = "STATUS")]
    pub status: SubagentSynthesisStatus,
    #[arg(long)]
    pub summary: String,
    #[arg(
        long = "human-verified",
        help = "Confirm a human parent session verified this synthesis"
    )]
    pub human_verified: bool,
    #[arg(long = "source-id", value_name = "SOURCE_ID")]
    pub source_ids: Vec<String>,
    #[arg(long = "artifact", value_name = "ARTIFACT")]
    pub artifacts: Vec<String>,
    #[arg(long = "recorded-at", value_name = "RFC3339")]
    pub recorded_at: Option<DateTime<Utc>>,
}

impl SubagentsRecordSynthesisArgs {
    fn into_core(self) -> RecordSubagentSynthesisArgs {
        RecordSubagentSynthesisArgs {
            capsule: self.capsule,
            batch_id: self.batch_id,
            status: self.status,
            summary: self.summary,
            human_verified: self.human_verified,
            source_ids: self.source_ids,
            artifacts: self.artifacts,
            recorded_at: self.recorded_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Args, Debug)]
pub struct PolicyManifestArgs {
    #[arg(long, default_value_t = PolicyProfile::CodexDev)]
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
    #[arg(long, default_value_t = PolicyProfile::CodexDev)]
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
pub struct CapsuleInitArgs {
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
    #[arg(long, default_value_t = CapsuleStatus::Active)]
    status: CapsuleStatus,
    #[arg(long, value_name = "RFC3339")]
    created_at: Option<DateTime<Utc>>,
    #[arg(long)]
    force: bool,
}

impl CapsuleInitArgs {
    fn into_core(self) -> InitArgs {
        let created_at = self.created_at.unwrap_or_else(Utc::now);
        let branch = self
            .branch
            .unwrap_or_else(|| current_git_branch().unwrap_or_else(|| "unknown".to_string()));
        let objective = self.objective.unwrap_or_else(|| self.title.clone());
        let policy_manifest = policy_manifest(PolicyProfile::CodexDev, created_at);
        InitArgs {
            title: self.title,
            objective,
            branch,
            base_branch: self.base_branch,
            issues: self.issues,
            pull_requests: self.pull_requests,
            root: self.root,
            slug: self.slug,
            id: self.id,
            status: self.status,
            created_at,
            policy_manifest,
            force: self.force,
        }
    }
}

#[derive(Args, Debug)]
pub struct PathArgs {
    #[arg(value_name = "CAPSULE_DIR")]
    path: PathBuf,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
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
                let result = init_capsule(args.into_core())?;
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
                    human: format!(
                        "{} [{}] on {}; evidence: {}",
                        result.title,
                        result.status,
                        result.branch,
                        render_evidence_counts(&result.evidence.by_kind)
                    ),
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
        Commands::Evidence { command } => match command {
            EvidenceCommand::Append(args) => {
                let result = append_evidence(args.into_core())?;
                Ok(CommandOutput {
                    ok: true,
                    command: "evidence append",
                    human: format!(
                        "appended {} evidence to {}; {} total evidence record(s)",
                        result.record.kind,
                        result.capsule.display(),
                        result.evidence.total
                    ),
                    result: serde_json::to_value(result)?,
                })
            }
        },
        Commands::Subagents { command } => match command {
            SubagentsCommand::Plan(args) => {
                let result = record_subagent_plan(args.into_core())?;
                Ok(CommandOutput {
                    ok: true,
                    command: "subagents record-plan",
                    human: format!(
                        "recorded subagent plan {} with {} role(s)",
                        result.batch.id,
                        result.batch.agents.len()
                    ),
                    result: serde_json::to_value(result)?,
                })
            }
            SubagentsCommand::Outcome(args) => {
                let result = record_subagent_outcome(args.into_core())?;
                Ok(CommandOutput {
                    ok: true,
                    command: "subagents record-outcome",
                    human: format!(
                        "recorded {} outcome for {} in {}",
                        result.agent.status, result.agent.role, result.batch.id
                    ),
                    result: serde_json::to_value(result)?,
                })
            }
            SubagentsCommand::Synthesis(args) => {
                let result = record_subagent_synthesis(args.into_core())?;
                Ok(CommandOutput {
                    ok: true,
                    command: "subagents record-synthesis",
                    human: format!(
                        "recorded {} synthesis for {}",
                        result.synthesis.status, result.batch.id
                    ),
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
                let (args, checked_at) = args.into_core();
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

fn render_evidence_counts(by_kind: &[EvidenceKindSummary]) -> String {
    if by_kind.is_empty() {
        return "0 records".to_string();
    }

    by_kind
        .iter()
        .map(|summary| format!("{}={}", summary.kind, summary.count))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn policy_manifest(profile: PolicyProfile, generated_at: DateTime<Utc>) -> PolicyManifest {
    PolicyManifest {
        schema: POLICY_GATES_SCHEMA.to_string(),
        profile,
        generated_at,
        gates: built_in_gates(profile),
    }
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

pub fn run_policy_gates(args: PolicyRunArgs, checked_at: DateTime<Utc>) -> Result<PolicyRunResult> {
    ensure_regular_contract_files(&args.capsule)?;
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

fn built_in_gates(profile: PolicyProfile) -> Vec<PolicyGate> {
    match profile {
        PolicyProfile::CodexDev => vec![
            policy_gate(
                "cargo-fmt",
                "Rust workspace formatting",
                ["cargo", "fmt", "--all", "--check"],
            ),
            policy_gate(
                "codex-dev-core-clippy",
                "codex-dev-core Clippy",
                [
                    "cargo",
                    "clippy",
                    "-p",
                    "codex-dev-core",
                    "--all-targets",
                    "--",
                    "-D",
                    "warnings",
                ],
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
                "codex-dev-core-check",
                "codex-dev-core cargo check",
                ["cargo", "check", "-p", "codex-dev-core"],
            ),
            policy_gate(
                "codex-dev-check",
                "codex-dev cargo check",
                ["cargo", "check", "-p", "codex-dev"],
            ),
            policy_gate(
                "codex-dev-core-test",
                "codex-dev-core tests",
                ["cargo", "test", "-p", "codex-dev-core"],
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

fn render_pr_record_command(capsule: &Path, source: &Path, checked_at: DateTime<Utc>) -> String {
    render_command(&[
        "codex-dev".to_string(),
        "pr".to_string(),
        "record".to_string(),
        "--capsule".to_string(),
        capsule.display().to_string(),
        "--source".to_string(),
        source.display().to_string(),
        "--checked-at".to_string(),
        checked_at.to_rfc3339_opts(SecondsFormat::AutoSi, true),
    ])
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
    ensure_regular_contract_files(capsule_path)?;
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
                source_ids: Vec::new(),
                actor: None,
                tool: None,
                confidence: None,
                residual_risk: None,
                artifacts: vec!["verification.json".to_string()],
            },
        )?;
    }

    let mut capsule: Capsule = read_json(&capsule_path.join("capsule.json"))?;
    capsule.updated_at = std::cmp::max(capsule.updated_at, checked_at);
    write_json(capsule_path.join("capsule.json"), &capsule)?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn init_args(root: PathBuf) -> InitArgs {
        let created_at = "2026-05-09T04:00:00Z".parse().expect("valid timestamp");
        InitArgs {
            title: "Build capsule CLI".to_string(),
            objective: "Create task capsules".to_string(),
            branch: "feat/codex-dev-task-capsules".to_string(),
            base_branch: "main".to_string(),
            issues: vec![22],
            pull_requests: Vec::new(),
            root,
            slug: Some("capsule-cli".to_string()),
            id: Some("20260509-040000-capsule-cli".to_string()),
            status: CapsuleStatus::Active,
            created_at,
            policy_manifest: policy_manifest(PolicyProfile::CodexDev, created_at),
            force: false,
        }
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
    fn policy_run_keeps_capsule_updated_at_monotonic() {
        let temp = tempdir().expect("tempdir");
        let mut args = init_args(temp.path().to_path_buf());
        args.created_at = "2026-05-09T10:00:00Z".parse().unwrap();
        let capsule = init_capsule(args).expect("init capsule").path;

        run_policy_gates(
            PolicyRunArgs {
                capsule: capsule.clone(),
                repo_root: None,
                profile: PolicyProfile::CodexDev,
                execute: false,
                allow_network: false,
                keep_going: false,
                checked_at: None,
            },
            "2026-05-09T09:00:00Z".parse().unwrap(),
        )
        .expect("policy dry run");

        let capsule_state: Capsule = read_json(&capsule.join("capsule.json")).expect("capsule");
        assert_eq!(
            capsule_state.updated_at,
            "2026-05-09T10:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn policy_run_rejects_symlinked_contract_file_before_writing() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        let evidence_path = capsule.join("evidence.jsonl");
        let outside_path = temp.path().join("outside-evidence.jsonl");
        fs::write(&outside_path, "external evidence\n").expect("outside evidence");
        fs::remove_file(&evidence_path).expect("remove evidence");
        std::os::unix::fs::symlink(&outside_path, &evidence_path).expect("symlink evidence");

        let error = run_policy_gates(
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
        .expect_err("policy run rejects symlinked contract file");

        assert!(
            format!("{error:#}").contains("symlinked capsule contract file"),
            "{error:#}"
        );
        assert_eq!(
            fs::read_to_string(outside_path).expect("outside unchanged"),
            "external evidence\n"
        );
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
}
