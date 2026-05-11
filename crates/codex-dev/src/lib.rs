use std::collections::BTreeSet;
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
    PolicyRunResult, PrControlCommand, PrControlPlan, PrRecordArgs, PrRecordSourceKind,
    RecordSubagentOutcomeArgs, RecordSubagentPlanArgs, RecordSubagentSynthesisArgs,
    SubagentDisposition, SubagentOutcomeStatus, SubagentSynthesisStatus, Verification,
    append_evidence, append_jsonl, capsule_status, ensure_regular_contract_files, init_capsule,
    pr_status, read_json, record_pr_snapshot, record_subagent_outcome, record_subagent_plan,
    record_subagent_synthesis, render_capsule, render_command, render_pr_label, render_pr_status,
    validate_capsule, write_json,
};
use serde::Serialize;
use serde_json::{Value, json};

const POLICY_DOCS_CHECK_SCHEMA: &str = "codex-dev.policy-docs-check.v1";
const POLICY_DOCS_SMOKE_MARKER: &str = "policy-manifest-smoke";
const POLICY_DOCS_ALL_MARKER: &str = "policy-manifest-all";

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
                PolicyCommand::DocsCheck(_) => "policy docs-check",
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
    /// Check machine-owned documentation mirrors for policy manifest commands.
    #[command(name = "docs-check")]
    DocsCheck(PolicyDocsCheckArgs),
    /// Plan or execute gates and record capsule evidence.
    Run(PolicyRunArgs),
}

#[derive(Subcommand, Debug)]
enum PrCommand {
    /// Print the live-command plan for PR evidence capture.
    Plan(PrPlanArgs),
    /// Normalize and record a PR evidence source into a task capsule.
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
        value_name = "SOURCE_JSON",
        help = "Local PR evidence source to normalize and record"
    )]
    pub source: PathBuf,
    #[arg(long, value_name = "KIND", default_value_t = PrRecordSourceKind::Normalized)]
    pub source_kind: PrRecordSourceKind,
    #[arg(long, value_name = "OWNER/REPO")]
    pub repo: Option<String>,
    #[arg(long, value_name = "PR_NUMBER")]
    pub number: Option<u64>,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
    #[arg(long, value_name = "RFC3339")]
    pub retrieved_at: Option<DateTime<Utc>>,
    #[arg(long, value_name = "COMMAND")]
    pub source_command: Option<String>,
}

impl PrRecordCliArgs {
    fn into_core(self) -> (PrRecordArgs, DateTime<Utc>) {
        let checked_at = self.checked_at.unwrap_or_else(Utc::now);
        let command = render_pr_record_command(&self, checked_at);
        (
            PrRecordArgs {
                capsule: self.capsule,
                source: self.source,
                source_kind: self.source_kind,
                repository: self.repo,
                number: self.number,
                retrieved_at: self.retrieved_at,
                source_command: self.source_command,
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
    #[arg(
        long,
        default_value_t = PolicyProfile::CodexDev,
        help = "Policy profile: codex_dev, codex_dev_tui, codex_research, skills, bootstrap_install, docs, release, or full_local"
    )]
    pub profile: PolicyProfile,
    #[arg(long, value_name = "RFC3339")]
    pub generated_at: Option<DateTime<Utc>>,
}

#[derive(Args, Debug)]
pub struct PolicyDocsCheckArgs {
    #[arg(
        long,
        value_name = "REPO_ROOT",
        help = "Repository root containing the checked documentation"
    )]
    pub repo_root: Option<PathBuf>,
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
    #[arg(
        long,
        default_value_t = PolicyProfile::CodexDev,
        help = "Policy profile: codex_dev, codex_dev_tui, codex_research, skills, bootstrap_install, docs, release, or full_local"
    )]
    pub profile: PolicyProfile,
    #[arg(long, help = "Execute gates instead of recording a dry-run plan")]
    pub execute: bool,
    #[arg(long, help = "Permit gates marked as network-using")]
    pub allow_network: bool,
    #[arg(long, help = "Permit gates marked as requiring secrets")]
    pub allow_secrets: bool,
    #[arg(long, help = "Continue executing after a failed required gate")]
    pub keep_going: bool,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PolicyDocsCheckResult {
    pub schema: &'static str,
    pub repo_root: PathBuf,
    pub passed: bool,
    pub blocks: Vec<PolicyDocsBlockResult>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PolicyDocsBlockResult {
    pub path: String,
    pub marker: String,
    pub profiles: Vec<PolicyProfile>,
    pub expected_commands: Vec<String>,
    pub actual_commands: Vec<String>,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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
            PolicyCommand::DocsCheck(args) => {
                let result = policy_docs_check(args.repo_root.as_deref())?;
                let failed = result.blocks.iter().filter(|block| !block.passed).count();
                let human = if result.passed {
                    format!(
                        "checked {} policy documentation mirror(s)",
                        result.blocks.len()
                    )
                } else {
                    format!(
                        "found {failed} stale policy documentation mirror(s) out of {}",
                        result.blocks.len()
                    )
                };
                Ok(CommandOutput {
                    ok: result.passed,
                    command: "policy docs-check",
                    human,
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
                let review_summary = if result.pr.review_threads.authoritative {
                    format!(
                        "{} unresolved thread(s)",
                        result.pr.review_threads.unresolved
                    )
                } else {
                    "review threads not checked".to_string()
                };
                let human = format!(
                    "recorded PR snapshot for {} with {review_summary}",
                    render_pr_label(&result.pr)
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

pub fn policy_docs_check(explicit_repo_root: Option<&Path>) -> Result<PolicyDocsCheckResult> {
    let repo_root = resolve_policy_docs_repo_root(explicit_repo_root)?;
    let blocks = policy_doc_block_specs()
        .iter()
        .map(|spec| check_policy_doc_block(&repo_root, spec))
        .collect::<Vec<_>>();
    let passed = blocks.iter().all(|block| block.passed);

    Ok(PolicyDocsCheckResult {
        schema: POLICY_DOCS_CHECK_SCHEMA,
        repo_root,
        passed,
        blocks,
    })
}

#[derive(Clone, Copy, Debug)]
struct PolicyDocBlockSpec {
    path: &'static str,
    marker: &'static str,
    kind: PolicyDocBlockKind,
}

#[derive(Clone, Copy, Debug)]
enum PolicyDocBlockKind {
    Smoke,
    AllProfiles,
}

fn policy_doc_block_specs() -> [PolicyDocBlockSpec; 5] {
    [
        PolicyDocBlockSpec {
            path: "AGENTS.md",
            marker: POLICY_DOCS_SMOKE_MARKER,
            kind: PolicyDocBlockKind::Smoke,
        },
        PolicyDocBlockSpec {
            path: "README.md",
            marker: POLICY_DOCS_SMOKE_MARKER,
            kind: PolicyDocBlockKind::Smoke,
        },
        PolicyDocBlockSpec {
            path: "docs/reference/codex-dev-cli.md",
            marker: POLICY_DOCS_SMOKE_MARKER,
            kind: PolicyDocBlockKind::Smoke,
        },
        PolicyDocBlockSpec {
            path: "docs/runbooks/validation.md",
            marker: POLICY_DOCS_SMOKE_MARKER,
            kind: PolicyDocBlockKind::Smoke,
        },
        PolicyDocBlockSpec {
            path: "docs/runbooks/validation.md",
            marker: POLICY_DOCS_ALL_MARKER,
            kind: PolicyDocBlockKind::AllProfiles,
        },
    ]
}

fn check_policy_doc_block(repo_root: &Path, spec: &PolicyDocBlockSpec) -> PolicyDocsBlockResult {
    let profiles = policy_doc_block_profiles(spec.kind);
    let expected_commands = profiles
        .iter()
        .map(|profile| policy_manifest_command(*profile))
        .collect::<Vec<_>>();

    let path = repo_root.join(spec.path);
    let (actual_commands, error) = match fs::read_to_string(&path) {
        Ok(contents) => match extract_policy_doc_commands(&contents, spec.marker) {
            Ok(commands) => (commands, None),
            Err(error) => (Vec::new(), Some(error.to_string())),
        },
        Err(error) => (
            Vec::new(),
            Some(format!("failed to read {}: {error}", path.display())),
        ),
    };
    let passed = error.is_none() && actual_commands == expected_commands;

    PolicyDocsBlockResult {
        path: spec.path.to_string(),
        marker: spec.marker.to_string(),
        profiles,
        expected_commands,
        actual_commands,
        passed,
        error,
    }
}

fn extract_policy_doc_commands(contents: &str, marker: &str) -> Result<Vec<String>> {
    let start = policy_doc_marker(marker, "start");
    let end = policy_doc_marker(marker, "end");
    let lines = contents.lines().collect::<Vec<_>>();
    let start_lines = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| policy_doc_marker_line(line, &start).then_some(index))
        .collect::<Vec<_>>();
    let end_lines = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| policy_doc_marker_line(line, &end).then_some(index))
        .collect::<Vec<_>>();
    if start_lines.len() != 1 || end_lines.len() != 1 {
        bail!(
            "expected exactly one {start:?} and one {end:?}, found {} and {}",
            start_lines.len(),
            end_lines.len()
        );
    }

    let start_line = start_lines[0];
    let end_line = end_lines[0];
    if start_line >= end_line {
        bail!("end marker appears before start marker");
    }

    Ok(lines[start_line + 1..end_line]
        .iter()
        .copied()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("```"))
        .map(str::to_string)
        .collect())
}

fn policy_doc_marker_line(line: &str, marker: &str) -> bool {
    let line = line.trim();
    line == marker || line.strip_prefix('#').map(str::trim) == Some(marker)
}

fn policy_doc_marker(marker: &str, side: &str) -> String {
    format!("codex-dev:{marker}:{side}")
}

fn policy_doc_block_profiles(kind: PolicyDocBlockKind) -> Vec<PolicyProfile> {
    match kind {
        PolicyDocBlockKind::Smoke => vec![PolicyProfile::CodexDev, PolicyProfile::FullLocal],
        PolicyDocBlockKind::AllProfiles => all_policy_profiles().to_vec(),
    }
}

fn policy_manifest_command(profile: PolicyProfile) -> String {
    format!("cargo run -q -p codex-dev -- --json policy manifest --profile {profile}")
}

pub fn pr_control_plan(
    repository: String,
    number: u64,
    generated_at: DateTime<Utc>,
) -> PrControlPlan {
    let (owner, name) = repository.split_once('/').unwrap_or((&repository, ""));
    let owner_arg = format!("owner={owner}");
    let name_arg = format!("name={name}");
    let number_arg = format!("number={number}");
    let reviews_path = format!("repos/{owner}/{name}/pulls/{number}/reviews");
    let review_comments_path = format!("repos/{owner}/{name}/pulls/{number}/comments");
    let review_threads_query = "query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){pullRequest(number:$number){reviewThreads(first:100){nodes{id isResolved isOutdated comments(first:10){nodes{id path line originalLine url}}}}}}}";
    let review_threads_query_arg = format!("query={review_threads_query}");

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
                    "number,url,state,isDraft,mergeable,reviewDecision,statusCheckRollup,headRefOid,updatedAt",
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
                    "--json",
                    "bucket,completedAt,description,event,link,name,startedAt,state,workflow",
                ],
            ),
            pr_control_command(
                "gh-reviews",
                "GitHub REST review submissions",
                ["gh", "api", &reviews_path],
            ),
            pr_control_command(
                "gh-review-comments",
                "GitHub REST review comments",
                ["gh", "api", &review_comments_path],
            ),
            pr_control_command(
                "gh-review-threads",
                "GitHub GraphQL review-thread state",
                [
                    "gh",
                    "api",
                    "graphql",
                    "-f",
                    &owner_arg,
                    "-f",
                    &name_arg,
                    "-F",
                    &number_arg,
                    "-f",
                    &review_threads_query_arg,
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
        } else if let Some(reason) = gate_skip_reason(gate, args.allow_network, args.allow_secrets)
        {
            skip_gate(gate, reason)
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
    record_policy_run(&args.capsule, &manifest, &results, checked_at)?;

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

fn all_policy_profiles() -> [PolicyProfile; 8] {
    [
        PolicyProfile::CodexDev,
        PolicyProfile::CodexDevTui,
        PolicyProfile::CodexResearch,
        PolicyProfile::Skills,
        PolicyProfile::BootstrapInstall,
        PolicyProfile::Docs,
        PolicyProfile::Release,
        PolicyProfile::FullLocal,
    ]
}

fn built_in_gates(profile: PolicyProfile) -> Vec<PolicyGate> {
    match profile {
        PolicyProfile::CodexDev => codex_dev_gates(),
        PolicyProfile::CodexDevTui => codex_dev_tui_gates(),
        PolicyProfile::CodexResearch => codex_research_gates(),
        PolicyProfile::Skills => skills_gates(),
        PolicyProfile::BootstrapInstall => bootstrap_install_gates(),
        PolicyProfile::Docs => docs_gates(),
        PolicyProfile::Release => release_gates(),
        PolicyProfile::FullLocal => full_local_gates(),
    }
}

fn codex_dev_gates() -> Vec<PolicyGate> {
    vec![
        cargo_fmt_gate(),
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
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means codex-dev-core has Rust lints or warnings that must be fixed before review.",
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
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means codex-dev CLI code has Rust lints or warnings that must be fixed before review.",
        ),
        policy_gate(
            "codex-dev-core-check",
            "codex-dev-core cargo check",
            ["cargo", "check", "-p", "codex-dev-core"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means codex-dev-core does not typecheck.",
        ),
        policy_gate(
            "codex-dev-check",
            "codex-dev cargo check",
            ["cargo", "check", "-p", "codex-dev"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means codex-dev does not typecheck.",
        ),
        policy_gate(
            "codex-dev-core-test",
            "codex-dev-core tests",
            ["cargo", "test", "-p", "codex-dev-core"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means shared capsule or evidence contracts regressed.",
        ),
        policy_gate(
            "codex-dev-test",
            "codex-dev tests",
            ["cargo", "test", "-p", "codex-dev"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means codex-dev CLI behavior or integration fixtures regressed.",
        ),
        policy_gate(
            "codex-dev-help",
            "codex-dev help smoke",
            ["cargo", "run", "-q", "-p", "codex-dev", "--", "--help"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the CLI cannot render its top-level Clap contract.",
        ),
        policy_gate(
            "codex-dev-policy-manifest",
            "codex-dev policy manifest smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-dev",
                "--",
                "--json",
                "policy",
                "manifest",
                "--profile",
                "codex_dev",
            ],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the codex_dev policy profile cannot be emitted as JSON.",
        ),
        policy_docs_check_gate(),
        policy_gate(
            "codex-dev-pr-plan-smoke",
            "codex-dev PR control-plan smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-dev",
                "--",
                "--json",
                "pr",
                "plan",
                "--repo",
                "BjornMelin/dev-skills",
                "--number",
                "25",
            ],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the local PR control-plan JSON contract regressed.",
        ),
        docs_links_gate(),
        diff_check_gate(),
    ]
}

fn codex_dev_tui_gates() -> Vec<PolicyGate> {
    vec![
        cargo_fmt_gate(),
        policy_gate(
            "codex-dev-tui-clippy",
            "codex-dev-tui Clippy",
            [
                "cargo",
                "clippy",
                "-p",
                "codex-dev-tui",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the TUI has Rust lints or warnings that must be fixed before review.",
        ),
        policy_gate(
            "codex-dev-tui-check",
            "codex-dev-tui cargo check",
            ["cargo", "check", "-p", "codex-dev-tui"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the TUI does not typecheck.",
        ),
        policy_gate(
            "codex-dev-tui-test",
            "codex-dev-tui tests",
            ["cargo", "test", "-p", "codex-dev-tui"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means TUI rendering or state fixtures regressed.",
        ),
        policy_gate(
            "codex-dev-tui-help",
            "codex-dev-tui help smoke",
            ["cargo", "run", "-q", "-p", "codex-dev-tui", "--", "--help"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the TUI cannot render its top-level Clap contract.",
        ),
    ]
}

fn codex_research_gates() -> Vec<PolicyGate> {
    vec![
        cargo_fmt_gate(),
        policy_gate(
            "codex-research-clippy",
            "codex-research Clippy",
            [
                "cargo",
                "clippy",
                "-p",
                "codex-research",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            "docs/runbooks/validation.md#rust-cli",
            ["cargo"],
            "Failure means codex-research has Rust lints or warnings that must be fixed before review.",
        ),
        policy_gate(
            "codex-research-check",
            "codex-research cargo check",
            ["cargo", "check", "-p", "codex-research"],
            "docs/runbooks/validation.md#rust-cli",
            ["cargo"],
            "Failure means codex-research does not typecheck.",
        ),
        policy_gate(
            "codex-research-test",
            "codex-research tests",
            ["cargo", "test", "-p", "codex-research"],
            "docs/runbooks/validation.md#rust-cli",
            ["cargo"],
            "Failure means codex-research unit or integration behavior regressed.",
        ),
        policy_gate(
            "codex-research-doctor",
            "codex-research doctor smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-research",
                "--",
                "--json",
                "doctor",
            ],
            "docs/runbooks/validation.md#rust-cli",
            ["cargo"],
            "Failure means codex-research local environment diagnostics regressed.",
        ),
        policy_gate(
            "codex-research-eval",
            "codex-research eval smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-research",
                "--",
                "--json",
                "eval",
            ],
            "docs/runbooks/validation.md#rust-cli",
            ["cargo"],
            "Failure means the embedded research eval suite regressed.",
        ),
        policy_gate(
            "codex-research-eval-list",
            "codex-research eval list",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-research",
                "--",
                "eval",
                "--list",
            ],
            "docs/runbooks/validation.md#rust-cli",
            ["cargo"],
            "Failure means the eval catalog cannot be listed.",
        ),
        policy_gate(
            "codex-research-eval-strict",
            "codex-research cited-claims strict eval",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-research",
                "--",
                "--json",
                "eval",
                "--task",
                "evidence-claims-cited",
                "--strict",
            ],
            "docs/runbooks/validation.md#rust-cli",
            ["cargo"],
            "Failure means strict cited-claim evidence behavior regressed.",
        ),
        policy_gate(
            "codex-research-plan-quick",
            "codex-research quick plan smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-research",
                "--",
                "--json",
                "plan",
                "validation smoke",
                "--profile",
                "quick",
            ],
            "docs/runbooks/validation.md#rust-cli",
            ["cargo"],
            "Failure means local research profile planning regressed.",
        ),
    ]
}

fn skills_gates() -> Vec<PolicyGate> {
    vec![
        policy_gate(
            "skills-quick-validate-all",
            "validate all skill metadata",
            [
                "bash",
                "-lc",
                "for d in skills/*; do [ -f \"$d/SKILL.md\" ] && python3 tools/skill/quick_validate.py \"$d\"; done",
            ],
            "docs/runbooks/validation.md#skills",
            ["bash", "python3"],
            "Failure means at least one skill is not AgentSkills-spec compliant.",
        ),
        policy_gate(
            "python-helpers-compile",
            "Python helper compile smoke",
            [
                "python3",
                "-m",
                "compileall",
                "-q",
                "skills/deep-researcher/scripts",
                "skills/subagent-creator/scripts",
                "skills/subspawn/scripts",
                "subagents/hardened-codex/scripts",
                "tools/bootstrap",
            ],
            "docs/runbooks/validation.md#python-helpers",
            ["python3"],
            "Failure means a tracked Python helper has syntax or import-time compilation errors.",
        ),
        policy_gate(
            "subagent-templates-validate",
            "validate bundled subagent templates",
            [
                "python3",
                "skills/subagent-creator/scripts/subagent_creator.py",
                "validate",
                "skills/deep-researcher/templates/agents",
                "skills/subagent-creator/templates/agents",
                "skills/subspawn/templates/agents",
                "subagents/hardened-codex/agents",
            ],
            "docs/runbooks/validation.md#subagent-templates",
            ["python3"],
            "Failure means one or more bundled custom subagent templates is invalid.",
        ),
        policy_gate(
            "subspawn-roles-validate",
            "validate subspawn role registry",
            [
                "python3",
                "skills/subspawn/scripts/subspawn_plan.py",
                "validate-roles",
            ],
            "docs/runbooks/validation.md#subagent-templates",
            ["python3"],
            "Failure means subspawn role discovery or duplicate-role policy regressed.",
        ),
        policy_gate(
            "subspawn-plan-research-smoke",
            "subspawn research plan smoke",
            [
                "python3",
                "skills/subspawn/scripts/subspawn_plan.py",
                "plan",
                "--preset",
                "research",
                "--task",
                "validation smoke",
                "--scope",
                "docs and template metadata",
                "--json",
            ],
            "docs/runbooks/validation.md#subagent-templates",
            ["python3"],
            "Failure means subspawn cannot emit the canonical research planning JSON.",
        ),
        policy_gate(
            "skill-subagent-eval",
            "skill and subagent eval smoke",
            ["python3", "tools/eval/skill_subagent_eval.py", "--json"],
            "docs/runbooks/validation.md#subagent-templates",
            ["python3"],
            "Failure means the local skill/subagent evaluation smoke regressed.",
        ),
    ]
}

fn bootstrap_install_gates() -> Vec<PolicyGate> {
    vec![
        bootstrap_pack_validate_gate(),
        policy_gate(
            "bootstrap-pack-render-smoke",
            "render bootstrap pack smoke fixtures",
            [
                "bash",
                "-lc",
                "tmp=$(mktemp -d); python3 tools/bootstrap/render_bootstrap_pack.py --pack codex-agent-repo --out \"$tmp/codex\" --repo-name codex-smoke --generated-at 2026-05-09T06:00:00Z && python3 tools/bootstrap/render_bootstrap_pack.py --pack rust-cli-agent-repo --out \"$tmp/rust\" --repo-name rust-smoke --primary-language rust --generated-at 2026-05-09T06:00:00Z",
            ],
            "docs/runbooks/validation.md#bootstrap-packs",
            ["bash", "python3", "mktemp"],
            "Failure means a bootstrap pack cannot render into a fresh local directory.",
        ),
        policy_gate(
            "hardened-codex-release-manifest",
            "validate hardened-codex release manifest",
            [
                "python3",
                "subagents/hardened-codex/scripts/sync_agents.py",
                "--validate-release-manifest",
            ],
            "docs/runbooks/validation.md#bootstrap-packs",
            ["python3"],
            "Failure means the hardened-codex release manifest is invalid.",
        ),
        policy_gate(
            "hardened-codex-global-dry-run",
            "hardened-codex global install dry-run",
            [
                "python3",
                "subagents/hardened-codex/scripts/sync_agents.py",
                "--global",
                "--all-overlays",
                "--dry-run",
            ],
            "docs/runbooks/validation.md#bootstrap-packs",
            ["python3"],
            "Failure means global subagent installation planning regressed.",
        ),
        policy_gate(
            "hardened-codex-validate-sources",
            "hardened-codex source validation",
            [
                "python3",
                "subagents/hardened-codex/scripts/sync_agents.py",
                "--global",
                "--all-overlays",
                "--validate-sources",
            ],
            "docs/runbooks/validation.md#bootstrap-packs",
            ["python3"],
            "Failure means hardened-codex source pack validation regressed.",
        ),
        policy_gate(
            "bootstrap-local-overlays-ignored",
            "prove private overlay paths stay gitignored",
            [
                "bash",
                "-lc",
                "for path in subagents/hardened-codex/overlays.local.json subagents/hardened-codex/roles.local.json subagents/hardened-codex/agents/overlays/private-repo/private_repo_reviewer.toml; do git check-ignore -q -- \"$path\" || exit 1; done",
            ],
            "docs/runbooks/validation.md#bootstrap-packs",
            ["bash", "git"],
            "Failure means private/local overlay paths may be accidentally trackable.",
        ),
    ]
}

fn docs_gates() -> Vec<PolicyGate> {
    vec![
        policy_gate(
            "docs-no-todo",
            "docs unresolved-marker check",
            [
                "bash",
                "-lc",
                "! rg -n \"TO[D]O|FIX[M]E\" docs README.md AGENTS.md",
            ],
            "docs/runbooks/validation.md#docs",
            ["bash", "rg"],
            "Failure means docs contain unresolved TODO/FIXME markers.",
        ),
        policy_docs_check_gate(),
        docs_links_gate(),
        diff_check_gate(),
    ]
}

fn release_gates() -> Vec<PolicyGate> {
    let mut gates = Vec::new();
    append_unique_gates(&mut gates, codex_dev_gates());
    append_unique_gates(&mut gates, codex_dev_tui_gates());
    append_unique_gates(&mut gates, codex_research_gates());
    append_unique_gates(&mut gates, docs_gates());
    append_unique_gates(&mut gates, vec![bootstrap_pack_validate_gate()]);
    append_unique_gates(&mut gates, skills_gates());
    gates
}

fn full_local_gates() -> Vec<PolicyGate> {
    let mut gates = Vec::new();
    append_unique_gates(&mut gates, codex_dev_gates());
    append_unique_gates(&mut gates, codex_dev_tui_gates());
    append_unique_gates(&mut gates, codex_research_gates());
    append_unique_gates(&mut gates, bootstrap_install_gates());
    append_unique_gates(&mut gates, skills_gates());
    append_unique_gates(&mut gates, docs_gates());
    gates
}

fn append_unique_gates(target: &mut Vec<PolicyGate>, gates: Vec<PolicyGate>) {
    let mut seen = target
        .iter()
        .map(|gate| gate.id.clone())
        .collect::<BTreeSet<_>>();
    for gate in gates {
        if seen.insert(gate.id.clone()) {
            target.push(gate);
        }
    }
}

fn cargo_fmt_gate() -> PolicyGate {
    policy_gate(
        "cargo-fmt",
        "Rust workspace formatting",
        ["cargo", "fmt", "--all", "--check"],
        "docs/runbooks/validation.md#full-local-gate",
        ["cargo"],
        "Failure means Rust formatting drift; run cargo fmt --all and review the diff.",
    )
}

fn docs_links_gate() -> PolicyGate {
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
        "docs/runbooks/validation.md#docs",
        ["python3"],
        "Failure means tracked docs contain broken local links or stale anchors.",
    )
}

fn bootstrap_pack_validate_gate() -> PolicyGate {
    policy_gate(
        "bootstrap-pack-validate",
        "validate bootstrap pack manifests",
        [
            "python3",
            "tools/bootstrap/render_bootstrap_pack.py",
            "--validate",
        ],
        "docs/runbooks/validation.md#bootstrap-packs",
        ["python3"],
        "Failure means bootstrap pack manifests or templates are invalid.",
    )
}

fn policy_docs_check_gate() -> PolicyGate {
    policy_gate(
        "codex-dev-policy-docs-check",
        "codex-dev policy docs drift check",
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "codex-dev",
            "--",
            "--json",
            "policy",
            "docs-check",
        ],
        "docs/runbooks/validation.md#validation-matrix-ownership",
        ["cargo"],
        "Failure means machine-owned policy manifest command mirrors in docs drifted from the Rust policy profile owner.",
    )
}

fn diff_check_gate() -> PolicyGate {
    policy_gate(
        "diff-check",
        "git whitespace check",
        ["git", "diff", "--check"],
        "docs/runbooks/validation.md#full-local-gate",
        ["git"],
        "Failure means the working diff has whitespace or conflict-marker problems.",
    )
}

fn render_pr_record_command(args: &PrRecordCliArgs, checked_at: DateTime<Utc>) -> String {
    let mut command = vec![
        "codex-dev".to_string(),
        "pr".to_string(),
        "record".to_string(),
        "--capsule".to_string(),
        args.capsule.display().to_string(),
        "--source".to_string(),
        args.source.display().to_string(),
        "--source-kind".to_string(),
        args.source_kind.to_string(),
    ];
    if let Some(repo) = &args.repo {
        command.push("--repo".to_string());
        command.push(repo.clone());
    }
    if let Some(number) = args.number {
        command.push("--number".to_string());
        command.push(number.to_string());
    }
    command.extend([
        "--checked-at".to_string(),
        checked_at.to_rfc3339_opts(SecondsFormat::AutoSi, true),
    ]);
    if let Some(retrieved_at) = args.retrieved_at {
        command.push("--retrieved-at".to_string());
        command.push(retrieved_at.to_rfc3339_opts(SecondsFormat::AutoSi, true));
    }
    if let Some(source_command) = &args.source_command {
        command.push("--source-command".to_string());
        command.push(source_command.clone());
    }
    render_command(&command)
}

fn policy_gate<const N: usize, const M: usize>(
    id: &str,
    name: &str,
    command: [&str; N],
    source: &str,
    required_tools: [&str; M],
    failure_interpretation: &str,
) -> PolicyGate {
    PolicyGate {
        id: id.to_string(),
        name: name.to_string(),
        command: command.iter().map(|part| (*part).to_string()).collect(),
        source: source.to_string(),
        working_directory: ".".to_string(),
        required_tools: required_tools
            .iter()
            .map(|tool| (*tool).to_string())
            .collect(),
        required: true,
        network: false,
        secrets: false,
        failure_interpretation: failure_interpretation.to_string(),
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
    let current_dir = env::current_dir().context("failed to read current directory")?;
    resolve_repo_root_from(capsule_path, explicit, &current_dir)
}

fn resolve_repo_root_from(
    capsule_path: &Path,
    explicit: Option<&Path>,
    current_dir: &Path,
) -> Result<PathBuf> {
    if let Some(root) = explicit {
        return canonicalize_repo_root(root);
    }

    let current_root = find_repo_root(current_dir);

    let capsule_path =
        fs::canonicalize(capsule_path).unwrap_or_else(|_| capsule_path.to_path_buf());
    let capsule_root = capsule_path.parent().and_then(find_repo_root);
    match (capsule_root, current_root) {
        (Some(capsule_root), Some(current_root)) if capsule_root != current_root => {
            bail!(
                "capsule path belongs to repo root {} but current directory is under {}; pass --repo-root to choose explicitly",
                capsule_root.display(),
                current_root.display()
            );
        }
        (Some(capsule_root), _) => return Ok(capsule_root),
        (None, Some(current_root)) => return Ok(current_root),
        (None, None) => {}
    }

    bail!(
        "failed to discover repository root from current directory or capsule path; run from the repo or pass --repo-root"
    );
}

fn gate_working_directory(repo_root: &Path, gate: &PolicyGate) -> Result<PathBuf> {
    if gate.working_directory.trim().is_empty() {
        bail!(
            "gate {} has empty working_directory {:?}",
            gate.id,
            gate.working_directory
        );
    }
    let relative = Path::new(&gate.working_directory);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        bail!(
            "gate {} has unsafe working_directory {:?}",
            gate.id,
            gate.working_directory
        );
    }
    Ok(repo_root.join(relative))
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

fn resolve_policy_docs_repo_root(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(root) = explicit {
        return canonicalize_repo_root(root);
    }

    let current_dir = env::current_dir().context("failed to read current directory")?;
    find_repo_root(&current_dir).ok_or_else(|| {
        anyhow::anyhow!(
            "failed to discover repository root from current directory; run from the repo or pass --repo-root"
        )
    })
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

fn gate_skip_reason(
    gate: &PolicyGate,
    allow_network: bool,
    allow_secrets: bool,
) -> Option<&'static str> {
    if gate.network && !allow_network {
        Some("gate requires network and --allow-network was not set")
    } else if gate.secrets && !allow_secrets {
        Some("gate requires secrets and --allow-secrets was not set")
    } else {
        None
    }
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
        match gate_working_directory(repo_root, gate) {
            Ok(working_directory) => {
                command.current_dir(working_directory);
            }
            Err(error) => {
                return gate_result(
                    gate,
                    GateStatus::Failed,
                    None,
                    Some(error.to_string()),
                    None,
                    None,
                );
            }
        }
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
    manifest: &PolicyManifest,
    results: &[PolicyGateResult],
    checked_at: DateTime<Utc>,
) -> Result<()> {
    ensure_regular_contract_files(capsule_path)?;
    write_json(capsule_path.join("policy.json"), manifest)?;

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
                allow_secrets: false,
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
    fn policy_run_persists_selected_profile_manifest() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        let result = run_policy_gates(
            PolicyRunArgs {
                capsule: capsule.clone(),
                repo_root: None,
                profile: PolicyProfile::FullLocal,
                execute: false,
                allow_network: false,
                allow_secrets: false,
                keep_going: false,
                checked_at: None,
            },
            "2026-05-09T05:00:00Z".parse().unwrap(),
        )
        .expect("policy full local dry run");

        let persisted: PolicyManifest = read_json(&capsule.join("policy.json")).expect("policy");
        assert_eq!(persisted.profile, PolicyProfile::FullLocal);
        assert_eq!(persisted.gates.len(), result.gates.len());
        assert_eq!(
            persisted.gates.last().map(|gate| gate.id.as_str()),
            result.gates.last().map(|gate| gate.id.as_str())
        );
    }

    #[test]
    fn policy_docs_check_passes_current_repo_docs() {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = crate_dir
            .parent()
            .and_then(Path::parent)
            .expect("repo root");

        let result = policy_docs_check(Some(repo_root)).expect("docs check");

        assert!(result.passed, "{:#?}", result.blocks);
    }

    #[test]
    fn policy_docs_check_reports_stale_doc_block() {
        let temp = tempdir().expect("tempdir");
        let smoke_commands = vec![policy_manifest_command(PolicyProfile::CodexDev)];
        let all_commands = all_policy_profiles()
            .iter()
            .map(|profile| policy_manifest_command(*profile))
            .collect::<Vec<_>>();
        write_policy_docs_fixture(temp.path(), &smoke_commands, &all_commands);

        let result = policy_docs_check(Some(temp.path())).expect("docs check");

        assert!(!result.passed);
        let agent_block = result
            .blocks
            .iter()
            .find(|block| block.path == "AGENTS.md")
            .expect("AGENTS block");
        assert_eq!(agent_block.actual_commands, smoke_commands);
        assert!(
            agent_block
                .expected_commands
                .iter()
                .any(|command| command.contains("--profile full_local"))
        );
    }

    #[test]
    fn policy_docs_extractor_ignores_marker_tokens_in_prose() {
        let command = policy_manifest_command(PolicyProfile::CodexDev);
        let contents = format!(
            "Prose can mention codex-dev:policy-manifest-smoke:start without opening a block.\n\
             ## codex-dev:policy-manifest-smoke:start\n\
             # codex-dev:policy-manifest-smoke:start\n\
             ```bash\n\
             {command}\n\
             ```\n\
             # codex-dev:policy-manifest-smoke:end\n\
             ## codex-dev:policy-manifest-smoke:end\n\
             Prose can mention codex-dev:policy-manifest-smoke:end too.\n"
        );

        let commands =
            extract_policy_doc_commands(&contents, POLICY_DOCS_SMOKE_MARKER).expect("commands");

        assert_eq!(commands, vec![command]);
    }

    #[test]
    fn policy_manifest_profiles_are_explicit_local_gates() {
        for profile in all_policy_profiles() {
            let manifest = policy_manifest(profile, "2026-05-09T05:00:00Z".parse().unwrap());

            assert_eq!(manifest.profile, profile);
            assert!(!manifest.gates.is_empty(), "{profile} should have gates");
            for gate in &manifest.gates {
                assert!(!gate.id.is_empty(), "{profile} has an empty gate id");
                assert!(
                    !gate.command.is_empty(),
                    "{profile} gate {} has no command",
                    gate.id
                );
                assert!(
                    !gate.source.is_empty(),
                    "{profile} gate {} has no source",
                    gate.id
                );
                assert_eq!(gate.working_directory, ".", "{profile} gate {}", gate.id);
                assert!(
                    !gate.required_tools.is_empty(),
                    "{profile} gate {} has no required_tools",
                    gate.id
                );
                assert!(
                    !gate.failure_interpretation.is_empty(),
                    "{profile} gate {} has no failure_interpretation",
                    gate.id
                );
                assert!(
                    !gate.network && !gate.secrets,
                    "{profile} gate {} unexpectedly requires network or secrets",
                    gate.id
                );
            }
        }
    }

    #[test]
    fn policy_manifest_profile_gate_ids_are_stable() {
        assert_profile_ids(
            PolicyProfile::CodexDev,
            &[
                "cargo-fmt",
                "codex-dev-core-clippy",
                "codex-dev-clippy",
                "codex-dev-core-check",
                "codex-dev-check",
                "codex-dev-core-test",
                "codex-dev-test",
                "codex-dev-help",
                "codex-dev-policy-manifest",
                "codex-dev-policy-docs-check",
                "codex-dev-pr-plan-smoke",
                "docs-links",
                "diff-check",
            ],
        );
        assert_profile_ids(
            PolicyProfile::CodexDevTui,
            &[
                "cargo-fmt",
                "codex-dev-tui-clippy",
                "codex-dev-tui-check",
                "codex-dev-tui-test",
                "codex-dev-tui-help",
            ],
        );
        assert_profile_ids(
            PolicyProfile::CodexResearch,
            &[
                "cargo-fmt",
                "codex-research-clippy",
                "codex-research-check",
                "codex-research-test",
                "codex-research-doctor",
                "codex-research-eval",
                "codex-research-eval-list",
                "codex-research-eval-strict",
                "codex-research-plan-quick",
            ],
        );
        assert_profile_ids(
            PolicyProfile::Skills,
            &[
                "skills-quick-validate-all",
                "python-helpers-compile",
                "subagent-templates-validate",
                "subspawn-roles-validate",
                "subspawn-plan-research-smoke",
                "skill-subagent-eval",
            ],
        );
        assert_profile_ids(
            PolicyProfile::BootstrapInstall,
            &[
                "bootstrap-pack-validate",
                "bootstrap-pack-render-smoke",
                "hardened-codex-release-manifest",
                "hardened-codex-global-dry-run",
                "hardened-codex-validate-sources",
                "bootstrap-local-overlays-ignored",
            ],
        );
        assert_profile_ids(
            PolicyProfile::Docs,
            &[
                "docs-no-todo",
                "codex-dev-policy-docs-check",
                "docs-links",
                "diff-check",
            ],
        );
        assert_profile_ids(
            PolicyProfile::Release,
            &[
                "cargo-fmt",
                "codex-dev-core-clippy",
                "codex-dev-clippy",
                "codex-dev-core-check",
                "codex-dev-check",
                "codex-dev-core-test",
                "codex-dev-test",
                "codex-dev-help",
                "codex-dev-policy-manifest",
                "codex-dev-policy-docs-check",
                "codex-dev-pr-plan-smoke",
                "docs-links",
                "diff-check",
                "codex-dev-tui-clippy",
                "codex-dev-tui-check",
                "codex-dev-tui-test",
                "codex-dev-tui-help",
                "codex-research-clippy",
                "codex-research-check",
                "codex-research-test",
                "codex-research-doctor",
                "codex-research-eval",
                "codex-research-eval-list",
                "codex-research-eval-strict",
                "codex-research-plan-quick",
                "docs-no-todo",
                "bootstrap-pack-validate",
                "skills-quick-validate-all",
                "python-helpers-compile",
                "subagent-templates-validate",
                "subspawn-roles-validate",
                "subspawn-plan-research-smoke",
                "skill-subagent-eval",
            ],
        );
        assert_profile_ids(
            PolicyProfile::FullLocal,
            &[
                "cargo-fmt",
                "codex-dev-core-clippy",
                "codex-dev-clippy",
                "codex-dev-core-check",
                "codex-dev-check",
                "codex-dev-core-test",
                "codex-dev-test",
                "codex-dev-help",
                "codex-dev-policy-manifest",
                "codex-dev-policy-docs-check",
                "codex-dev-pr-plan-smoke",
                "docs-links",
                "diff-check",
                "codex-dev-tui-clippy",
                "codex-dev-tui-check",
                "codex-dev-tui-test",
                "codex-dev-tui-help",
                "codex-research-clippy",
                "codex-research-check",
                "codex-research-test",
                "codex-research-doctor",
                "codex-research-eval",
                "codex-research-eval-list",
                "codex-research-eval-strict",
                "codex-research-plan-quick",
                "bootstrap-pack-validate",
                "bootstrap-pack-render-smoke",
                "hardened-codex-release-manifest",
                "hardened-codex-global-dry-run",
                "hardened-codex-validate-sources",
                "bootstrap-local-overlays-ignored",
                "skills-quick-validate-all",
                "python-helpers-compile",
                "subagent-templates-validate",
                "subspawn-roles-validate",
                "subspawn-plan-research-smoke",
                "skill-subagent-eval",
                "docs-no-todo",
            ],
        );
    }

    #[test]
    fn policy_manifest_profile_snapshots_are_stable() {
        let actual = all_policy_profiles()
            .iter()
            .map(|profile| profile_snapshot(*profile))
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(
            actual.trim(),
            include_str!("../tests/snapshots/policy_profiles.tsv").trim()
        );
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
                allow_secrets: false,
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
                allow_secrets: false,
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
            working_directory: ".".to_string(),
            required_tools: vec!["codex-dev-command-that-does-not-exist".to_string()],
            required: true,
            network: false,
            secrets: false,
            failure_interpretation: "fixture failure".to_string(),
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
            working_directory: ".".to_string(),
            required_tools: vec!["python3".to_string()],
            required: true,
            network: false,
            secrets: false,
            failure_interpretation: "fixture failure".to_string(),
        };

        let result = execute_gate(&gate, Some(temp.path()));

        assert_eq!(result.status, GateStatus::Passed);
    }

    #[test]
    fn policy_repo_root_resolution_rejects_mismatched_capsule_and_current_repos() {
        let temp = tempdir().expect("tempdir");
        let capsule_repo = temp.path().join("capsule-repo");
        let current_repo = temp.path().join("current-repo");
        write_repo_fixture(&capsule_repo);
        write_repo_fixture(&current_repo);
        let capsule = capsule_repo.join(".codex/tasks/example");
        fs::create_dir_all(&capsule).expect("capsule dir");
        let current_dir = current_repo.join("nested");
        fs::create_dir_all(&current_dir).expect("current dir");

        let error = resolve_repo_root_from(&capsule, None, &current_dir)
            .expect_err("mismatched roots rejected");

        assert!(error.to_string().contains("pass --repo-root"), "{error:#}");
    }

    #[test]
    fn policy_gate_skip_reason_requires_secret_opt_in() {
        let mut gate = cargo_fmt_gate();
        gate.secrets = true;

        assert_eq!(
            gate_skip_reason(&gate, false, false),
            Some("gate requires secrets and --allow-secrets was not set")
        );
        assert_eq!(gate_skip_reason(&gate, false, true), None);
    }

    #[test]
    fn policy_execution_rejects_unsafe_working_directory() {
        let temp = tempdir().expect("tempdir");
        let mut gate = cargo_fmt_gate();
        gate.working_directory = "../outside".to_string();

        let result = execute_gate(&gate, Some(temp.path()));

        assert_eq!(result.status, GateStatus::Failed);
        assert!(
            result
                .error
                .as_deref()
                .expect("error")
                .contains("unsafe working_directory")
        );
    }

    #[test]
    fn policy_execution_rejects_blank_working_directory() {
        let temp = tempdir().expect("tempdir");
        let mut gate = cargo_fmt_gate();
        gate.working_directory = " ".to_string();

        let result = execute_gate(&gate, Some(temp.path()));

        assert_eq!(result.status, GateStatus::Failed);
        assert!(
            result
                .error
                .as_deref()
                .expect("error")
                .contains("empty working_directory")
        );
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
            working_directory: ".".to_string(),
            required_tools: vec!["python3".to_string()],
            required: true,
            network: false,
            secrets: false,
            failure_interpretation: "fixture failure".to_string(),
        };

        let result = execute_gate(&gate, None);

        assert_eq!(result.status, GateStatus::Failed);
        assert_eq!(result.exit_code, Some(9));
        assert_eq!(result.stderr.as_deref(), Some("boom"));
    }

    fn write_repo_fixture(root: &Path) {
        fs::create_dir_all(root.join("docs/runbooks")).expect("docs dir");
        fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("cargo toml");
        fs::write(root.join("docs/runbooks/validation.md"), "# Validation\n")
            .expect("validation doc");
    }

    fn write_policy_docs_fixture(root: &Path, smoke_commands: &[String], all_commands: &[String]) {
        fs::create_dir_all(root.join("docs/reference")).expect("reference dir");
        fs::create_dir_all(root.join("docs/runbooks")).expect("runbooks dir");
        fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("cargo toml");
        fs::write(
            root.join("AGENTS.md"),
            policy_doc_fixture_block(POLICY_DOCS_SMOKE_MARKER, smoke_commands),
        )
        .expect("agents doc");
        fs::write(
            root.join("README.md"),
            policy_doc_fixture_block(POLICY_DOCS_SMOKE_MARKER, smoke_commands),
        )
        .expect("readme");
        fs::write(
            root.join("docs/reference/codex-dev-cli.md"),
            policy_doc_fixture_block(POLICY_DOCS_SMOKE_MARKER, smoke_commands),
        )
        .expect("cli reference");
        fs::write(
            root.join("docs/runbooks/validation.md"),
            format!(
                "{}\n{}",
                policy_doc_fixture_block(POLICY_DOCS_SMOKE_MARKER, smoke_commands),
                policy_doc_fixture_block(POLICY_DOCS_ALL_MARKER, all_commands)
            ),
        )
        .expect("validation doc");
    }

    fn policy_doc_fixture_block(marker: &str, commands: &[String]) -> String {
        format!(
            "{}\n```bash\n{}\n```\n{}\n",
            policy_doc_marker(marker, "start"),
            commands.join("\n"),
            policy_doc_marker(marker, "end")
        )
    }

    fn assert_profile_ids(profile: PolicyProfile, expected: &[&str]) {
        let manifest = policy_manifest(profile, "2026-05-09T05:00:00Z".parse().unwrap());
        let ids = manifest
            .gates
            .iter()
            .map(|gate| gate.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, expected, "{profile} profile gate ids drifted");
    }

    fn profile_snapshot(profile: PolicyProfile) -> String {
        let manifest = policy_manifest(profile, "2026-05-09T05:00:00Z".parse().unwrap());
        let mut lines = vec![format!("== {profile} ==")];
        lines.extend(manifest.gates.iter().map(|gate| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                gate.id,
                serde_json::to_string(&gate.command).expect("command json"),
                gate.source,
                gate.working_directory,
                serde_json::to_string(&gate.required_tools).expect("tools json"),
                gate.required,
                gate.network,
                gate.secrets,
                gate.failure_interpretation
            )
        }));
        lines.join("\n")
    }
}
