use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

mod bun;

use anyhow::{Context, Result, bail};
use bun::{
    BunCommand, ToolCommand, bun_command_name, handle_bun_command, handle_tool_command,
    tool_command_name,
};
use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use codex_dev_core::{
    AgentSkillsCatalogArgs, AppendEvidenceArgs, Capsule, CapsuleStatus, EVIDENCE_SCHEMA,
    EvidenceKind, EvidenceKindSummary, EvidenceRecord, EvidenceSummary, GateRecord, GateStatus,
    InitArgs, KimiSyncArgs, KimiSyncReport, KimiSyncScope, OUTPUT_SCHEMA,
    OrchestrationDiagnosticSeverity, OrchestrationRunReport, POLICY_GATES_SCHEMA,
    PR_AGENT_HOSTED_ACTION_SCHEMA, PR_AGENT_READINESS_SCHEMA, PR_AGENT_STATE_SCHEMA,
    PR_CONTROL_PLAN_SCHEMA, PolicyGate, PolicyGateResult, PolicyManifest, PolicyProfile,
    PolicyRunResult, PrAgentDiagnostic, PrAgentHostedActionExecution, PrAgentHostedActionReport,
    PrAgentHostedActionSpec, PrAgentHostedActionStatus, PrAgentReadinessAction,
    PrAgentReadinessActionStatus, PrAgentReadinessAttempt, PrAgentReadinessCheck,
    PrAgentReadinessReport, PrAgentReadinessStatus, PrAgentSeverity, PrAgentSourceRecord,
    PrAgentSourceStatus, PrAgentStateReport, PrControlCommand, PrControlPlan, PrEvidence,
    PrRecordArgs, PrRecordSourceKind, RecordSubagentOutcomeArgs, RecordSubagentPlanArgs,
    RecordSubagentSynthesisArgs, SubagentDisposition, SubagentOutcomeStatus,
    SubagentSynthesisStatus, SubagentWaitStatus, TaskRootStatus, Verification,
    agent_skills_catalog, append_evidence, append_jsonl, capsule_status,
    ensure_regular_contract_files, init_capsule, kimi_sync, orchestration_run, pr_status,
    read_json, recommend_pr_agent_actions, record_pr_snapshot, record_subagent_outcome,
    record_subagent_plan, record_subagent_synthesis, render_capsule, render_command,
    render_pr_label, render_pr_status, stable_json_hash, task_export, task_index, task_show,
    validate_capsule, write_json,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

mod review_workflow;
use review_workflow::{
    CommitCommand, PrReviewCommand, ReviewCommand, handle_commit_command, handle_pr_review_command,
    handle_review_command,
};

const POLICY_DOCS_CHECK_SCHEMA: &str = "codex-dev.policy-docs-check.v1";
const POLICY_EXPLAIN_SCHEMA: &str = "policy_explain.v1";
const RESEARCH_EVIDENCE_IMPORT_SCHEMA: &str = "research_evidence_import.v1";
const CODEX_RESEARCH_EVIDENCE_BUNDLE_SCHEMA: &str = "codex-research.evidence-bundle.v1";
const BOOTSTRAP_STATUS_SCHEMA: &str = "bootstrap_status.v1";
const BOOTSTRAP_PLAN_SCHEMA: &str = "bootstrap_plan.v1";
const BOOTSTRAP_PACK_SCHEMA: &str = "dev-skills.bootstrap-pack.v1";
const RESEARCH_IMPORT_MAX_TEXT_CHARS: usize = 512;
const RESEARCH_IMPORT_MAX_RESIDUAL_RISK_CHARS: usize = 1000;
const RESEARCH_IMPORT_MAX_LIST_ITEMS: usize = 20;
const RESEARCH_IMPORT_MAX_UNKNOWN_SOURCE_IDS: usize = 50;
const RESEARCH_IMPORT_MAX_SOURCE_IDS: usize = 100;
const RESEARCH_IMPORT_MAX_CLAIM_IDS: usize = 100;
const RESEARCH_IMPORT_MAX_ARTIFACTS: usize = 50;
const RESEARCH_IMPORT_MAX_BUDGET_PROVIDERS: usize = 32;
const RESEARCH_IMPORT_MAX_FRESHNESS_STATUSES: usize = 32;
const LOCAL_DOCTOR_SCHEMA: &str = "codex-dev.local-doctor.v1";
const POLICY_DOCS_SMOKE_MARKER: &str = "policy-manifest-smoke";
const POLICY_DOCS_ALL_MARKER: &str = "policy-manifest-all";
const ORCHESTRATION_STALE_AFTER_MINUTES: u64 = 120;
const GITHUB_TOKEN_ENV_VARS: &[&str] = &[
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "GH_ENTERPRISE_TOKEN",
    "GITHUB_ENTERPRISE_TOKEN",
];
const GH_PR_VIEW_JSON_FIELDS: &str = "number,url,state,isDraft,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup,headRefOid,headRefName,baseRefName,baseRefOid,updatedAt,labels";
// Per-thread comment pagination is intentionally not expanded here; worklist and
// closeout paths fail closed when comments.pageInfo.hasNextPage is true.
const PR_REVIEW_THREADS_QUERY: &str = "query($owner:String!,$name:String!,$number:Int!,$endCursor:String){repository(owner:$owner,name:$name){pullRequest(number:$number){reviewThreads(first:100,after:$endCursor){pageInfo{hasNextPage endCursor} nodes{id isResolved isOutdated comments(first:100){totalCount pageInfo{hasNextPage endCursor} nodes{id author{login} path line startLine originalLine originalStartLine body diffHunk url}}}}}}}";
const RESOLVE_REVIEW_THREAD_MUTATION: &str = "mutation($threadId:ID!){resolveReviewThread(input:{threadId:$threadId}){thread{id isResolved}}}";
const UNRESOLVE_REVIEW_THREAD_MUTATION: &str = "mutation($threadId:ID!){unresolveReviewThread(input:{threadId:$threadId}){thread{id isResolved}}}";

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
            Commands::Research { command } => match command {
                ResearchCommand::ImportBundle(_) => "research import-bundle",
            },
            Commands::Bun { command } => bun_command_name(command),
            Commands::Tool { command } => tool_command_name(command),
            Commands::Policy { command } => match command {
                PolicyCommand::Manifest(_) => "policy manifest",
                PolicyCommand::Explain(_) => "policy explain",
                PolicyCommand::DocsCheck(_) => "policy docs-check",
                PolicyCommand::Run(_) => "policy run",
            },
            Commands::Local { command } => match command {
                LocalCommand::Doctor(_) => "local doctor",
                LocalCommand::Status(_) => "local status",
            },
            Commands::Skills { command } => match command {
                SkillsCommand::Catalog(_) => "skills catalog",
                SkillsCommand::Inventory(_) => "skills inventory",
                SkillsCommand::Validate(_) => "skills validate",
                SkillsCommand::Audit(_) => "skills audit",
                SkillsCommand::SyncKimi(_) => "skills sync-kimi",
            },
            Commands::Bootstrap { command } => match command {
                BootstrapCommand::Status(_) => "bootstrap status",
                BootstrapCommand::Plan(_) => "bootstrap plan",
            },
            Commands::Task { command } => match command {
                TaskCommand::List(_) => "task list",
                TaskCommand::Show(_) => "task show",
                TaskCommand::Export(_) => "task export",
            },
            Commands::Pr { command } => match command {
                PrCommand::Agent(_) => "pr agent",
                PrCommand::AgentAction(_) => "pr agent-action",
                PrCommand::Plan(_) => "pr plan",
                PrCommand::Readiness(_) => "pr readiness",
                PrCommand::Record(_) => "pr record",
                PrCommand::Review { command } => match command {
                    PrReviewCommand::Start(_) => "pr review start",
                    PrReviewCommand::Refresh(_) => "pr review refresh",
                    PrReviewCommand::Query(_) => "pr review query",
                    PrReviewCommand::Render(_) => "pr review render",
                    PrReviewCommand::ApplySuggestions(_) => "pr review apply-suggestions",
                    PrReviewCommand::Closeout(_) => "pr review closeout",
                },
                PrCommand::Status(_) => "pr status",
            },
            Commands::Review { command } => match command {
                ReviewCommand::Ingest(_) => "review ingest",
                ReviewCommand::Render(_) => "review render",
                ReviewCommand::Query(_) => "review query",
            },
            Commands::Commit { command } => match command {
                CommitCommand::Plan(_) => "commit plan",
                CommitCommand::Validate(_) => "commit validate",
            },
            Commands::Subagents { command } => match command {
                SubagentsCommand::Plan(_) => "subagents record-plan",
                SubagentsCommand::Outcome(_) => "subagents record-outcome",
                SubagentsCommand::Synthesis(_) => "subagents record-synthesis",
            },
            Commands::Orchestration { command } => match command {
                OrchestrationCommand::Plan(_) => "orchestration plan",
                OrchestrationCommand::Record(_) => "orchestration record",
                OrchestrationCommand::Close(_) => "orchestration close",
                OrchestrationCommand::Verify(_) => "orchestration verify",
            },
            Commands::Completions(_) => "completions",
            Commands::Manpage => "manpage",
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
    /// Import sanitized codex-research metadata into task capsules.
    Research {
        #[command(subcommand)]
        command: ResearchCommand,
    },
    /// Audit and maintain Bun-first repositories and bun-dev skill references.
    Bun {
        #[command(subcommand)]
        command: BunCommand,
    },
    /// Import reports from external tools into task capsules.
    Tool {
        #[command(subcommand)]
        command: ToolCommand,
    },
    /// Plan or run repo-native validation policy gates.
    Policy {
        #[command(subcommand)]
        command: PolicyCommand,
    },
    /// Inspect local workstation readiness without mutating state.
    Local {
        #[command(subcommand)]
        command: LocalCommand,
    },
    /// Inspect tracked skill metadata and packaging readiness.
    Skills {
        #[command(subcommand)]
        command: SkillsCommand,
    },
    /// Inspect repo bootstrap packs without rendering files.
    Bootstrap {
        #[command(subcommand)]
        command: BootstrapCommand,
    },
    /// Read local task capsules from a task root.
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    /// Capture hosted PR evidence into task capsules.
    Pr {
        #[command(subcommand)]
        command: PrCommand,
    },
    /// Ingest and query local review-note worklists.
    Review {
        #[command(subcommand)]
        command: ReviewCommand,
    },
    /// Plan and validate scoped semantic Conventional Commit groups.
    Commit {
        #[command(subcommand)]
        command: CommitCommand,
    },
    /// Record subspawn plans, outcomes, and synthesis into task capsules.
    Subagents {
        #[command(subcommand)]
        command: SubagentsCommand,
    },
    /// Record and verify subspawn orchestration runs without spawning agents.
    Orchestration {
        #[command(subcommand)]
        command: OrchestrationCommand,
    },
    /// Generate shell completions for local installation.
    Completions(CompletionArgs),
    /// Generate a roff manpage for local installation.
    Manpage,
}

#[derive(Args, Debug)]
pub struct CompletionArgs {
    #[arg(value_enum)]
    shell: Shell,
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
enum ResearchCommand {
    /// Import a codex-research evidence bundle summary into evidence.jsonl.
    #[command(name = "import-bundle")]
    ImportBundle(ResearchImportBundleArgs),
}

#[derive(Subcommand, Debug)]
enum PolicyCommand {
    /// Print a machine-readable gate manifest.
    Manifest(PolicyManifestArgs),
    /// Explain a policy profile without executing gates.
    Explain(PolicyExplainArgs),
    /// Check machine-owned documentation mirrors for policy manifest commands.
    #[command(name = "docs-check")]
    DocsCheck(PolicyDocsCheckArgs),
    /// Plan or execute gates and record capsule evidence.
    Run(PolicyRunArgs),
}

#[derive(Subcommand, Debug)]
enum LocalCommand {
    /// Run a read-only local preflight for this repository.
    Doctor(LocalDoctorArgs),
    /// Print a compact read-only local readiness status.
    Status(LocalDoctorArgs),
}

#[derive(Subcommand, Debug)]
enum SkillsCommand {
    /// Emit the public Agent Skills Lab catalog artifact.
    Catalog(SkillsCatalogArgs),
    /// Emit a read-only machine-readable inventory of tracked skills.
    Inventory(SkillsInventoryArgs),
    /// Validate skill frontmatter and entrypoint contracts.
    Validate(SkillsValidateArgs),
    /// Audit skills for validation, metadata, stale references, and generated artifacts.
    Audit(SkillsAuditArgs),
    /// Sync Kimi Code skill loading to the Codex enabled skill set.
    #[command(name = "sync-kimi")]
    SyncKimi(SkillsSyncKimiArgs),
}

#[derive(Subcommand, Debug)]
enum BootstrapCommand {
    /// Emit read-only bootstrap pack validity and policy-gate status.
    Status(BootstrapStatusArgs),
    /// Emit a read-only dry-run render plan for one bootstrap pack.
    Plan(BootstrapPlanArgs),
}

#[derive(Subcommand, Debug)]
enum TaskCommand {
    /// List immediate task capsules under a task root.
    List(TaskRootArgs),
    /// Show one task capsule by ID or path.
    Show(TaskSelectorArgs),
    /// Export one task capsule with all local contract payloads.
    Export(TaskSelectorArgs),
}

#[derive(Subcommand, Debug)]
enum PrCommand {
    /// Gather live PR state into the capsule and recommend next dry-run actions.
    Agent(PrAgentArgs),
    /// Plan or apply one explicit hosted PR action.
    #[command(name = "agent-action")]
    AgentAction(PrAgentActionArgs),
    /// Evaluate CI, review, and merge readiness with a bounded closeout loop.
    Readiness(PrReadinessArgs),
    /// Print the live-command plan for PR evidence capture.
    Plan(PrPlanArgs),
    /// Normalize and record a PR evidence source into a task capsule.
    Record(PrRecordCliArgs),
    /// Capture, query, patch, and close hosted PR review work.
    Review {
        #[command(subcommand)]
        command: PrReviewCommand,
    },
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

#[derive(Subcommand, Debug)]
enum OrchestrationCommand {
    /// Record a planned subspawn batch and emit orchestration_run.v1.
    Plan(SubagentsRecordPlanArgs),
    /// Record one planned agent outcome and emit orchestration_run.v1.
    Record(SubagentsRecordOutcomeArgs),
    /// Record parent synthesis for a batch and emit orchestration_run.v1.
    Close(SubagentsRecordSynthesisArgs),
    /// Verify completion coverage for a recorded orchestration run.
    Verify(OrchestrationVerifyArgs),
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
pub struct PrAgentArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
    #[arg(long, value_name = "OWNER/REPO")]
    pub repo: String,
    #[arg(long, value_name = "PR_NUMBER")]
    pub number: u64,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
    #[arg(
        long,
        value_name = "SOURCE_DIR",
        help = "Replay captured source JSON files instead of running gh"
    )]
    pub source_dir: Option<PathBuf>,
}

#[derive(Args, Clone, Debug)]
pub struct PrAgentActionArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
    #[arg(long, value_name = "OWNER/REPO")]
    pub repo: String,
    #[arg(long, value_name = "PR_NUMBER")]
    pub number: u64,
    #[arg(long, value_name = "PLAN_ID")]
    pub plan_id: String,
    #[arg(long, value_enum, value_name = "ACTION")]
    pub action: PrAgentHostedActionKind,
    #[arg(
        long,
        help = "Execute the hosted mutation after writing the dry-run plan"
    )]
    pub apply: bool,
    #[arg(long, value_name = "TEXT")]
    pub body: Option<String>,
    #[arg(long, value_name = "MARKDOWN_FILE")]
    pub body_file: Option<PathBuf>,
    #[arg(long, value_name = "COMMENT_ID")]
    pub review_comment_id: Option<u64>,
    #[arg(long, value_name = "THREAD_ID")]
    pub thread_id: Option<String>,
    #[arg(long = "label", value_name = "LABEL")]
    pub labels: Vec<String>,
    #[arg(long, value_name = "RUN_ID")]
    pub run_id: Option<u64>,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
    #[arg(
        long,
        value_name = "SOURCE_DIR",
        help = "Replay captured source JSON files for dry-run planning; rejected with --apply"
    )]
    pub source_dir: Option<PathBuf>,
}

#[derive(Args, Clone, Debug)]
pub struct PrReadinessArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
    #[arg(long, value_name = "OWNER/REPO")]
    pub repo: String,
    #[arg(long, value_name = "PR_NUMBER")]
    pub number: u64,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
    #[arg(
        long,
        value_name = "SOURCE_DIR",
        help = "Replay captured source JSON files instead of running gh; rejected with --apply"
    )]
    pub source_dir: Option<PathBuf>,
    #[arg(long, default_value_t = 1, value_name = "COUNT")]
    pub poll_attempts: u64,
    #[arg(long, default_value_t = 60, value_name = "SECONDS")]
    pub poll_interval_seconds: u64,
    #[arg(long, help = "Allow requested hosted actions to execute")]
    pub apply: bool,
    #[arg(long, help = "Plan or apply reruns for failed GitHub Actions runs")]
    pub rerun_failed: bool,
    #[arg(long, help = "Plan or apply a PR merge when the final state is ready")]
    pub merge: bool,
    #[arg(long, value_enum, default_value_t = PrMergeMethod::Squash)]
    pub merge_method: PrMergeMethod,
    #[arg(long, help = "Delete the PR head branch after an applied merge")]
    pub delete_branch: bool,
    #[arg(long, value_name = "TEXT")]
    pub merge_subject: Option<String>,
    #[arg(long, value_name = "TEXT")]
    pub merge_body: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum PrAgentHostedActionKind {
    PostIssueComment,
    ReplyReviewComment,
    ResolveReviewThread,
    UnresolveReviewThread,
    AddLabels,
    RemoveLabels,
    RerunFailedJobs,
}

impl PrAgentHostedActionKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::PostIssueComment => "post-issue-comment",
            Self::ReplyReviewComment => "reply-review-comment",
            Self::ResolveReviewThread => "resolve-review-thread",
            Self::UnresolveReviewThread => "unresolve-review-thread",
            Self::AddLabels => "add-labels",
            Self::RemoveLabels => "remove-labels",
            Self::RerunFailedJobs => "rerun-failed-jobs",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum PrMergeMethod {
    Merge,
    Squash,
    Rebase,
}

impl PrMergeMethod {
    fn flag(self) -> &'static str {
        match self {
            Self::Merge => "--merge",
            Self::Squash => "--squash",
            Self::Rebase => "--rebase",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Merge => "merge",
            Self::Squash => "squash",
            Self::Rebase => "rebase",
        }
    }
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
pub struct ResearchImportBundleArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
    #[arg(long, value_name = "EVIDENCE_BUNDLE_JSON")]
    pub bundle: PathBuf,
    #[arg(long = "source-command", value_name = "COMMAND")]
    pub source_command: Option<String>,
    #[arg(long = "source-exit-code", value_name = "EXIT_CODE")]
    pub source_exit_code: Option<i32>,
    #[arg(long = "imported-at", value_name = "RFC3339")]
    pub imported_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ResearchEvidenceImportReport {
    pub schema: &'static str,
    pub imported_at: DateTime<Utc>,
    pub capsule: PathBuf,
    pub evidence_path: PathBuf,
    pub bundle_path: PathBuf,
    pub bundle: ResearchEvidenceImportBundleSummary,
    pub record: EvidenceRecord,
    pub evidence: EvidenceSummary,
}

#[derive(Debug, Serialize)]
pub struct ResearchEvidenceImportBundleSummary {
    pub schema: String,
    pub generated_at: DateTime<Utc>,
    pub status: String,
    pub strict: bool,
    pub query: String,
    pub profile: String,
    pub topic: String,
    pub run_status: String,
    pub source_count: usize,
    pub claim_count: usize,
    pub cited_claims: usize,
    pub uncited_claims: usize,
    pub missing_source_refs: Vec<String>,
    pub coverage: f64,
    pub source_freshness: BTreeMap<String, usize>,
    pub unknown_source_ids: Vec<String>,
    pub report_path: String,
    pub report_exists: bool,
    pub artifact_paths: Vec<String>,
    pub budget: ResearchEvidenceImportBudgetSummary,
    pub provider_error_count: usize,
    pub warning_count: usize,
    pub failure_count: usize,
    pub warnings: Vec<String>,
    pub failures: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ResearchEvidenceImportBudgetSummary {
    pub status: String,
    pub spent_total: u32,
    pub remaining_total: u32,
    pub providers: Vec<ResearchEvidenceImportBudgetProvider>,
}

#[derive(Debug, Serialize)]
pub struct ResearchEvidenceImportBudgetProvider {
    pub provider: String,
    pub budget: u32,
    pub spent: u32,
    pub remaining: u32,
}

#[derive(Debug, Deserialize)]
struct ResearchEvidenceBundleInput {
    schema: String,
    generated_at: DateTime<Utc>,
    status: String,
    strict: bool,
    run: ResearchEvidenceBundleRunInput,
    budget: ResearchEvidenceBundleBudgetInput,
    #[serde(default)]
    provider_errors: Vec<ResearchEvidenceBundleProviderErrorInput>,
    ledger: ResearchEvidenceBundleLedgerInput,
    citation_coverage: ResearchEvidenceBundleCitationCoverageInput,
    source_freshness: ResearchEvidenceBundleFreshnessInput,
    report: ResearchEvidenceBundleReportInput,
    #[serde(default)]
    artifacts: Vec<String>,
    #[serde(default)]
    warnings: Vec<String>,
    #[serde(default)]
    failures: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ResearchEvidenceBundleRunInput {
    #[serde(default)]
    query: String,
    #[serde(default)]
    profile: String,
    #[serde(default)]
    topic: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    cache_source_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ResearchEvidenceBundleBudgetInput {
    #[serde(default)]
    by_provider: Vec<ResearchEvidenceBundleBudgetProviderInput>,
}

#[derive(Debug, Deserialize)]
struct ResearchEvidenceBundleBudgetProviderInput {
    provider: String,
    budget: u32,
    spent: u32,
    remaining: u32,
}

#[derive(Debug, Deserialize)]
struct ResearchEvidenceBundleProviderErrorInput {
    provider: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ResearchEvidenceBundleLedgerInput {
    source_count: usize,
    claim_count: usize,
    #[serde(default)]
    source_ids: Vec<String>,
    #[serde(default)]
    claim_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ResearchEvidenceBundleCitationCoverageInput {
    cited_claims: usize,
    uncited_claims: usize,
    #[serde(default)]
    uncited_claim_ids: Vec<String>,
    #[serde(default)]
    missing_source_refs: Vec<String>,
    coverage: f64,
}

#[derive(Debug, Deserialize, Default)]
struct ResearchEvidenceBundleFreshnessInput {
    #[serde(default)]
    by_status: BTreeMap<String, usize>,
    #[serde(default)]
    unknown_source_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ResearchEvidenceBundleReportInput {
    #[serde(default)]
    path: String,
    exists: bool,
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
    #[arg(long = "agent-id", value_name = "AGENT_ID")]
    pub agent_id: Option<String>,
    #[arg(long, value_name = "STATUS")]
    pub status: SubagentOutcomeStatus,
    #[arg(long = "wait-status", value_name = "WAIT_STATUS")]
    pub wait_status: Option<SubagentWaitStatus>,
    #[arg(long = "wait-elapsed-ms", value_name = "MILLISECONDS")]
    pub wait_elapsed_ms: Option<u64>,
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
            agent_id: self.agent_id,
            status: self.status,
            summary: self.summary,
            wait_status: self.wait_status,
            wait_elapsed_ms: self.wait_elapsed_ms,
            disposition: self.disposition,
            human_verified: self.human_verified,
            source_ids: self.source_ids,
            artifacts: self.artifacts,
            recorded_at: self.recorded_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Args, Debug)]
pub struct OrchestrationVerifyArgs {
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: PathBuf,
    #[arg(long = "batch-id", value_name = "BATCH_ID")]
    pub batch_id: String,
    #[arg(long = "checked-at", value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
    #[arg(
        long = "stale-after-minutes",
        value_name = "MINUTES",
        default_value_t = ORCHESTRATION_STALE_AFTER_MINUTES,
        help = "Warn when incomplete orchestration evidence is older than this threshold"
    )]
    pub stale_after_minutes: u64,
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
pub struct PolicyExplainArgs {
    #[arg(
        long,
        default_value_t = PolicyProfile::CodexDev,
        help = "Policy profile: codex_dev, codex_dev_tui, codex_research, skills, bootstrap_install, docs, release, or full_local"
    )]
    pub profile: PolicyProfile,
    #[arg(
        long,
        value_name = "REPO_ROOT",
        help = "Repository root containing the checked documentation mirror"
    )]
    pub repo_root: Option<PathBuf>,
    #[arg(
        long,
        help = "Include absolute local repo and tool paths in the JSON report"
    )]
    pub include_local_paths: bool,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
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

/// Arguments shared by the local readiness subcommands.
#[derive(Args, Clone, Debug)]
pub struct LocalDoctorArgs {
    /// Repository root to inspect instead of discovering the current worktree root.
    #[arg(
        long,
        value_name = "REPO_ROOT",
        help = "Repository root to inspect; defaults to the current git worktree root when available"
    )]
    pub repo_root: Option<PathBuf>,
    /// Whether missing globally installed codex-dev binaries should fail the report.
    #[arg(
        long,
        help = "Treat missing globally installed codex-dev binaries as errors instead of warnings"
    )]
    pub strict_global_binaries: bool,
    /// Deterministic report timestamp, primarily for tests and fixture generation.
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
}

/// Arguments for the read-only skill inventory report.
#[derive(Args, Clone, Debug)]
pub struct SkillsInventoryArgs {
    /// Repository root to inspect instead of discovering the current worktree root.
    #[arg(
        long,
        value_name = "REPO_ROOT",
        help = "Repository root to inspect; defaults to the current git worktree root when available"
    )]
    pub repo_root: Option<PathBuf>,
    /// Skills directory to inspect directly, for installed global skill roots.
    #[arg(
        long,
        value_name = "SKILLS_ROOT",
        help = "Skills root to inspect directly instead of <repo-root>/skills"
    )]
    pub skills_root: Option<PathBuf>,
    /// Deterministic report timestamp, primarily for tests and fixture generation.
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
}

/// Arguments for skill validation.
#[derive(Args, Clone, Debug)]
pub struct SkillsValidateArgs {
    /// Repository root to inspect instead of discovering the current worktree root.
    #[arg(
        long,
        value_name = "REPO_ROOT",
        help = "Repository root to inspect; defaults to the current git worktree root when available"
    )]
    pub repo_root: Option<PathBuf>,
    /// Skills directory to inspect directly, for installed global skill roots.
    #[arg(
        long,
        value_name = "SKILLS_ROOT",
        help = "Skills root to inspect directly instead of <repo-root>/skills"
    )]
    pub skills_root: Option<PathBuf>,
    /// Deterministic report timestamp, primarily for tests and fixture generation.
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
}

/// Arguments for skill hygiene audits.
#[derive(Args, Clone, Debug)]
pub struct SkillsAuditArgs {
    /// Repository root to inspect instead of discovering the current worktree root.
    #[arg(
        long,
        value_name = "REPO_ROOT",
        help = "Repository root to inspect; defaults to the current git worktree root when available"
    )]
    pub repo_root: Option<PathBuf>,
    /// Skills directory to inspect directly, for installed global skill roots.
    #[arg(
        long,
        value_name = "SKILLS_ROOT",
        help = "Skills root to inspect directly instead of <repo-root>/skills"
    )]
    pub skills_root: Option<PathBuf>,
    /// Warn when SKILL.md is longer than this many lines.
    #[arg(long, default_value_t = 500, value_name = "LINES")]
    pub max_skill_md_lines: usize,
    /// Deterministic report timestamp, primarily for tests and fixture generation.
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
}

/// Arguments for syncing Kimi Code skill discovery to Codex enabled skills.
#[derive(Args, Clone, Debug)]
pub struct SkillsSyncKimiArgs {
    /// Print the planned mirror without writing it.
    #[arg(long, conflicts_with = "apply")]
    pub dry_run: bool,
    /// Write the generated Kimi mirror and any requested wrapper.
    #[arg(long)]
    pub apply: bool,
    /// Codex skill/plugin scope to mirror into Kimi.
    #[arg(long, value_enum, default_value_t = KimiSyncScopeArg::Focused)]
    pub scope: KimiSyncScopeArg,
    /// Codex home directory; defaults to ~/.codex.
    #[arg(long, value_name = "PATH")]
    pub codex_home: Option<PathBuf>,
    /// Agent skills home directory; defaults to ~/.agents.
    #[arg(long, value_name = "PATH")]
    pub agents_home: Option<PathBuf>,
    /// Kimi Code home directory; defaults to ~/.kimi-code.
    #[arg(long, value_name = "PATH")]
    pub kimi_home: Option<PathBuf>,
    /// Project root whose project-local skills should be mirrored.
    #[arg(
        long,
        value_name = "PATH",
        help = "Project root to inspect; defaults to the current git worktree root when available"
    )]
    pub project_root: Option<PathBuf>,
    /// Deterministic report timestamp, primarily for tests and fixture generation.
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
    /// Install or refresh ~/.local/bin/kimi-codex.
    #[arg(long)]
    pub install_wrapper: bool,
    /// Override the wrapper path used with --install-wrapper.
    #[arg(long, value_name = "PATH")]
    pub wrapper_path: Option<PathBuf>,
    /// Launch Kimi after applying the sync.
    #[arg(long)]
    pub launch: bool,
    /// Arguments passed through to Kimi when --launch is used.
    #[arg(last = true)]
    pub kimi_args: Vec<OsString>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum KimiSyncScopeArg {
    Focused,
    AllEnabled,
    GlobalOnly,
}

impl From<KimiSyncScopeArg> for KimiSyncScope {
    fn from(value: KimiSyncScopeArg) -> Self {
        match value {
            KimiSyncScopeArg::Focused => Self::Focused,
            KimiSyncScopeArg::AllEnabled => Self::AllEnabled,
            KimiSyncScopeArg::GlobalOnly => Self::GlobalOnly,
        }
    }
}

/// Arguments for the public Agent Skills Lab catalog artifact.
#[derive(Args, Clone, Debug)]
pub struct SkillsCatalogArgs {
    /// Repository root to inspect instead of discovering the current worktree root.
    #[arg(
        long,
        value_name = "REPO_ROOT",
        help = "Repository root to inspect; defaults to the current git worktree root when available"
    )]
    pub repo_root: Option<PathBuf>,
    /// Deterministic report timestamp, primarily for tests and fixture generation.
    #[arg(long, value_name = "RFC3339")]
    pub generated_at: Option<DateTime<Utc>>,
    /// Public source repository URL used when building per-skill source links.
    #[arg(
        long,
        default_value = "https://github.com/BjornMelin/dev-skills",
        value_name = "URL"
    )]
    pub source_repository: String,
    /// Source commit SHA used to validate catalog paths.
    #[arg(long, value_name = "SHA")]
    pub source_commit: Option<String>,
    /// Public Git ref used when building GitHub source links; defaults to source commit.
    #[arg(long, value_name = "REF")]
    pub source_ref: Option<String>,
    /// Write the raw catalog artifact to a path instead of only printing the JSON envelope.
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,
}

/// Arguments for the read-only bootstrap pack status report.
#[derive(Args, Clone, Debug)]
pub struct BootstrapStatusArgs {
    /// Repository root to inspect instead of discovering the current worktree root.
    #[arg(
        long,
        value_name = "REPO_ROOT",
        help = "Repository root to inspect; defaults to the current git worktree root when available"
    )]
    pub repo_root: Option<PathBuf>,
    /// Limit the report to one pack name under bootstrap/packs.
    #[arg(long, value_name = "PACK")]
    pub pack: Option<String>,
    /// Include absolute local paths in JSON; redacted by default.
    #[arg(long, help = "Report absolute local paths in JSON")]
    pub include_local_paths: bool,
    /// Deterministic report timestamp, primarily for tests and fixture generation.
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
}

/// Arguments for read-only bootstrap pack dry-run planning.
#[derive(Args, Clone, Debug)]
pub struct BootstrapPlanArgs {
    /// Repository root to inspect instead of discovering the current worktree root.
    #[arg(
        long,
        value_name = "REPO_ROOT",
        help = "Repository root to inspect; defaults to the current git worktree root when available"
    )]
    pub repo_root: Option<PathBuf>,
    /// Pack name under bootstrap/packs.
    #[arg(long, value_name = "PACK")]
    pub pack: String,
    /// Output directory to inspect for would-write or would-overwrite actions.
    #[arg(long, value_name = "OUTPUT_DIR")]
    pub out: PathBuf,
    /// Repository name that would be passed to the Python renderer.
    #[arg(long, default_value = "new-repo")]
    pub repo_name: String,
    /// Primary language that would be passed to the Python renderer.
    #[arg(long, default_value = "unspecified")]
    pub primary_language: String,
    /// Include absolute local output paths in JSON; redacted by default.
    #[arg(long, help = "Report absolute local output paths in JSON")]
    pub include_local_paths: bool,
    /// Deterministic report timestamp, primarily for tests and fixture generation.
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
}

/// Read-only bootstrap pack validity report.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct BootstrapStatusReport {
    pub schema: &'static str,
    pub checked_at: DateTime<Utc>,
    pub repo_root: String,
    pub pack_root: String,
    pub template_root: String,
    pub ok: bool,
    pub total: usize,
    pub valid: usize,
    pub invalid: usize,
    pub diagnostics: Vec<BootstrapDiagnostic>,
    pub packs: Vec<BootstrapPackStatus>,
    pub policy_gates: BootstrapPolicyGateSummary,
}

/// Read-only bootstrap pack render plan report.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct BootstrapPlanReport {
    pub schema: &'static str,
    pub checked_at: DateTime<Utc>,
    pub repo_root: String,
    pub pack: BootstrapPackStatus,
    pub ok: bool,
    pub dry_run: bool,
    pub output_root: String,
    pub repo_name: String,
    pub primary_language: String,
    pub target_count: usize,
    pub action_counts: BTreeMap<String, usize>,
    pub files: Vec<BootstrapPlannedFile>,
    pub advisory_host_checks: Vec<String>,
    pub diagnostics: Vec<BootstrapDiagnostic>,
}

/// Bootstrap status diagnostic.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct BootstrapDiagnostic {
    pub severity: LocalDiagnosticSeverity,
    pub code: String,
    pub message: String,
}

/// One manifest-backed bootstrap pack status.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct BootstrapPackStatus {
    pub name: String,
    pub path: String,
    pub schema: String,
    pub description: String,
    pub valid: bool,
    pub errors: Vec<String>,
    pub file_count: usize,
    pub files: Vec<BootstrapPackFileStatus>,
    pub composes: BootstrapPackComposesStatus,
    pub advisory_host_checks: Vec<String>,
}

/// Bootstrap pack composed resource metadata.
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct BootstrapPackComposesStatus {
    pub skills: Vec<String>,
    pub subagent_sources: Vec<String>,
}

/// One file entry from a bootstrap pack manifest.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct BootstrapPackFileStatus {
    pub target: String,
    pub template: String,
    pub template_exists: bool,
}

/// One planned bootstrap dry-run file action.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct BootstrapPlannedFile {
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_path: Option<PathBuf>,
    pub template: String,
    pub action: String,
}

/// Summary of the bootstrap-install policy profile backing the pack workflow.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct BootstrapPolicyGateSummary {
    pub profile: PolicyProfile,
    pub gate_count: usize,
    pub required_gate_count: usize,
    pub gate_ids: Vec<String>,
}

/// Operator intent for a local readiness report.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalReportMode {
    /// Full local preflight report.
    Doctor,
    /// Compact status-oriented local readiness report.
    Status,
}

/// Read-only workstation and checkout readiness report.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct LocalDoctorReport {
    /// Versioned schema identifier for the report payload inside the command envelope.
    pub schema: &'static str,
    /// Subcommand intent that produced this report.
    pub mode: LocalReportMode,
    /// Timestamp at which the report was generated.
    pub checked_at: DateTime<Utc>,
    /// Current working directory of the process that generated the report.
    pub cwd: PathBuf,
    /// Repository root inspected by the report.
    pub repo_root: PathBuf,
    /// True when no error-severity diagnostics were emitted.
    pub ok: bool,
    /// Human-actionable readiness findings.
    pub diagnostics: Vec<LocalDiagnostic>,
    /// Globally installed Codex binary posture.
    pub binaries: Vec<LocalToolStatus>,
    /// Required and optional development tool posture.
    pub tools: Vec<LocalToolStatus>,
    /// GitHub CLI and categorical authentication posture.
    pub github: LocalGithubStatus,
    /// Local task-capsule root state.
    pub capsule_root: LocalPathStatus,
    /// Local cache and install-smoke roots that should not become tracked artifacts.
    pub cache_roots: Vec<LocalPathStatus>,
    /// Built-in policy profile gate counts.
    pub policy_profiles: Vec<LocalPolicyProfileStatus>,
}

/// Human-actionable local readiness finding.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct LocalDiagnostic {
    /// Severity used to compute the report verdict.
    pub severity: LocalDiagnosticSeverity,
    /// Stable machine-readable diagnostic code.
    pub code: String,
    /// Human-readable remediation hint.
    pub message: String,
}

/// Severity for local readiness diagnostics.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalDiagnosticSeverity {
    /// Informational diagnostic that does not require action.
    Info,
    /// Non-blocking readiness concern.
    Warning,
    /// Blocking readiness failure.
    Error,
}

/// Availability and optional version data for a local executable.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct LocalToolStatus {
    /// Command name as searched on PATH.
    pub name: String,
    /// Whether absence of the command is an error.
    pub required: bool,
    /// Whether an executable file was found on PATH.
    pub available: bool,
    /// Resolved executable path when available.
    pub path: Option<PathBuf>,
    /// Redacted first-line version output when probed.
    pub version: Option<String>,
}

/// Categorical GitHub CLI authentication posture without credential values.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct LocalGithubStatus {
    /// Whether the `gh` executable was found on PATH.
    pub gh_available: bool,
    /// Resolved `gh` executable path when available.
    pub gh_path: Option<PathBuf>,
    /// Names of non-empty GitHub token environment variables, never their values.
    pub token_sources: Vec<String>,
    /// Whether a GitHub CLI hosts configuration file was detected.
    pub config_present: bool,
    /// Coarse authentication source class for local readiness reporting.
    pub auth_class: String,
}

/// Local path existence and git-ignore status.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct LocalPathStatus {
    /// Stable path role name used in diagnostics.
    pub name: String,
    /// Absolute path inspected by the report.
    pub path: PathBuf,
    /// Whether the path exists on disk.
    pub exists: bool,
    /// Git ignore result, or `None` when the probe could not determine it.
    pub git_ignored: Option<bool>,
}

/// Summary of built-in policy gate counts for one profile.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct LocalPolicyProfileStatus {
    /// Policy profile summarized by this row.
    pub profile: PolicyProfile,
    /// Total built-in gates in the profile.
    pub gates: usize,
    /// Gates that must pass for the profile to pass.
    pub required_gates: usize,
    /// Gates that may require network access.
    pub network_gates: usize,
    /// Gates that may require secrets or credentials.
    pub secret_gates: usize,
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

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PolicyExplainReport {
    pub schema: &'static str,
    pub profile: PolicyProfile,
    pub checked_at: DateTime<Utc>,
    pub manifest_schema: String,
    pub gate_count: usize,
    pub required_gate_count: usize,
    pub network_gate_count: usize,
    pub secret_gate_count: usize,
    pub docs_mirror: PolicyExplainDocsMirror,
    pub required_tools: Vec<PolicyExplainToolStatus>,
    pub missing_local_prerequisites: Vec<PolicyExplainMissingPrerequisite>,
    pub gates: Vec<PolicyExplainGate>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PolicyExplainDocsMirror {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_root: Option<PathBuf>,
    pub status: String,
    pub passed: bool,
    pub blocks: Vec<PolicyExplainDocsBlock>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PolicyExplainDocsBlock {
    pub path: String,
    pub marker: String,
    pub profiles: Vec<PolicyProfile>,
    pub status: String,
    pub expected_commands: Vec<String>,
    pub actual_commands: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PolicyExplainToolStatus {
    pub name: String,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PolicyExplainMissingPrerequisite {
    pub tool: String,
    pub gate_ids: Vec<String>,
    pub detail: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PolicyExplainGate {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub source: String,
    pub command: Vec<String>,
    pub command_display: String,
    pub working_directory: String,
    pub required: bool,
    pub required_tools: Vec<PolicyExplainToolStatus>,
    pub missing_required_tools: Vec<String>,
    pub network: bool,
    pub network_posture: String,
    pub secrets: bool,
    pub secrets_posture: String,
    pub docs_mirror_status: String,
    pub expected_artifacts: Vec<String>,
    pub failure_interpretation: String,
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

#[derive(Args, Debug)]
pub struct TaskRootArgs {
    #[arg(
        long,
        value_name = "TASK_ROOT",
        default_value = ".codex/tasks",
        help = "Directory containing local task capsule directories"
    )]
    root: PathBuf,
}

#[derive(Args, Debug)]
pub struct TaskSelectorArgs {
    #[arg(
        long,
        value_name = "TASK_ROOT",
        default_value = ".codex/tasks",
        help = "Directory containing local task capsule directories"
    )]
    root: PathBuf,
    #[arg(value_name = "TASK_ID_OR_DIR")]
    task: PathBuf,
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
    let json_output = cli.json;
    match cli.command {
        Commands::Completions(args) => {
            let shell = shell_name(args.shell);
            let content = render_completion(args.shell)?;
            Ok(CommandOutput {
                ok: true,
                command: "completions",
                human: content.clone(),
                result: json!({
                    "binary": "codex-dev",
                    "shell": shell,
                    "content": content,
                }),
            })
        }
        Commands::Manpage => {
            let content = render_manpage()?;
            Ok(CommandOutput {
                ok: true,
                command: "manpage",
                human: content.clone(),
                result: json!({
                    "binary": "codex-dev",
                    "section": 1,
                    "content": content,
                }),
            })
        }
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
        Commands::Research { command } => match command {
            ResearchCommand::ImportBundle(args) => {
                let result = import_research_bundle(args, Utc::now())?;
                Ok(CommandOutput {
                    ok: true,
                    command: "research import-bundle",
                    human: format!(
                        "imported {} codex-research bundle with {} source(s), {} claim(s), and {} failure(s)",
                        result.bundle.status,
                        result.bundle.source_count,
                        result.bundle.claim_count,
                        result.bundle.failure_count
                    ),
                    result: serde_json::to_value(result)?,
                })
            }
        },
        Commands::Bun { command } => handle_bun_command(command),
        Commands::Tool { command } => handle_tool_command(command),
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
        Commands::Orchestration { command } => match command {
            OrchestrationCommand::Plan(args) => {
                let core_args = args.into_core();
                let capsule = core_args.capsule.clone();
                let batch_id = core_args.batch_id.clone();
                let checked_at = core_args.recorded_at;
                record_subagent_plan(core_args)?;
                let report = orchestration_run(
                    &capsule,
                    &batch_id,
                    checked_at,
                    ORCHESTRATION_STALE_AFTER_MINUTES,
                )?;
                orchestration_output("orchestration plan", report, false)
            }
            OrchestrationCommand::Record(args) => {
                let core_args = args.into_core();
                let capsule = core_args.capsule.clone();
                let batch_id = core_args.batch_id.clone();
                let checked_at = core_args.recorded_at;
                record_subagent_outcome(core_args)?;
                let report = orchestration_run(
                    &capsule,
                    &batch_id,
                    checked_at,
                    ORCHESTRATION_STALE_AFTER_MINUTES,
                )?;
                orchestration_output("orchestration record", report, false)
            }
            OrchestrationCommand::Close(args) => {
                let core_args = args.into_core();
                let capsule = core_args.capsule.clone();
                let batch_id = core_args.batch_id.clone();
                let checked_at = core_args.recorded_at;
                record_subagent_synthesis(core_args)?;
                let report = orchestration_run(
                    &capsule,
                    &batch_id,
                    checked_at,
                    ORCHESTRATION_STALE_AFTER_MINUTES,
                )?;
                orchestration_output("orchestration close", report, false)
            }
            OrchestrationCommand::Verify(args) => {
                let checked_at = args.checked_at.unwrap_or_else(Utc::now);
                let report = orchestration_run(
                    &args.capsule,
                    &args.batch_id,
                    checked_at,
                    args.stale_after_minutes,
                )?;
                orchestration_output("orchestration verify", report, true)
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
            PolicyCommand::Explain(args) => {
                let checked_at = args.checked_at.unwrap_or_else(Utc::now);
                let result = policy_explain(args, checked_at)?;
                let missing = result.missing_local_prerequisites.len();
                let human = if missing == 0 {
                    format!(
                        "explained {} policy gate(s) for {}",
                        result.gate_count, result.profile
                    )
                } else {
                    format!(
                        "explained {} policy gate(s) for {} with {} missing prerequisite(s)",
                        result.gate_count, result.profile, missing
                    )
                };
                Ok(CommandOutput {
                    ok: true,
                    command: "policy explain",
                    human,
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
        Commands::Local { command } => match command {
            LocalCommand::Doctor(args) => {
                let result = local_doctor(args, LocalReportMode::Doctor)?;
                let human = render_local_report_human(&result);
                Ok(CommandOutput {
                    ok: result.ok,
                    command: "local doctor",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            LocalCommand::Status(args) => {
                let result = local_doctor(args, LocalReportMode::Status)?;
                let human = render_local_report_human(&result);
                Ok(CommandOutput {
                    ok: result.ok,
                    command: "local status",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
        },
        Commands::Skills { command } => match command {
            SkillsCommand::Catalog(args) => {
                let source_commit = match &args.source_commit {
                    Some(source_commit) => {
                        let source_commit = source_commit.trim();
                        if source_commit.is_empty() {
                            bail!("--source-commit must not be empty");
                        }
                        source_commit.to_string()
                    }
                    None => resolve_source_commit(args.repo_root.as_deref())?,
                };
                let out = args.out;
                let result = agent_skills_catalog(AgentSkillsCatalogArgs {
                    repo_root: args.repo_root,
                    generated_at: args.generated_at,
                    source_repository: args.source_repository,
                    source_commit,
                    source_ref: args.source_ref,
                })?;
                if let Some(out) = out {
                    if let Some(parent) =
                        out.parent().filter(|parent| !parent.as_os_str().is_empty())
                    {
                        fs::create_dir_all(parent)
                            .with_context(|| format!("failed to create {}", parent.display()))?;
                    }
                    write_json(out, &result)?;
                }
                let human = format!(
                    "generated Agent Skills Lab catalog with {} skill(s)",
                    result.skills_count
                );
                Ok(CommandOutput {
                    ok: true,
                    command: "skills catalog",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            SkillsCommand::Inventory(args) => {
                let result = skills_inventory(args)?;
                let human = format!(
                    "inventoried {} skill(s): {} valid, {} invalid",
                    result.total, result.valid, result.invalid
                );
                Ok(CommandOutput {
                    ok: result.ok,
                    command: "skills inventory",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            SkillsCommand::Validate(args) => {
                let result = skills_validate(args)?;
                let human = if result.ok {
                    format!("validated {} skill(s): all valid", result.total)
                } else {
                    format!(
                        "validated {} skill(s): {} valid, {} invalid",
                        result.total, result.valid, result.invalid
                    )
                };
                Ok(CommandOutput {
                    ok: result.ok,
                    command: "skills validate",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            SkillsCommand::Audit(args) => {
                let result = skills_audit(args)?;
                let human = format!(
                    "audited {} skill(s) and {} archived skill(s): {} error(s), {} warning(s)",
                    result.total, result.archive.total, result.error_count, result.warning_count
                );
                Ok(CommandOutput {
                    ok: result.ok,
                    command: "skills audit",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            SkillsCommand::SyncKimi(args) => handle_skills_sync_kimi(args, json_output),
        },
        Commands::Bootstrap { command } => match command {
            BootstrapCommand::Status(args) => {
                let checked_at = args.checked_at.unwrap_or_else(Utc::now);
                let result = bootstrap_status(args, checked_at)?;
                let human = format!(
                    "checked {} bootstrap pack(s): {} valid, {} invalid",
                    result.total, result.valid, result.invalid
                );
                Ok(CommandOutput {
                    ok: result.ok,
                    command: "bootstrap status",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            BootstrapCommand::Plan(args) => {
                let checked_at = args.checked_at.unwrap_or_else(Utc::now);
                let result = bootstrap_plan(args, checked_at)?;
                let human = if result.ok {
                    format!(
                        "planned {} bootstrap file action(s) for {}",
                        result.target_count, result.pack.name
                    )
                } else {
                    format!(
                        "bootstrap plan for {} has {} diagnostic(s)",
                        result.pack.name,
                        result.diagnostics.len()
                    )
                };
                Ok(CommandOutput {
                    ok: result.ok,
                    command: "bootstrap plan",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
        },
        Commands::Task { command } => match command {
            TaskCommand::List(args) => {
                let result = task_index(&args.root)?;
                let human = if result.diagnostics.is_empty() {
                    format!(
                        "listed {} task capsule(s): {} valid, {} invalid",
                        result.total, result.valid, result.invalid
                    )
                } else {
                    format!(
                        "listed {} task capsule(s) with {} diagnostic(s): {} valid, {} invalid",
                        result.total,
                        result.diagnostics.len(),
                        result.valid,
                        result.invalid
                    )
                };
                Ok(CommandOutput {
                    ok: result.invalid == 0 && result.root_status != TaskRootStatus::Unusable,
                    command: "task list",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            TaskCommand::Show(args) => {
                let result = task_show(&args.root, &args.task)?;
                let human = match result.task.capsule.as_ref() {
                    Some(capsule) => format!(
                        "{} [{}] on {}; evidence: {}",
                        capsule.title,
                        capsule.status,
                        capsule.branch,
                        render_evidence_counts(&capsule.evidence.by_kind)
                    ),
                    None => format!(
                        "invalid task capsule at {}: {} issue(s)",
                        result.task.path.display(),
                        result.task.errors.len()
                    ),
                };
                Ok(CommandOutput {
                    ok: result.task.valid,
                    command: "task show",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            TaskCommand::Export(args) => {
                let result = task_export(&args.root, &args.task)?;
                let human = format!(
                    "exported task capsule {} from {}",
                    result.capsule.id,
                    result.task.path.display()
                );
                Ok(CommandOutput {
                    ok: true,
                    command: "task export",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
        },
        Commands::Pr { command } => match command {
            PrCommand::Agent(args) => {
                let checked_at = args.checked_at.unwrap_or_else(Utc::now);
                let result = run_pr_agent_state(args, checked_at)?;
                let blocking = result
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.severity == PrAgentSeverity::Error)
                    .count();
                let human = if blocking == 0 {
                    format!(
                        "recorded dry-run PR agent state for {}#{}; {} recommended action(s)",
                        result.repository,
                        result.number,
                        result.actions.len()
                    )
                } else {
                    format!(
                        "recorded partial dry-run PR agent state for {}#{} with {blocking} error(s)",
                        result.repository, result.number
                    )
                };
                Ok(CommandOutput {
                    ok: blocking == 0,
                    command: "pr agent",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            PrCommand::AgentAction(args) => {
                let generated_at = args.checked_at.unwrap_or_else(Utc::now);
                let result = run_pr_agent_hosted_action(args, generated_at)?;
                let failed = result
                    .execution
                    .as_ref()
                    .is_some_and(|execution| execution.status == PrAgentHostedActionStatus::Failed);
                let human = if result.dry_run {
                    format!(
                        "planned hosted PR action {} for {}#{}; rerun with --apply to execute",
                        result.plan_id, result.repository, result.number
                    )
                } else if let Some(execution) = &result.execution {
                    format!(
                        "hosted PR action {} for {}#{} finished with {:?}",
                        result.plan_id, result.repository, result.number, execution.status
                    )
                } else {
                    format!(
                        "hosted PR action {} for {}#{} did not execute",
                        result.plan_id, result.repository, result.number
                    )
                };
                Ok(CommandOutput {
                    ok: !failed,
                    command: "pr agent-action",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            PrCommand::Readiness(args) => {
                let generated_at = args.checked_at.unwrap_or_else(Utc::now);
                let result = run_pr_readiness_loop(args, generated_at)?;
                let human = format!(
                    "PR readiness for {}#{} is {:?} after {} attempt(s)",
                    result.repository,
                    result.number,
                    result.final_status,
                    result.attempts.len()
                );
                Ok(CommandOutput {
                    ok: matches!(
                        result.final_status,
                        PrAgentReadinessStatus::Ready | PrAgentReadinessStatus::Merged
                    ) && result
                        .actions
                        .iter()
                        .all(|action| action.status != PrAgentReadinessActionStatus::Failed),
                    command: "pr readiness",
                    human,
                    result: serde_json::to_value(result)?,
                })
            }
            PrCommand::Plan(args) => {
                let generated_at = args.generated_at.unwrap_or_else(Utc::now);
                let result = pr_control_plan(args.repo, args.number, generated_at)?;
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
            PrCommand::Review { command } => handle_pr_review_command(command),
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
        Commands::Review { command } => handle_review_command(command),
        Commands::Commit { command } => handle_commit_command(command),
    }
}

fn render_completion(shell: Shell) -> Result<String> {
    let mut command = Cli::command();
    let binary_name = command.get_name().to_string();
    let mut buffer = Vec::new();
    clap_complete::generate(shell, &mut command, binary_name, &mut buffer);
    Ok(String::from_utf8(buffer)?)
}

fn render_manpage() -> Result<String> {
    let command = Cli::command();
    let mut buffer = Vec::new();
    clap_mangen::Man::new(command).render(&mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

fn shell_name(shell: Shell) -> String {
    shell
        .to_possible_value()
        .map(|value| value.get_name().to_string())
        .unwrap_or_else(|| format!("{shell:?}").to_ascii_lowercase())
}

fn handle_skills_sync_kimi(args: SkillsSyncKimiArgs, json_output: bool) -> Result<CommandOutput> {
    if args.install_wrapper && !args.apply {
        bail!("--install-wrapper writes ~/.local/bin/kimi-codex and requires --apply");
    }
    if args.launch && !args.apply {
        bail!("--launch requires --apply so Kimi receives a current mirror");
    }
    if args.launch && json_output {
        bail!("--launch is interactive and cannot be combined with --json");
    }
    if !args.launch && !args.kimi_args.is_empty() {
        bail!("Kimi passthrough arguments require --launch");
    }

    let wrapper_path = args.wrapper_path.clone();
    let launch = args.launch;
    let install_wrapper = args.install_wrapper;
    let kimi_args = args.kimi_args.clone();
    let result = kimi_sync(KimiSyncArgs {
        apply: args.apply,
        scope: args.scope.into(),
        codex_home: args.codex_home,
        agents_home: args.agents_home,
        kimi_home: args.kimi_home,
        project_root: args.project_root,
        checked_at: args.checked_at,
    })?;

    if install_wrapper {
        install_kimi_wrapper(wrapper_path.as_deref())?;
    }
    if launch {
        launch_kimi(&result, &kimi_args)?;
    }

    let mode = if result.dry_run { "planned" } else { "applied" };
    let wrapper = if install_wrapper {
        "; installed kimi-codex wrapper"
    } else {
        ""
    };
    let human = format!(
        "{mode} Kimi skill sync with {} included skill(s), {} excluded skill(s), {} diagnostic(s){wrapper}",
        result.summary.included, result.summary.excluded, result.summary.diagnostics
    );
    Ok(CommandOutput {
        ok: result.ok,
        command: "skills sync-kimi",
        human,
        result: serde_json::to_value(result)?,
    })
}

#[cfg(unix)]
fn install_kimi_wrapper(wrapper_path: Option<&Path>) -> Result<()> {
    let path = match wrapper_path {
        Some(path) => path.to_path_buf(),
        None => default_wrapper_path()?,
    };
    if fs::symlink_metadata(&path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        bail!(
            "refusing to overwrite symlink wrapper path: {}",
            path.display()
        );
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    file.write_all(
        b"#!/usr/bin/env sh\nset -eu\nexec codex-dev skills sync-kimi --apply --launch -- \"$@\"\n",
    )?;
    drop(file);
    set_executable(&path)?;
    Ok(())
}

#[cfg(not(unix))]
fn install_kimi_wrapper(wrapper_path: Option<&Path>) -> Result<()> {
    let _ = wrapper_path;
    bail!("--install-wrapper is currently supported only on Unix");
}

#[cfg(unix)]
fn default_wrapper_path() -> Result<PathBuf> {
    let home = env::var_os("HOME").ok_or_else(|| anyhow::anyhow!("HOME is not set"))?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("bin")
        .join("kimi-codex"))
}

#[cfg(not(unix))]
fn default_wrapper_path() -> Result<PathBuf> {
    bail!("default kimi-codex wrapper path is currently supported only on Unix")
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .with_context(|| format!("failed to inspect {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("failed to chmod {}", path.display()))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<()> {
    Ok(())
}

fn launch_kimi(report: &KimiSyncReport, kimi_args: &[OsString]) -> Result<()> {
    let mut command = Command::new("kimi");
    command.arg("--skills-dir").arg(&report.skills_root);
    command.args(kimi_args);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let error = command.exec();
        bail!("failed to launch kimi: {error}");
    }
    #[cfg(not(unix))]
    {
        let status = command.status().context("failed to launch kimi")?;
        if status.success() {
            Ok(())
        } else {
            bail!("kimi exited with status {status}")
        }
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

fn resolve_source_commit(repo_root: Option<&Path>) -> Result<String> {
    let mut command = Command::new("git");
    if let Some(repo_root) = repo_root {
        command.arg("-C").arg(repo_root);
    }
    let output = command
        .args(["rev-parse", "HEAD"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run git rev-parse HEAD")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("failed to resolve source commit with git rev-parse HEAD: {stderr}");
    }
    let source_commit = String::from_utf8(output.stdout)
        .context("git rev-parse HEAD emitted non-UTF-8 output")?
        .trim()
        .to_string();
    if source_commit.is_empty() {
        bail!("git rev-parse HEAD emitted an empty source commit");
    }
    Ok(source_commit)
}

fn orchestration_output(
    command: &'static str,
    report: OrchestrationRunReport,
    require_complete: bool,
) -> Result<CommandOutput> {
    let blocking = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == OrchestrationDiagnosticSeverity::Error)
        .count();
    let write_blocking = report.diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.code.as_str(),
            "invalid_capsule" | "invalid_subagents_contract" | "unexpected_agent"
        )
    });
    let complete = blocking == 0 && report.completion.complete && report.synthesis_status.is_some();
    let human = if complete {
        format!(
            "orchestration batch {} complete: {}/{} role(s), synthesis {}",
            report.batch_id,
            report.completion.human_verified,
            report.completion.expected,
            report.synthesis_status.as_deref().unwrap_or("missing")
        )
    } else if require_complete {
        format!(
            "orchestration batch {} incomplete: {} blocking diagnostic(s), {}/{} role(s) verified",
            report.batch_id, blocking, report.completion.human_verified, report.completion.expected
        )
    } else {
        format!(
            "recorded orchestration batch {} with {} error diagnostic(s), {}/{} role(s) verified",
            report.batch_id, blocking, report.completion.human_verified, report.completion.expected
        )
    };
    let ok = if require_complete {
        complete
    } else {
        !write_blocking
    };
    Ok(CommandOutput {
        ok,
        command,
        human,
        result: serde_json::to_value(report)?,
    })
}

/// Build a read-only machine-readable inventory of tracked skill folders.
pub fn skills_inventory(
    args: SkillsInventoryArgs,
) -> Result<codex_dev_core::SkillsInventoryReport> {
    codex_dev_core::skills_inventory(codex_dev_core::SkillInventoryArgs {
        repo_root: args.repo_root,
        skills_root: args.skills_root,
        checked_at: args.checked_at,
    })
}

/// Build a read-only validation report for skill folders.
pub fn skills_validate(args: SkillsValidateArgs) -> Result<codex_dev_core::SkillsInventoryReport> {
    codex_dev_core::skills_inventory(codex_dev_core::SkillInventoryArgs {
        repo_root: args.repo_root,
        skills_root: args.skills_root,
        checked_at: args.checked_at,
    })
}

/// Build a read-only hygiene audit for skill folders.
pub fn skills_audit(args: SkillsAuditArgs) -> Result<codex_dev_core::SkillsAuditReport> {
    codex_dev_core::skills_audit(codex_dev_core::SkillAuditArgs {
        repo_root: args.repo_root,
        skills_root: args.skills_root,
        checked_at: args.checked_at,
        max_skill_md_lines: args.max_skill_md_lines,
    })
}

/// Build a read-only machine-readable bootstrap pack status report.
pub fn bootstrap_status(
    args: BootstrapStatusArgs,
    checked_at: DateTime<Utc>,
) -> Result<BootstrapStatusReport> {
    let repo_root = resolve_policy_docs_repo_root(args.repo_root.as_deref())?;
    let pack_root = repo_root.join("bootstrap/packs");
    let mut diagnostics = Vec::new();
    let repo_root_display = bootstrap_local_path_display(&repo_root, args.include_local_paths);
    let pack_paths = bootstrap_pack_paths(
        &repo_root,
        &pack_root,
        args.pack.as_deref(),
        &mut diagnostics,
    );
    let packs = pack_paths
        .iter()
        .map(|path| bootstrap_pack_status(&repo_root, path))
        .collect::<Vec<_>>();
    let valid = packs.iter().filter(|pack| pack.valid).count();
    let invalid = packs.len().saturating_sub(valid);
    let ok = diagnostics
        .iter()
        .all(|diagnostic| diagnostic.severity != LocalDiagnosticSeverity::Error)
        && invalid == 0;

    Ok(BootstrapStatusReport {
        schema: BOOTSTRAP_STATUS_SCHEMA,
        checked_at,
        repo_root: repo_root_display,
        pack_root: "bootstrap/packs".to_string(),
        template_root: "bootstrap/templates".to_string(),
        ok,
        total: packs.len(),
        valid,
        invalid,
        diagnostics,
        packs,
        policy_gates: bootstrap_policy_gate_summary(),
    })
}

/// Build a read-only machine-readable bootstrap pack dry-run plan.
pub fn bootstrap_plan(
    args: BootstrapPlanArgs,
    checked_at: DateTime<Utc>,
) -> Result<BootstrapPlanReport> {
    let repo_root = resolve_policy_docs_repo_root(args.repo_root.as_deref())?;
    let mut status = bootstrap_status(
        BootstrapStatusArgs {
            repo_root: Some(repo_root.clone()),
            pack: Some(args.pack.clone()),
            include_local_paths: args.include_local_paths,
            checked_at: Some(checked_at),
        },
        checked_at,
    )?;
    let pack = status.packs.pop().unwrap_or_else(|| BootstrapPackStatus {
        name: args.pack.clone(),
        path: format!("bootstrap/packs/{}.json", args.pack),
        schema: String::new(),
        description: String::new(),
        valid: false,
        errors: vec![format!("missing bootstrap pack: {}", args.pack)],
        file_count: 0,
        files: Vec::new(),
        composes: BootstrapPackComposesStatus::default(),
        advisory_host_checks: Vec::new(),
    });
    let mut diagnostics = status.diagnostics;
    diagnostics.extend(pack.errors.iter().map(|error| BootstrapDiagnostic {
        severity: LocalDiagnosticSeverity::Error,
        code: "invalid_bootstrap_pack".to_string(),
        message: error.clone(),
    }));

    let repo_root_display = bootstrap_local_path_display(&repo_root, args.include_local_paths);
    let mut action_counts = BTreeMap::new();
    let mut files = Vec::new();
    let (output_root, output_root_display) = match normalize_output_root(&args.out) {
        Ok(output_root) => {
            let display = if args.include_local_paths {
                output_root.display().to_string()
            } else {
                "<bootstrap-out>".to_string()
            };
            (Some(output_root), display)
        }
        Err(error) => {
            let message = if args.include_local_paths {
                format!(
                    "failed to resolve bootstrap output root {}: {error}",
                    args.out.display()
                )
            } else {
                "failed to resolve bootstrap output root".to_string()
            };
            diagnostics.push(bootstrap_error("invalid_bootstrap_output_root", message));
            let display = if args.include_local_paths {
                args.out.display().to_string()
            } else {
                "<bootstrap-out>".to_string()
            };
            (None, display)
        }
    };

    if pack.valid
        && let Some(output_root) = &output_root
    {
        for file in &pack.files {
            let target = safe_bootstrap_relative_path(&file.target, &pack.path, "files[].target")
                .map(PathBuf::from)
                .map_err(anyhow::Error::msg)?;
            let target_path = match resolve_bootstrap_output_target(output_root, &target) {
                Ok(path) => path,
                Err(error) => {
                    diagnostics.push(bootstrap_error("output_target_escape", error));
                    continue;
                }
            };
            let action = if target_path.exists() {
                "would_overwrite"
            } else {
                "would_write"
            }
            .to_string();
            *action_counts.entry(action.clone()).or_insert(0) += 1;
            files.push(BootstrapPlannedFile {
                target: file.target.clone(),
                target_path: args.include_local_paths.then_some(target_path),
                template: file.template.clone(),
                action,
            });
        }
    }

    let ok = pack.valid
        && diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != LocalDiagnosticSeverity::Error);
    let target_count = files.len();
    let advisory_host_checks = pack.advisory_host_checks.clone();

    Ok(BootstrapPlanReport {
        schema: BOOTSTRAP_PLAN_SCHEMA,
        checked_at,
        repo_root: repo_root_display,
        pack,
        ok,
        dry_run: true,
        output_root: output_root_display,
        repo_name: args.repo_name,
        primary_language: args.primary_language,
        target_count,
        action_counts,
        files,
        advisory_host_checks,
        diagnostics,
    })
}

fn bootstrap_pack_paths(
    repo_root: &Path,
    pack_root: &Path,
    pack: Option<&str>,
    diagnostics: &mut Vec<BootstrapDiagnostic>,
) -> Vec<PathBuf> {
    if let Err(error) = validate_bootstrap_repo_path(repo_root, "bootstrap/packs", "pack root") {
        diagnostics.push(bootstrap_error("invalid_bootstrap_pack_root", error));
        return Vec::new();
    }

    if let Some(pack) = pack {
        if safe_bootstrap_relative_path(pack, "bootstrap pack", "--pack").is_err()
            || pack.contains('/')
            || pack.contains('\\')
        {
            diagnostics.push(bootstrap_error(
                "unsafe_bootstrap_pack",
                format!("bootstrap pack name is not safe: {pack}"),
            ));
            return Vec::new();
        }
        let path = pack_root.join(format!("{pack}.json"));
        if !path.is_file() {
            diagnostics.push(bootstrap_error(
                "missing_bootstrap_pack",
                format!("bootstrap pack does not exist: {pack}"),
            ));
            return Vec::new();
        }
        if let Err(error) =
            validate_bootstrap_existing_path(repo_root, &path, "bootstrap pack manifest")
        {
            diagnostics.push(bootstrap_error("invalid_bootstrap_pack_manifest", error));
            return Vec::new();
        }
        return vec![path];
    }

    if !pack_root.is_dir() {
        diagnostics.push(bootstrap_error(
            "missing_bootstrap_pack_root",
            "bootstrap pack root is not a directory: bootstrap/packs".to_string(),
        ));
        return Vec::new();
    }

    let mut paths = Vec::new();
    match fs::read_dir(pack_root) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.extension().and_then(|value| value.to_str()) == Some("json") {
                            if let Err(error) = validate_bootstrap_existing_path(
                                repo_root,
                                &path,
                                "bootstrap pack manifest",
                            ) {
                                diagnostics.push(bootstrap_error(
                                    "invalid_bootstrap_pack_manifest",
                                    error,
                                ));
                                continue;
                            }
                            paths.push(path);
                        }
                    }
                    Err(error) => diagnostics.push(bootstrap_error(
                        "bootstrap_pack_entry_read_error",
                        format!("failed to read bootstrap pack entry in bootstrap/packs: {error}"),
                    )),
                }
            }
        }
        Err(error) => diagnostics.push(bootstrap_error(
            "bootstrap_pack_root_read_error",
            format!("failed to read bootstrap pack root bootstrap/packs: {error}"),
        )),
    }
    paths.sort();
    if paths.is_empty() && diagnostics.is_empty() {
        diagnostics.push(bootstrap_error(
            "missing_bootstrap_packs",
            "bootstrap pack root contains no JSON pack manifests".to_string(),
        ));
    }
    paths
}

fn bootstrap_pack_status(repo_root: &Path, path: &Path) -> BootstrapPackStatus {
    let relative_path = repo_relative_string(repo_root, path);
    let fallback_name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .to_string();
    if let Err(error) = validate_bootstrap_existing_path(repo_root, path, "bootstrap pack manifest")
    {
        return BootstrapPackStatus {
            name: fallback_name,
            path: relative_path,
            schema: String::new(),
            description: String::new(),
            valid: false,
            errors: vec![error],
            file_count: 0,
            files: Vec::new(),
            composes: BootstrapPackComposesStatus::default(),
            advisory_host_checks: Vec::new(),
        };
    }
    let payload = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            return BootstrapPackStatus {
                name: fallback_name,
                path: relative_path,
                schema: String::new(),
                description: String::new(),
                valid: false,
                errors: vec![format!("failed to read pack manifest: {error}")],
                file_count: 0,
                files: Vec::new(),
                composes: BootstrapPackComposesStatus::default(),
                advisory_host_checks: Vec::new(),
            };
        }
    };
    let value = match serde_json::from_str::<Value>(&payload) {
        Ok(value) => value,
        Err(error) => {
            return BootstrapPackStatus {
                name: fallback_name,
                path: relative_path,
                schema: String::new(),
                description: String::new(),
                valid: false,
                errors: vec![format!("invalid pack JSON: {error}")],
                file_count: 0,
                files: Vec::new(),
                composes: BootstrapPackComposesStatus::default(),
                advisory_host_checks: Vec::new(),
            };
        }
    };

    bootstrap_pack_status_from_value(repo_root, path, relative_path, fallback_name, &value)
}

fn bootstrap_pack_status_from_value(
    repo_root: &Path,
    path: &Path,
    relative_path: String,
    fallback_name: String,
    value: &Value,
) -> BootstrapPackStatus {
    let mut errors = Vec::new();
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if schema != BOOTSTRAP_PACK_SCHEMA {
        errors.push(format!(
            "{relative_path}: schema must be {BOOTSTRAP_PACK_SCHEMA}"
        ));
    }
    let name = match value.get("name").and_then(Value::as_str) {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => {
            errors.push(format!("{relative_path}: name must be a non-empty string"));
            fallback_name
        }
    };
    let description = value
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let composes = bootstrap_pack_composes(&relative_path, value, &mut errors);
    let files = bootstrap_pack_files(repo_root, path, &relative_path, value, &mut errors);
    let advisory_host_checks = bootstrap_string_array_field(
        &relative_path,
        value.get("advisory_host_checks"),
        "advisory_host_checks",
        false,
        &mut errors,
    );

    BootstrapPackStatus {
        name,
        path: relative_path,
        schema,
        description,
        valid: errors.is_empty(),
        errors,
        file_count: files.len(),
        files,
        composes,
        advisory_host_checks,
    }
}

fn bootstrap_pack_composes(
    relative_path: &str,
    value: &Value,
    errors: &mut Vec<String>,
) -> BootstrapPackComposesStatus {
    let Some(composes) = value.get("composes") else {
        return BootstrapPackComposesStatus::default();
    };
    let Some(composes) = composes.as_object() else {
        errors.push(format!(
            "{relative_path}: composes must be an object when present"
        ));
        return BootstrapPackComposesStatus::default();
    };
    BootstrapPackComposesStatus {
        skills: bootstrap_string_array_field(
            relative_path,
            composes.get("skills"),
            "composes.skills",
            false,
            errors,
        ),
        subagent_sources: bootstrap_string_array_field(
            relative_path,
            composes.get("subagent_sources"),
            "composes.subagent_sources",
            false,
            errors,
        ),
    }
}

fn bootstrap_pack_files(
    repo_root: &Path,
    manifest_path: &Path,
    relative_path: &str,
    value: &Value,
    errors: &mut Vec<String>,
) -> Vec<BootstrapPackFileStatus> {
    let Some(files) = value.get("files").and_then(Value::as_array) else {
        errors.push(format!("{relative_path}: files must be a non-empty array"));
        return Vec::new();
    };
    if files.is_empty() {
        errors.push(format!("{relative_path}: files must be a non-empty array"));
        return Vec::new();
    }
    let mut output = Vec::new();
    for (index, item) in files.iter().enumerate() {
        let Some(item) = item.as_object() else {
            errors.push(format!("{relative_path}: files[{index}] must be an object"));
            continue;
        };
        let target = item.get("target").and_then(Value::as_str);
        let template = item.get("template").and_then(Value::as_str);
        let mut entry_valid = true;
        let Some(target) = target.filter(|target| !target.is_empty()) else {
            errors.push(format!(
                "{relative_path}: files[{index}].target must be a string"
            ));
            continue;
        };
        if let Err(error) = safe_bootstrap_relative_path(target, relative_path, "files[].target") {
            errors.push(error);
            entry_valid = false;
        }
        let Some(template) = template.filter(|template| !template.is_empty()) else {
            errors.push(format!(
                "{relative_path}: files[{index}].template must be a string"
            ));
            continue;
        };
        let template_exists = match bootstrap_template_exists(repo_root, manifest_path, template) {
            Ok(exists) => exists,
            Err(error) => {
                errors.push(error);
                entry_valid = false;
                false
            }
        };
        if !template_exists {
            errors.push(format!("{relative_path}: missing template {template}"));
            entry_valid = false;
        }
        if entry_valid {
            output.push(BootstrapPackFileStatus {
                target: target.to_string(),
                template: format!("bootstrap/templates/{template}"),
                template_exists,
            });
        }
    }
    output
}

fn bootstrap_string_array_field(
    relative_path: &str,
    value: Option<&Value>,
    label: &str,
    required: bool,
    errors: &mut Vec<String>,
) -> Vec<String> {
    let Some(value) = value else {
        if required {
            errors.push(format!(
                "{relative_path}: {label} must be a non-empty array"
            ));
        }
        return Vec::new();
    };
    let Some(values) = value.as_array() else {
        errors.push(format!(
            "{relative_path}: {label} must be a non-empty array"
        ));
        return Vec::new();
    };
    if required && values.is_empty() {
        errors.push(format!(
            "{relative_path}: {label} must be a non-empty array"
        ));
    }
    let mut output = Vec::new();
    for (index, item) in values.iter().enumerate() {
        match item.as_str() {
            Some(item) if !item.is_empty() => output.push(item.to_string()),
            _ => errors.push(format!(
                "{relative_path}: {label}[{index}] must be a non-empty string"
            )),
        }
    }
    output
}

fn bootstrap_template_exists(
    repo_root: &Path,
    manifest_path: &Path,
    template: &str,
) -> Result<bool, String> {
    let relative = safe_bootstrap_relative_path(
        template,
        &repo_relative_string(repo_root, manifest_path),
        "files[].template",
    )?;
    validate_bootstrap_repo_path(repo_root, "bootstrap/templates", "template root")?;
    let template_relative = Path::new("bootstrap/templates").join(&relative);
    let template_relative = template_relative.to_string_lossy().replace('\\', "/");
    reject_bootstrap_repo_symlinks(repo_root, &template_relative, "bootstrap template")?;
    let candidate = repo_root.join(&template_relative);
    let Ok(metadata) = fs::metadata(&candidate) else {
        return Ok(false);
    };
    if !metadata.is_file() {
        return Ok(false);
    }
    validate_bootstrap_existing_path(repo_root, &candidate, "bootstrap template")?;
    Ok(true)
}

fn safe_bootstrap_relative_path(
    value: &str,
    source: &str,
    label: &str,
) -> std::result::Result<String, String> {
    let normalized = value.replace('\\', "/");
    let has_windows_drive = value.as_bytes().get(1) == Some(&b':')
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic);
    if value.is_empty()
        || Path::new(value).is_absolute()
        || normalized.starts_with('/')
        || normalized.starts_with("//")
        || has_windows_drive
        || normalized.split('/').any(|part| part == "..")
    {
        return Err(format!("unsafe relative path in {source} {label}: {value}"));
    }
    Ok(value.to_string())
}

fn normalize_output_root(path: &Path) -> Result<PathBuf> {
    let expanded = expand_home_path(path)?;
    let output = if expanded.is_absolute() {
        expanded
    } else {
        env::current_dir()
            .context("failed to read current directory")?
            .join(expanded)
    };
    resolve_path_allow_missing(&output)
}

fn expand_home_path(path: &Path) -> Result<PathBuf> {
    let raw = path.to_string_lossy();
    if raw == "~" {
        return Ok(PathBuf::from(home_dir()?));
    }
    if let Some(rest) = raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\")) {
        return Ok(PathBuf::from(home_dir()?).join(rest));
    }
    Ok(path.to_path_buf())
}

fn home_dir() -> Result<OsString> {
    env::var_os("HOME").ok_or_else(|| anyhow::anyhow!("HOME is required to expand ~ paths"))
}

fn resolve_bootstrap_output_target(
    output_root: &Path,
    target: &Path,
) -> std::result::Result<PathBuf, String> {
    let candidate = output_root.join(target);
    let resolved = resolve_path_allow_missing(&candidate).map_err(|_| {
        format!(
            "failed to resolve bootstrap output target: {}",
            target.display()
        )
    })?;
    if path_within(&resolved, output_root) {
        Ok(resolved)
    } else {
        Err(format!(
            "bootstrap output target escapes output root: {}",
            target.display()
        ))
    }
}

fn resolve_path_allow_missing(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path)
            .with_context(|| format!("failed to resolve path {}", path.display()));
    }

    let mut existing = path.to_path_buf();
    let mut missing = Vec::new();
    while !existing.exists() {
        let Some(name) = existing.file_name().map(OsString::from) else {
            break;
        };
        missing.push(name);
        if !existing.pop() {
            break;
        }
    }
    let mut resolved = fs::canonicalize(&existing)
        .with_context(|| format!("failed to resolve path prefix {}", existing.display()))?;
    for name in missing.iter().rev() {
        resolved.push(name);
    }
    Ok(resolved)
}

fn validate_bootstrap_repo_path(
    repo_root: &Path,
    relative: &str,
    label: &str,
) -> std::result::Result<(), String> {
    reject_bootstrap_repo_symlinks(repo_root, relative, label)?;
    let path = repo_root.join(relative);
    validate_bootstrap_existing_path(repo_root, &path, label)
}

fn validate_bootstrap_existing_path(
    repo_root: &Path,
    path: &Path,
    label: &str,
) -> std::result::Result<(), String> {
    let relative = repo_relative_string(repo_root, path);
    reject_bootstrap_repo_symlinks(repo_root, &relative, label)?;
    let canonical_repo = fs::canonicalize(repo_root)
        .map_err(|error| format!("failed to inspect repo root: {error}"))?;
    let resolved = fs::canonicalize(path)
        .map_err(|error| format!("failed to inspect {label} {relative}: {error}"))?;
    if path_within(&resolved, &canonical_repo) {
        Ok(())
    } else {
        Err(format!("{label} escapes repository root: {relative}"))
    }
}

fn reject_bootstrap_repo_symlinks(
    repo_root: &Path,
    relative: &str,
    label: &str,
) -> std::result::Result<(), String> {
    let relative = safe_bootstrap_relative_path(relative, "bootstrap path", label)?;
    let mut current = repo_root.to_path_buf();
    for component in Path::new(&relative).components() {
        match component {
            Component::Normal(part) => current.push(part),
            _ => {
                return Err(format!(
                    "unsafe relative path in bootstrap path {label}: {relative}"
                ));
            }
        }
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "{label} contains symlink: {}",
                    repo_relative_string(repo_root, &current)
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(format!(
                    "failed to inspect {label} {}: {error}",
                    repo_relative_string(repo_root, &current)
                ));
            }
        }
    }
    Ok(())
}

fn path_within(path: &Path, root: &Path) -> bool {
    path == root || path.starts_with(root)
}

fn bootstrap_local_path_display(path: &Path, include_local_paths: bool) -> String {
    if include_local_paths {
        path.display().to_string()
    } else {
        "<repo-root>".to_string()
    }
}

fn repo_relative_string(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.display().to_string())
}

fn bootstrap_error(code: &str, message: String) -> BootstrapDiagnostic {
    BootstrapDiagnostic {
        severity: LocalDiagnosticSeverity::Error,
        code: code.to_string(),
        message,
    }
}

fn bootstrap_policy_gate_summary() -> BootstrapPolicyGateSummary {
    let gates = built_in_gates(PolicyProfile::BootstrapInstall);
    BootstrapPolicyGateSummary {
        profile: PolicyProfile::BootstrapInstall,
        gate_count: gates.len(),
        required_gate_count: gates.iter().filter(|gate| gate.required).count(),
        gate_ids: gates.into_iter().map(|gate| gate.id).collect(),
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

/// Build a read-only local readiness report for the current workstation and checkout.
pub fn local_doctor(args: LocalDoctorArgs, mode: LocalReportMode) -> Result<LocalDoctorReport> {
    let checked_at = args.checked_at.unwrap_or_else(Utc::now);
    let cwd = env::current_dir().context("failed to read current directory")?;
    let repo_root = match args.repo_root {
        Some(path) => canonicalize_repo_root(&path)?,
        None => find_repo_root(&cwd).ok_or_else(|| {
            anyhow::anyhow!(
                "failed to discover repository root from current directory; run from the repo or pass --repo-root"
            )
        })?,
    };

    let binaries = ["codex-dev", "codex-dev-tui", "codex-research"]
        .into_iter()
        .map(|name| local_tool_status(name, args.strict_global_binaries, false))
        .collect::<Vec<_>>();
    let tools = [
        ("cargo", true, true),
        ("rustc", true, true),
        ("git", true, true),
        ("gh", true, true),
        ("python3", true, true),
        ("cargo-deny", false, true),
        ("cargo-audit", false, true),
    ]
    .into_iter()
    .map(|(name, required, version)| local_tool_status(name, required, version))
    .collect::<Vec<_>>();

    let github = local_github_status();
    let capsule_root = local_path_status(&repo_root, "capsule_root", ".codex/tasks");
    let mut cache_roots = vec![
        local_path_status(&repo_root, "research_cache", ".codex/research"),
        local_path_status(
            &repo_root,
            "install_smoke_target",
            "target/codex-dev-install-smoke",
        ),
    ];
    if let Some(path) = codex_cache_dir() {
        cache_roots.push(global_cache_status(
            &repo_root,
            normalize_local_path(&repo_root, path),
        ));
    }

    let policy_profiles = all_policy_profiles()
        .into_iter()
        .map(|profile| {
            let gates = built_in_gates(profile);
            LocalPolicyProfileStatus {
                profile,
                gates: gates.len(),
                required_gates: gates.iter().filter(|gate| gate.required).count(),
                network_gates: gates.iter().filter(|gate| gate.network).count(),
                secret_gates: gates.iter().filter(|gate| gate.secrets).count(),
            }
        })
        .collect::<Vec<_>>();

    let diagnostics = local_diagnostics(
        &binaries,
        &tools,
        &github,
        &capsule_root,
        &repo_root,
        &cache_roots,
    );
    let ok = diagnostics
        .iter()
        .all(|diagnostic| diagnostic.severity != LocalDiagnosticSeverity::Error);

    Ok(LocalDoctorReport {
        schema: LOCAL_DOCTOR_SCHEMA,
        mode,
        checked_at,
        cwd,
        repo_root,
        ok,
        diagnostics,
        binaries,
        tools,
        github,
        capsule_root,
        cache_roots,
        policy_profiles,
    })
}

/// Render the local readiness report as a compact human-facing summary.
fn render_local_report_human(report: &LocalDoctorReport) -> String {
    let errors = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == LocalDiagnosticSeverity::Error)
        .count();
    let warnings = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == LocalDiagnosticSeverity::Warning)
        .count();
    let binaries = report
        .binaries
        .iter()
        .filter(|binary| binary.available)
        .count();
    let tools = report.tools.iter().filter(|tool| tool.available).count();
    format!(
        "local {:?}: {} error(s), {} warning(s), {}/{} global binary(s), {}/{} tool(s)",
        report.mode,
        errors,
        warnings,
        binaries,
        report.binaries.len(),
        tools,
        report.tools.len()
    )
}

/// Inspect one executable expected on PATH.
fn local_tool_status(name: &str, required: bool, include_version: bool) -> LocalToolStatus {
    let path = find_executable_on_path(name);
    let version = path
        .as_ref()
        .filter(|_| include_version)
        .and_then(|path| command_version(path));
    LocalToolStatus {
        name: name.to_string(),
        required,
        available: path.is_some(),
        path,
        version,
    }
}

/// Detect GitHub CLI availability and categorical authentication source hints.
fn local_github_status() -> LocalGithubStatus {
    let gh_path = find_executable_on_path("gh");
    let token_sources = GITHUB_TOKEN_ENV_VARS
        .iter()
        .filter(|name| non_empty_env_var(name))
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();
    let config_present = gh_config_dir()
        .map(|config_dir| config_dir.join("hosts.yml").is_file())
        .unwrap_or(false);
    let auth_class = if !token_sources.is_empty() {
        "env_token"
    } else if config_present {
        "gh_config"
    } else if gh_path.is_some() {
        "gh_available_no_auth_hint"
    } else {
        "gh_missing"
    }
    .to_string();
    LocalGithubStatus {
        gh_available: gh_path.is_some(),
        gh_path,
        token_sources,
        config_present,
        auth_class,
    }
}

/// Read a path-valued environment variable when it is present and non-empty.
fn non_empty_env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// Return true when an environment variable is present and non-empty.
fn non_empty_env_var(name: &str) -> bool {
    env::var_os(name).is_some_and(|value| !value.is_empty())
}

/// Resolve the GitHub CLI config directory using GitHub CLI environment precedence.
fn gh_config_dir() -> Option<PathBuf> {
    non_empty_env_path("GH_CONFIG_DIR")
        .or_else(|| non_empty_env_path("XDG_CONFIG_HOME").map(|path| path.join("gh")))
        .or(windows_appdata_gh_config_dir())
        .or_else(|| non_empty_env_path("HOME").map(|path| path.join(".config/gh")))
}

/// Resolve the Windows GitHub CLI config fallback from APPDATA.
fn windows_appdata_gh_config_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        non_empty_env_path("APPDATA").map(|path| path.join("GitHub CLI"))
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Resolve the codex-research cache directory using XDG cache precedence.
fn codex_cache_dir() -> Option<PathBuf> {
    non_empty_env_path("XDG_CACHE_HOME")
        .map(|path| path.join("codex-research"))
        .or_else(|| non_empty_env_path("HOME").map(|path| path.join(".cache/codex-research")))
}

/// Resolve relative environment-derived paths against the inspected repository.
fn normalize_local_path(repo_root: &Path, path: PathBuf) -> PathBuf {
    let path = if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    };
    path.canonicalize().unwrap_or(path)
}

/// Inspect a repository-local path and its git-ignore state.
fn local_path_status(repo_root: &Path, name: &str, relative: &str) -> LocalPathStatus {
    let path = repo_root.join(relative);
    LocalPathStatus {
        name: name.to_string(),
        path,
        exists: repo_root.join(relative).exists(),
        git_ignored: git_check_ignored(repo_root, &directory_ignore_probe(relative)),
    }
}

/// Inspect a cache path that may be outside the repository.
fn global_cache_status(repo_root: &Path, path: PathBuf) -> LocalPathStatus {
    let git_ignored = repo_relative_path_for_git(repo_root, &path)
        .and_then(|relative| git_check_ignored(repo_root, &directory_ignore_probe(&relative)));
    LocalPathStatus {
        name: "global_codex_cache".to_string(),
        exists: path.exists(),
        path,
        git_ignored,
    }
}

/// Convert a repository-local path into the slash-separated form expected by git.
fn repo_relative_path_for_git(repo_root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(repo_root).ok()?;
    if relative.as_os_str().is_empty() {
        return None;
    }
    Some(relative.to_string_lossy().replace('\\', "/"))
}

/// Probe a directory-style path so ignore rules for the directory and its contents apply.
fn directory_ignore_probe(relative: &str) -> String {
    format!("{}/probe", relative.trim_end_matches('/'))
}

/// Convert collected local readiness facts into actionable diagnostics.
fn local_diagnostics(
    binaries: &[LocalToolStatus],
    tools: &[LocalToolStatus],
    github: &LocalGithubStatus,
    capsule_root: &LocalPathStatus,
    repo_root: &Path,
    cache_roots: &[LocalPathStatus],
) -> Vec<LocalDiagnostic> {
    let mut diagnostics = Vec::new();
    for status in binaries.iter().chain(tools.iter()) {
        if status.required && !status.available {
            diagnostics.push(LocalDiagnostic {
                severity: LocalDiagnosticSeverity::Error,
                code: format!("missing_{}", status.name.replace('-', "_")),
                message: format!("required command `{}` was not found on PATH", status.name),
            });
        } else if !status.required && !status.available {
            diagnostics.push(LocalDiagnostic {
                severity: LocalDiagnosticSeverity::Warning,
                code: format!("missing_optional_{}", status.name.replace('-', "_")),
                message: format!("optional command `{}` was not found on PATH", status.name),
            });
        }
    }
    if github.gh_available && github.auth_class == "gh_available_no_auth_hint" {
        diagnostics.push(LocalDiagnostic {
            severity: LocalDiagnosticSeverity::Warning,
            code: "github_auth_unverified".to_string(),
            message: "`gh` is installed, but no env token or gh hosts config was detected"
                .to_string(),
        });
    }
    match capsule_root.git_ignored {
        Some(true) => {}
        Some(false) => diagnostics.push(LocalDiagnostic {
            severity: LocalDiagnosticSeverity::Error,
            code: "capsule_root_not_ignored".to_string(),
            message: format!(
                "local capsule root {} must be ignored by git",
                capsule_root.path.display()
            ),
        }),
        None => diagnostics.push(LocalDiagnostic {
            severity: LocalDiagnosticSeverity::Error,
            code: "capsule_root_ignore_unknown".to_string(),
            message: format!(
                "unable to determine whether local capsule root {} is ignored by git",
                capsule_root.path.display()
            ),
        }),
    }
    for cache_root in cache_roots {
        match cache_root.git_ignored {
            Some(true) => {}
            Some(false) => diagnostics.push(LocalDiagnostic {
                severity: LocalDiagnosticSeverity::Error,
                code: format!("{}_not_ignored", cache_root.name),
                message: format!(
                    "local cache root {} must be ignored by git",
                    cache_root.path.display()
                ),
            }),
            None if cache_root.name == "global_codex_cache"
                && repo_relative_path_for_git(repo_root, &cache_root.path).is_none() => {}
            None => diagnostics.push(LocalDiagnostic {
                severity: LocalDiagnosticSeverity::Error,
                code: format!("{}_ignore_unknown", cache_root.name),
                message: format!(
                    "unable to determine whether local cache root {} is ignored by git",
                    cache_root.path.display()
                ),
            }),
        }
    }
    if diagnostics.is_empty() {
        diagnostics.push(LocalDiagnostic {
            severity: LocalDiagnosticSeverity::Info,
            code: "local_ready".to_string(),
            message: "required local development tools and ignored capsule root are present"
                .to_string(),
        });
    }
    diagnostics
}

/// Captured result from a bounded local subprocess probe.
struct LocalProbeOutput {
    /// Whether the subprocess exited successfully.
    success: bool,
    /// Numeric process exit code when the platform reported one.
    code: Option<i32>,
    /// Bounded standard output captured from the subprocess.
    stdout: Vec<u8>,
}

/// Return true when a path is a usable executable file for the current platform.
fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Run a local command without inherited environment and with bounded output/time.
fn run_bounded_local_probe(
    command: &Path,
    args: &[&str],
    max_stdout_bytes: Option<u64>,
) -> Option<LocalProbeOutput> {
    let mut child = Command::new(command)
        .args(args)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let limit = max_stdout_bytes.unwrap_or(4096);
    let reader = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stdout.by_ref().take(limit).read_to_end(&mut buffer);
        buffer
    });
    let deadline = Instant::now() + Duration::from_secs(2);
    let status = loop {
        if let Some(status) = child.try_wait().ok()? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            return None;
        }
        sleep(Duration::from_millis(10));
    };
    let stdout = reader.join().ok()?;
    Some(LocalProbeOutput {
        success: status.success(),
        code: status.code(),
        stdout,
    })
}

/// Build executable file-name candidates for PATH lookup on the current platform.
fn executable_candidates(command: &str) -> Vec<OsString> {
    #[cfg(windows)]
    {
        if Path::new(command).extension().is_some() {
            return vec![OsString::from(command)];
        }
        let mut names = vec![OsString::from(command)];
        let pathext =
            env::var_os("PATHEXT").unwrap_or_else(|| OsString::from(".COM;.EXE;.BAT;.CMD"));
        names.extend(
            pathext
                .to_string_lossy()
                .split(';')
                .map(str::trim)
                .filter(|extension| !extension.is_empty())
                .map(|extension| {
                    if extension.starts_with('.') {
                        format!("{command}{extension}")
                    } else {
                        format!("{command}.{extension}")
                    }
                })
                .map(OsString::from),
        );
        names
    }
    #[cfg(not(windows))]
    {
        vec![OsString::from(command)]
    }
}

/// Resolve the first executable candidate found on PATH.
fn find_executable_on_path(command: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    let candidates = executable_candidates(command);
    env::split_paths(&paths).find_map(|dir| {
        candidates
            .iter()
            .map(|candidate| dir.join(candidate))
            .find(|path| is_executable_file(path))
    })
}

/// Read and redact the first line of a command's `--version` output.
fn command_version(command: &Path) -> Option<String> {
    let output = run_bounded_local_probe(command, &["--version"], None)?;
    if !output.success {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|text| {
            redact_sensitive_text(text.lines().next().unwrap_or("").trim())
                .chars()
                .take(240)
                .collect::<String>()
        })
        .filter(|line| !line.is_empty())
}

/// Check whether a repository-relative path is ignored by git.
fn git_check_ignored(repo_root: &Path, relative: &str) -> Option<bool> {
    let git = find_executable_on_path("git")?;
    let repo = repo_root.to_string_lossy().to_string();
    let output = run_bounded_local_probe(
        &git,
        &["-C", &repo, "check-ignore", "-q", "--", relative],
        Some(1024),
    )?;
    match output.code {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    }
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

pub fn policy_explain(
    args: PolicyExplainArgs,
    checked_at: DateTime<Utc>,
) -> Result<PolicyExplainReport> {
    let include_local_paths = args.include_local_paths;
    policy_explain_inner(args, checked_at)
        .map_err(|error| policy_explain_error_without_local_paths(error, include_local_paths))
}

fn policy_explain_inner(
    args: PolicyExplainArgs,
    checked_at: DateTime<Utc>,
) -> Result<PolicyExplainReport> {
    let selected_profile = args.profile;
    let manifest = policy_manifest(selected_profile, checked_at);
    let docs_check = policy_docs_check(args.repo_root.as_deref())?;
    let docs_mirror_status = policy_explain_profile_docs_status(selected_profile, &docs_check);
    let docs_mirror_passed = policy_explain_profile_docs_passed(selected_profile, &docs_check);
    let tool_statuses =
        policy_explain_required_tool_statuses(&manifest.gates, args.include_local_paths);
    let missing_local_prerequisites =
        policy_explain_missing_prerequisites(&manifest.gates, &tool_statuses);
    let gates = manifest
        .gates
        .iter()
        .map(|gate| {
            let gate_docs_mirror_status =
                policy_explain_gate_docs_status(selected_profile, gate, &docs_check);
            policy_explain_gate(gate, &tool_statuses, &gate_docs_mirror_status)
        })
        .collect::<Vec<_>>();
    let docs_repo_root = args
        .include_local_paths
        .then(|| docs_check.repo_root.clone());

    Ok(PolicyExplainReport {
        schema: POLICY_EXPLAIN_SCHEMA,
        profile: manifest.profile,
        checked_at,
        manifest_schema: manifest.schema,
        gate_count: manifest.gates.len(),
        required_gate_count: manifest.gates.iter().filter(|gate| gate.required).count(),
        network_gate_count: manifest.gates.iter().filter(|gate| gate.network).count(),
        secret_gate_count: manifest.gates.iter().filter(|gate| gate.secrets).count(),
        docs_mirror: PolicyExplainDocsMirror {
            repo_root: docs_repo_root,
            status: docs_mirror_status,
            passed: docs_mirror_passed,
            blocks: docs_check
                .blocks
                .into_iter()
                .filter(|block| block.profiles.contains(&selected_profile))
                .map(|block| PolicyExplainDocsBlock {
                    status: policy_explain_block_status(block.passed),
                    error: policy_explain_doc_error(
                        block.error,
                        &block.path,
                        args.include_local_paths,
                    ),
                    path: block.path,
                    marker: block.marker,
                    profiles: block.profiles,
                    expected_commands: block.expected_commands,
                    actual_commands: block.actual_commands,
                })
                .collect(),
        },
        required_tools: tool_statuses,
        missing_local_prerequisites,
        gates,
    })
}

fn policy_explain_error_without_local_paths(
    error: anyhow::Error,
    include_local_paths: bool,
) -> anyhow::Error {
    if include_local_paths {
        error
    } else {
        anyhow::anyhow!("{}", policy_explain_redacted_error_message(&error))
    }
}

fn policy_explain_redacted_error_message(error: &anyhow::Error) -> String {
    let message = redact_local_paths_in_text(&format!("{error:#}"));
    format!("{message}; rerun with --include-local-paths for local path details")
}

fn redact_local_paths_in_text(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut redacted = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if is_unix_absolute_path_start(&chars, index)
            || is_windows_absolute_path_start(&chars, index)
        {
            let end = local_path_end(&chars, index);
            redacted.push_str("<local-path>");
            index = end;
        } else {
            redacted.push(chars[index]);
            index += 1;
        }
    }
    redacted
}

fn is_unix_absolute_path_start(chars: &[char], index: usize) -> bool {
    chars[index] == '/'
        && chars
            .get(index + 1)
            .is_some_and(|next| !next.is_whitespace())
        && local_path_start_boundary(chars, index)
}

fn local_path_start_boundary(chars: &[char], index: usize) -> bool {
    index == 0
        || matches!(
            chars.get(index - 1),
            Some(' ' | '\t' | '\n' | '\r' | '(' | '[' | '{' | '"' | '\'' | '`' | '=')
        )
}

fn is_windows_absolute_path_start(chars: &[char], index: usize) -> bool {
    is_windows_drive_absolute_path_start(chars, index)
        || is_windows_unc_or_device_path_start(chars, index)
}

fn is_windows_drive_absolute_path_start(chars: &[char], index: usize) -> bool {
    chars[index].is_ascii_alphabetic()
        && chars.get(index + 1) == Some(&':')
        && matches!(chars.get(index + 2), Some('\\' | '/'))
}

fn is_windows_unc_or_device_path_start(chars: &[char], index: usize) -> bool {
    chars[index] == '\\'
        && chars.get(index + 1) == Some(&'\\')
        && chars
            .get(index + 2)
            .is_some_and(|next| !next.is_whitespace())
}

fn local_path_end(chars: &[char], start: usize) -> usize {
    let mut index = start;
    while index < chars.len() {
        let current = chars[index];
        let next = chars.get(index + 1);
        if current.is_whitespace() || matches!(current, ',' | ';') {
            break;
        }
        if current == ':' && next.is_some_and(|next| next.is_whitespace()) {
            break;
        }
        if current == '.' && next.is_none_or(|next| next.is_whitespace()) {
            break;
        }
        if matches!(current, '"' | '\'' | '`' | ')' | ']' | '}') {
            break;
        }
        index += 1;
    }
    index
}

fn policy_explain_profile_docs_status(
    profile: PolicyProfile,
    docs_check: &PolicyDocsCheckResult,
) -> String {
    let relevant_blocks = docs_check
        .blocks
        .iter()
        .filter(|block| block.profiles.contains(&profile))
        .collect::<Vec<_>>();
    if relevant_blocks.is_empty() {
        "not_mirrored".to_string()
    } else if relevant_blocks.iter().all(|block| block.passed) {
        "current".to_string()
    } else {
        "stale_or_missing".to_string()
    }
}

fn policy_explain_profile_docs_passed(
    profile: PolicyProfile,
    docs_check: &PolicyDocsCheckResult,
) -> bool {
    docs_check
        .blocks
        .iter()
        .filter(|block| block.profiles.contains(&profile))
        .all(|block| block.passed)
}

fn policy_explain_gate_docs_status(
    profile: PolicyProfile,
    gate: &PolicyGate,
    docs_check: &PolicyDocsCheckResult,
) -> String {
    let (source_path, source_anchor) = gate
        .source
        .split_once('#')
        .map_or((gate.source.as_str(), None), |(path, anchor)| {
            (path, Some(anchor))
        });
    let matching_blocks = docs_check
        .blocks
        .iter()
        .filter(|block| {
            block.path == source_path
                && block.profiles.contains(&profile)
                && policy_doc_block_matches_source_anchor(block, source_anchor)
        })
        .collect::<Vec<_>>();

    if matching_blocks.is_empty() {
        "not_mirrored".to_string()
    } else if matching_blocks.iter().all(|block| block.passed) {
        "current".to_string()
    } else {
        "stale_or_missing".to_string()
    }
}

fn policy_doc_block_matches_source_anchor(
    block: &PolicyDocsBlockResult,
    source_anchor: Option<&str>,
) -> bool {
    let Some(source_anchor) = source_anchor else {
        return true;
    };
    policy_doc_block_source_anchor(block) == Some(source_anchor)
}

fn policy_doc_block_source_anchor(block: &PolicyDocsBlockResult) -> Option<&'static str> {
    match (block.path.as_str(), block.marker.as_str()) {
        ("docs/runbooks/validation.md", POLICY_DOCS_SMOKE_MARKER) => {
            Some("codex-dev-operating-layer")
        }
        ("docs/runbooks/validation.md", POLICY_DOCS_ALL_MARKER) => Some("full-local-gate"),
        _ => None,
    }
}

fn policy_explain_block_status(passed: bool) -> String {
    if passed {
        "current"
    } else {
        "stale_or_missing"
    }
    .to_string()
}

fn policy_explain_required_tool_statuses(
    gates: &[PolicyGate],
    include_local_paths: bool,
) -> Vec<PolicyExplainToolStatus> {
    gates
        .iter()
        .flat_map(|gate| gate.required_tools.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|name| {
            let path = find_executable_on_path(&name);
            PolicyExplainToolStatus {
                name,
                available: path.is_some(),
                path: include_local_paths.then_some(path).flatten(),
            }
        })
        .collect()
}

fn policy_explain_doc_error(
    error: Option<String>,
    path: &str,
    include_local_paths: bool,
) -> Option<String> {
    error.map(|message| {
        if include_local_paths {
            return message;
        }
        if message.starts_with("failed to read ") {
            let reason = message
                .rsplit_once(": ")
                .map(|(_, reason)| format!(": {reason}"))
                .unwrap_or_default();
            format!("failed to read {path}{reason}")
        } else {
            message
        }
    })
}

fn policy_explain_missing_prerequisites(
    gates: &[PolicyGate],
    tool_statuses: &[PolicyExplainToolStatus],
) -> Vec<PolicyExplainMissingPrerequisite> {
    let unavailable = tool_statuses
        .iter()
        .filter(|tool| !tool.available)
        .map(|tool| tool.name.as_str())
        .collect::<BTreeSet<_>>();
    if unavailable.is_empty() {
        return Vec::new();
    }

    let mut gate_ids_by_tool = BTreeMap::<String, Vec<String>>::new();
    for gate in gates {
        for tool in &gate.required_tools {
            if unavailable.contains(tool.as_str()) {
                gate_ids_by_tool
                    .entry(tool.clone())
                    .or_default()
                    .push(gate.id.clone());
            }
        }
    }

    gate_ids_by_tool
        .into_iter()
        .map(|(tool, gate_ids)| PolicyExplainMissingPrerequisite {
            detail: format!("required command `{tool}` was not found on PATH"),
            tool,
            gate_ids,
        })
        .collect()
}

fn policy_explain_gate(
    gate: &PolicyGate,
    tool_statuses: &[PolicyExplainToolStatus],
    docs_mirror_status: &str,
) -> PolicyExplainGate {
    let required_tools = gate
        .required_tools
        .iter()
        .filter_map(|name| {
            tool_statuses
                .iter()
                .find(|tool| tool.name == *name)
                .map(|tool| PolicyExplainToolStatus {
                    name: tool.name.clone(),
                    available: tool.available,
                    path: tool.path.clone(),
                })
        })
        .collect::<Vec<_>>();
    let missing_required_tools = required_tools
        .iter()
        .filter(|tool| !tool.available)
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();

    PolicyExplainGate {
        id: gate.id.clone(),
        name: gate.name.clone(),
        purpose: policy_gate_purpose(gate),
        source: gate.source.clone(),
        command: gate.command.clone(),
        command_display: render_command(&gate.command),
        working_directory: gate.working_directory.clone(),
        required: gate.required,
        required_tools,
        missing_required_tools,
        network: gate.network,
        network_posture: if gate.network {
            "requires_explicit_allow_network"
        } else {
            "local_only"
        }
        .to_string(),
        secrets: gate.secrets,
        secrets_posture: if gate.secrets {
            "requires_explicit_allow_secrets"
        } else {
            "no_secrets_required"
        }
        .to_string(),
        docs_mirror_status: docs_mirror_status.to_string(),
        expected_artifacts: policy_gate_expected_artifacts(gate),
        failure_interpretation: gate.failure_interpretation.clone(),
    }
}

fn policy_gate_purpose(gate: &PolicyGate) -> String {
    let consequence = gate
        .failure_interpretation
        .strip_prefix("Failure means ")
        .unwrap_or(gate.failure_interpretation.as_str());
    format!("Validate {}; {consequence}", gate.name)
}

fn policy_gate_expected_artifacts(gate: &PolicyGate) -> Vec<String> {
    if gate.command.iter().any(|part| part == "completions") {
        vec!["shell completion text on stdout".to_string()]
    } else if gate.command.iter().any(|part| part == "manpage") {
        vec!["roff manpage text on stdout".to_string()]
    } else if let Some(artifact) = policy_gate_install_smoke_artifact(gate) {
        vec![artifact]
    } else if command_contains_sequence(&gate.command, &["policy", "manifest"]) {
        vec!["policy gate manifest JSON on stdout".to_string()]
    } else if command_contains_sequence(&gate.command, &["policy", "explain"]) {
        vec!["policy_explain.v1 JSON on stdout".to_string()]
    } else if command_contains_sequence(&gate.command, &["policy", "docs-check"]) {
        vec!["policy docs-check JSON on stdout".to_string()]
    } else if command_contains_sequence(&gate.command, &["skills", "inventory"]) {
        vec!["skill_inventory.v1 JSON on stdout".to_string()]
    } else if command_contains_sequence(&gate.command, &["pr", "plan"]) {
        vec!["pr_control_plan.v1 JSON on stdout".to_string()]
    } else if gate.command.iter().any(|part| part == "--list") {
        vec!["catalog listing on stdout".to_string()]
    } else {
        vec!["stdout/stderr validation output; no tracked artifact expected".to_string()]
    }
}

fn policy_gate_install_smoke_artifact(gate: &PolicyGate) -> Option<String> {
    let is_install_smoke = gate.id.starts_with("cargo-install-")
        || gate
            .command
            .iter()
            .any(|part| part == "install-smoke" || part.contains("target/codex-dev-install-smoke"));
    if !is_install_smoke {
        return None;
    }

    let binary = gate
        .id
        .strip_prefix("cargo-install-")
        .and_then(|name| name.strip_suffix("-smoke"))
        .unwrap_or("<binary>");
    Some(format!(
        "isolated install root under target/codex-dev-install-smoke/{binary} on filesystem"
    ))
}

fn command_contains_sequence(command: &[String], sequence: &[&str]) -> bool {
    if sequence.is_empty() || command.len() < sequence.len() {
        return false;
    }
    command.windows(sequence.len()).any(|window| {
        window
            .iter()
            .map(String::as_str)
            .eq(sequence.iter().copied())
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
    let expected_commands = policy_doc_block_expected_commands(spec.kind);

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

fn policy_doc_block_expected_commands(kind: PolicyDocBlockKind) -> Vec<String> {
    policy_doc_block_profiles(kind)
        .iter()
        .flat_map(|profile| {
            [
                policy_manifest_command(*profile),
                policy_explain_command(*profile),
            ]
        })
        .collect()
}

fn policy_manifest_command(profile: PolicyProfile) -> String {
    format!("cargo run -q -p codex-dev -- --json policy manifest --profile {profile}")
}

fn policy_explain_command(profile: PolicyProfile) -> String {
    format!("cargo run -q -p codex-dev -- --json policy explain --profile {profile}")
}

pub fn pr_control_plan(
    repository: String,
    number: u64,
    generated_at: DateTime<Utc>,
) -> Result<PrControlPlan> {
    let (owner, name) = parse_github_repository(&repository)?;
    let owner_arg = format!("owner={owner}");
    let name_arg = format!("name={name}");
    let number_arg = format!("number={number}");
    let reviews_path = format!("repos/{owner}/{name}/pulls/{number}/reviews?per_page=100");
    let review_comments_path = format!("repos/{owner}/{name}/pulls/{number}/comments?per_page=100");
    let review_threads_query_arg = format!("query={PR_REVIEW_THREADS_QUERY}");

    Ok(PrControlPlan {
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
                    GH_PR_VIEW_JSON_FIELDS,
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
                ["gh", "api", "--paginate", "--slurp", &reviews_path],
            ),
            pr_control_command(
                "gh-review-comments",
                "GitHub REST review comments",
                ["gh", "api", "--paginate", "--slurp", &review_comments_path],
            ),
            pr_control_command(
                "gh-review-threads",
                "GitHub GraphQL review-thread state",
                [
                    "gh",
                    "api",
                    "graphql",
                    "--paginate",
                    "--slurp",
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
                "codex-dev-pr-review-start",
                "First-class hosted review worklist",
                [
                    "codex-dev",
                    "--json",
                    "pr",
                    "review",
                    "start",
                    "--repo",
                    &repository,
                    "--number",
                    &number.to_string(),
                    "--fresh",
                ],
            ),
            pr_control_command_with_manual_input(
                "codex-dev-commit-plan",
                "Scoped semantic Conventional Commit grouping plan",
                [
                    "codex-dev",
                    "--json",
                    "commit",
                    "plan",
                    "--worklist",
                    "<pr-review-worklist.json>",
                ],
                "replace <pr-review-worklist.json> with the path emitted by `codex-dev pr review start`",
            ),
            pr_control_command_with_manual_input(
                "codex-dev-pr-review-closeout",
                "Batch resolve verified fixed hosted review threads",
                [
                    "codex-dev",
                    "--json",
                    "pr",
                    "review",
                    "closeout",
                    "--repo",
                    &repository,
                    "--number",
                    &number.to_string(),
                    "--worklist",
                    "<pr-review-worklist.json>",
                    "--expected-head-sha",
                    "<pushed-head-sha>",
                    "--commit",
                    "<semantic-fix-commit-sha>",
                    "--validation-command",
                    "<passed-validation-command>",
                ],
                "replace placeholders after fixes are validated, committed, pushed, and fresh PR head state is known; add --apply only for hosted closeout",
            ),
        ],
    })
}

pub fn run_pr_agent_state(
    args: PrAgentArgs,
    checked_at: DateTime<Utc>,
) -> Result<PrAgentStateReport> {
    let (owner, name) = parse_github_repository(&args.repo)?;
    ensure_regular_contract_files(&args.capsule)?;
    let validation = validate_capsule(&args.capsule)?;
    if !validation.valid {
        bail!(
            "invalid capsule at {}: {}",
            args.capsule.display(),
            validation.errors.join("; ")
        );
    }

    let output_dir = prepare_pr_agent_output_dir(&args.capsule, checked_at)?;

    let agent_command = render_pr_agent_command(&args, checked_at);
    let mut diagnostics = Vec::new();
    let mut sources = Vec::new();

    for spec in pr_agent_source_specs(&args.repo, owner, name, args.number) {
        let capture = capture_pr_agent_source(&args, &spec, &output_dir, checked_at)?;
        sources.push(capture.source.clone());
        diagnostics.extend(capture.diagnostics);

        if capture.source.status != PrAgentSourceStatus::Captured {
            continue;
        }

        if let Some(source_kind) = spec.source_kind {
            let record_result = record_pr_snapshot(
                PrRecordArgs {
                    capsule: args.capsule.clone(),
                    source: capture.path.clone(),
                    source_kind,
                    repository: Some(args.repo.clone()),
                    number: Some(args.number),
                    retrieved_at: Some(capture.source.retrieved_at),
                    source_command: Some(render_command(&spec.command)),
                    command: Some(agent_command.clone()),
                },
                checked_at,
            );
            if let Err(error) = record_result {
                diagnostics.push(PrAgentDiagnostic {
                    source: spec.id.clone(),
                    severity: PrAgentSeverity::Error,
                    message: format!("failed to normalize captured PR source: {error:#}"),
                    command: Some(render_command(&spec.command)),
                    exit_code: capture.source.exit_code,
                    at: checked_at,
                });
            }
        } else if spec.id == "gh-rate-limit" {
            diagnostics.extend(rate_limit_diagnostics(&capture.path, &spec, checked_at)?);
        }
    }

    let pr = pr_status(&args.capsule)?.pr;
    if sources.iter().any(|source| {
        source.id == "gh-review-threads" && source.status == PrAgentSourceStatus::Captured
    }) && !pr.review_threads.authoritative
    {
        diagnostics.push(PrAgentDiagnostic {
            source: "gh-review-threads".to_string(),
            severity: PrAgentSeverity::Warning,
            message: "GitHub review-thread pagination did not reach a final page; review-thread state is not authoritative".to_string(),
            command: sources
                .iter()
                .find(|source| source.id == "gh-review-threads")
                .map(|source| source.command.clone()),
            exit_code: sources
                .iter()
                .find(|source| source.id == "gh-review-threads")
                .and_then(|source| source.exit_code),
            at: checked_at,
        });
    }
    let actions = recommend_pr_agent_actions(&pr, &diagnostics);
    let report = PrAgentStateReport {
        schema: PR_AGENT_STATE_SCHEMA.to_string(),
        repository: args.repo.clone(),
        number: args.number,
        checked_at,
        dry_run: true,
        pr,
        sources,
        diagnostics,
        actions,
    };
    let report_path = args.capsule.join("pr-agent-state.json");
    ensure_pr_agent_report_path_safe(&report_path)?;
    write_json(report_path, &report)?;

    append_evidence(AppendEvidenceArgs {
        capsule: args.capsule.clone(),
        record: EvidenceRecord {
            schema: EVIDENCE_SCHEMA.to_string(),
            kind: EvidenceKind::Decision,
            at: checked_at,
            summary: format!(
                "PR agent dry-run state recorded for {}#{}; {} source(s), {} diagnostic(s), {} action(s)",
                report.repository,
                report.number,
                report.sources.len(),
                report.diagnostics.len(),
                report.actions.len()
            ),
            command: Some(agent_command),
            exit_code: Some(
                if report
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.severity == PrAgentSeverity::Error)
                {
                    1
                } else {
                    0
                },
            ),
            source_ids: Vec::new(),
            actor: None,
            tool: Some("codex-dev".to_string()),
            confidence: None,
            residual_risk: report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == PrAgentSeverity::Error)
                .then(|| {
                    "one or more hosted-state sources failed to capture or normalize".to_string()
                }),
            artifacts: vec!["pr.json".to_string(), "pr-agent-state.json".to_string()],
        },
    })?;

    Ok(report)
}

pub fn run_pr_readiness_loop(
    args: PrReadinessArgs,
    generated_at: DateTime<Utc>,
) -> Result<PrAgentReadinessReport> {
    if args.poll_attempts == 0 {
        bail!("--poll-attempts must be at least 1");
    }
    if args.apply && args.source_dir.is_some() {
        bail!(
            "--source-dir is only allowed for dry-run readiness evaluation; --apply must capture live state"
        );
    }
    parse_github_repository(&args.repo)?;
    ensure_regular_contract_files(&args.capsule)?;
    let validation = validate_capsule(&args.capsule)?;
    if !validation.valid {
        bail!(
            "invalid capsule at {}: {}",
            args.capsule.display(),
            validation.errors.join("; ")
        );
    }

    let mut attempts = Vec::new();
    let mut last_state = None;
    for attempt in 1..=args.poll_attempts {
        if attempt > 1 && args.poll_interval_seconds > 0 {
            sleep(Duration::from_secs(args.poll_interval_seconds));
        }
        let checked_at =
            readiness_attempt_checked_at(generated_at, attempt, args.poll_interval_seconds)?;
        let state = run_pr_agent_state(
            PrAgentArgs {
                capsule: args.capsule.clone(),
                repo: args.repo.clone(),
                number: args.number,
                checked_at: Some(checked_at),
                source_dir: args.source_dir.clone(),
            },
            checked_at,
        )?;
        let attempt_report = evaluate_pr_readiness_attempt(attempt, &state)?;
        let terminal = !matches!(attempt_report.status, PrAgentReadinessStatus::Waiting);
        attempts.push(attempt_report);
        last_state = Some(state);
        if terminal {
            break;
        }
    }

    let final_attempt = attempts
        .last()
        .context("readiness loop did not record any attempts")?;
    let state = last_state
        .as_ref()
        .context("readiness loop did not retain the latest PR state")?;
    let mut actions = Vec::new();

    if args.rerun_failed {
        actions.extend(plan_or_apply_failed_check_reruns(
            &args,
            state,
            final_attempt,
        )?);
    }
    if args.merge {
        actions.push(plan_or_apply_merge(
            &args,
            state,
            final_attempt,
            generated_at,
        )?);
    }

    let final_status = if actions
        .iter()
        .any(|action| action.status == PrAgentReadinessActionStatus::Failed)
    {
        PrAgentReadinessStatus::Blocked
    } else if actions.iter().any(|action| {
        action.kind == "merge" && action.status == PrAgentReadinessActionStatus::Applied
    }) {
        PrAgentReadinessStatus::Merged
    } else {
        final_attempt.status
    };
    let report_path = args.capsule.join("pr-readiness.json");
    let markdown_path = args.capsule.join("pr-readiness.md");
    ensure_pr_agent_report_path_safe(&report_path)?;
    ensure_pr_agent_report_path_safe(&markdown_path)?;
    let mut report = PrAgentReadinessReport {
        schema: PR_AGENT_READINESS_SCHEMA.to_string(),
        repository: args.repo.clone(),
        number: args.number,
        generated_at,
        apply_requested: args.apply,
        rerun_failed_requested: args.rerun_failed,
        merge_requested: args.merge,
        ready: matches!(
            final_status,
            PrAgentReadinessStatus::Ready | PrAgentReadinessStatus::Merged
        ),
        final_status,
        attempts,
        actions,
        markdown_path: markdown_path.display().to_string(),
        report_path: report_path.display().to_string(),
    };
    let markdown = render_pr_readiness_markdown(&report);
    write_json(report_path.clone(), &report)?;
    fs::write(&markdown_path, markdown)
        .with_context(|| format!("failed to write {}", markdown_path.display()))?;

    append_evidence(AppendEvidenceArgs {
        capsule: args.capsule.clone(),
        record: EvidenceRecord {
            schema: EVIDENCE_SCHEMA.to_string(),
            kind: EvidenceKind::Decision,
            at: generated_at,
            summary: format!(
                "PR readiness for {}#{} finished as {:?} after {} attempt(s)",
                report.repository,
                report.number,
                report.final_status,
                report.attempts.len()
            ),
            command: Some(render_pr_readiness_command(&args)),
            exit_code: Some(
                if report
                    .actions
                    .iter()
                    .any(|action| action.status == PrAgentReadinessActionStatus::Failed)
                {
                    1
                } else {
                    0
                },
            ),
            source_ids: Vec::new(),
            actor: None,
            tool: Some("codex-dev".to_string()),
            confidence: None,
            residual_risk: readiness_residual_risk(&report),
            artifacts: vec![
                "pr.json".to_string(),
                "pr-agent-state.json".to_string(),
                "pr-readiness.json".to_string(),
                "pr-readiness.md".to_string(),
            ],
        },
    })?;

    report.report_path = report_path.display().to_string();
    report.markdown_path = markdown_path.display().to_string();
    Ok(report)
}

fn readiness_attempt_checked_at(
    generated_at: DateTime<Utc>,
    attempt: u64,
    poll_interval_seconds: u64,
) -> Result<DateTime<Utc>> {
    let zero_based_attempt = attempt
        .checked_sub(1)
        .context("readiness poll attempts are one-indexed")?;
    let step_seconds = poll_interval_seconds.max(1);
    let offset_seconds = zero_based_attempt
        .checked_mul(step_seconds)
        .context("readiness poll timestamp offset overflowed")?;
    let offset_seconds = i64::try_from(offset_seconds)
        .context("readiness poll timestamp offset exceeds supported range")?;
    generated_at
        .checked_add_signed(TimeDelta::seconds(offset_seconds))
        .context("readiness poll timestamp exceeds supported range")
}

fn evaluate_pr_readiness_attempt(
    attempt: u64,
    state: &PrAgentStateReport,
) -> Result<PrAgentReadinessAttempt> {
    let mut blockers = Vec::new();
    let mut wait_reasons = Vec::new();
    let mut warnings = Vec::new();
    let mut failing_checks = Vec::new();
    let mut pending_checks = Vec::new();
    let (active_review_comments, outdated_review_comments) = review_comment_counts(state)?;

    for diagnostic in &state.diagnostics {
        if diagnostic.severity == PrAgentSeverity::Error {
            blockers.push(format!(
                "state source {} failed: {}",
                diagnostic.source, diagnostic.message
            ));
        }
    }

    let pr = &state.pr;
    match pr.state.as_str() {
        "merged" => {
            return readiness_attempt(
                attempt,
                state,
                ReadinessAttemptParts {
                    status: PrAgentReadinessStatus::Merged,
                    blockers,
                    wait_reasons,
                    warnings,
                    failing_checks,
                    pending_checks,
                    active_review_comments,
                    outdated_review_comments,
                },
            );
        }
        "closed" => {
            blockers.push("pull request is closed without a merge".to_string());
            return readiness_attempt(
                attempt,
                state,
                ReadinessAttemptParts {
                    status: PrAgentReadinessStatus::Stopped,
                    blockers,
                    wait_reasons,
                    warnings,
                    failing_checks,
                    pending_checks,
                    active_review_comments,
                    outdated_review_comments,
                },
            );
        }
        "open" | "draft" => {}
        other => wait_reasons.push(format!("pull request state is {other}; expected open")),
    }

    if pr.is_draft.unwrap_or(pr.state == "draft") {
        blockers.push("pull request is still draft".to_string());
    }
    if missing_text(pr.head_sha.as_deref()) {
        blockers.push("PR head SHA was not captured".to_string());
    }
    if missing_text(pr.head_ref_name.as_deref()) {
        blockers.push("PR head branch name was not captured".to_string());
    }
    if missing_text(pr.base_ref_name.as_deref()) {
        blockers.push("PR base branch name was not captured".to_string());
    }
    if missing_text(pr.base_ref_oid.as_deref()) {
        blockers.push("PR base branch OID was not captured".to_string());
    }

    match pr.mergeable.as_deref() {
        Some("mergeable" | "clean") => {}
        Some("conflicting" | "dirty") => {
            blockers.push("GitHub reports merge conflicts".to_string());
        }
        Some("unknown") | None => {
            wait_reasons.push("GitHub mergeability is not known yet".to_string());
        }
        Some(other) => blockers.push(format!("GitHub mergeability is {other}")),
    }

    match pr.merge_state_status.as_deref() {
        Some("clean" | "has_hooks") => {}
        Some("behind") => blockers.push("head branch is behind the base branch".to_string()),
        Some("blocked" | "dirty" | "draft") => {
            blockers.push(format!(
                "GitHub merge state is {}",
                pr.merge_state_status.as_deref().unwrap()
            ));
        }
        Some("unknown") => wait_reasons.push("GitHub merge state is not known yet".to_string()),
        None => blockers.push("GitHub merge state was not captured".to_string()),
        Some("unstable") => warnings.push(
            "GitHub merge state is unstable; check-level evidence determines the blocker"
                .to_string(),
        ),
        Some(other) => blockers.push(format!("GitHub merge state is {other}")),
    }

    for check in &pr.checks {
        let status = check.status.to_ascii_lowercase();
        let conclusion = check.conclusion.as_deref().map(str::to_ascii_lowercase);
        if check_is_failure(status.as_str(), conclusion.as_deref()) {
            let readiness_check = readiness_check_from_check(check, &state.repository);
            blockers.push(format!(
                "check {} failed; inspect {}",
                check.name, readiness_check.diagnostic_command
            ));
            failing_checks.push(readiness_check);
        } else if check_is_pending(status.as_str()) {
            let readiness_check = readiness_check_from_check(check, &state.repository);
            wait_reasons.push(format!("check {} is still {}", check.name, check.status));
            pending_checks.push(readiness_check);
        } else if status == "completed" && conclusion.is_none() {
            let readiness_check = readiness_check_from_check(check, &state.repository);
            wait_reasons.push(format!(
                "check {} completed without an explicit conclusion",
                check.name
            ));
            pending_checks.push(readiness_check);
        } else if status == "completed" && !check_is_success(status.as_str(), conclusion.as_deref())
        {
            let readiness_check = readiness_check_from_check(check, &state.repository);
            blockers.push(format!(
                "check {} completed with unsupported conclusion {}; inspect {}",
                check.name,
                conclusion.as_deref().unwrap_or("none"),
                readiness_check.diagnostic_command
            ));
            failing_checks.push(readiness_check);
        } else if !check_is_success(status.as_str(), conclusion.as_deref()) {
            let readiness_check = readiness_check_from_check(check, &state.repository);
            wait_reasons.push(format!(
                "check {} has unrecognized status {}",
                check.name, check.status
            ));
            pending_checks.push(readiness_check);
        }
    }
    if pr.checks.is_empty() {
        blockers.push("no PR checks were captured; cannot prove CI passed".to_string());
    }

    if !pr.review_threads.authoritative {
        blockers.push("hosted review-thread state is not authoritative".to_string());
    } else if pr.review_threads.unresolved > 0 {
        blockers.push(format!(
            "{} hosted review thread(s) remain unresolved",
            pr.review_threads.unresolved
        ));
    }

    match pr.review_decision.as_deref() {
        Some("approved") => {}
        Some("changes_requested")
            if pr.review_threads.authoritative && pr.review_threads.unresolved == 0 =>
        {
            warnings.push(
                "reviewDecision is changes_requested but thread-level state is clean; treating reviewDecision as stale"
                    .to_string(),
            );
        }
        Some("changes_requested") => {
            blockers.push("latest review decision is changes_requested".to_string());
        }
        Some("review_required") => {
            wait_reasons.push("required approving review has not landed yet".to_string());
        }
        Some("commented") => warnings.push(
            "latest review decision is commented; thread-level state determines readiness"
                .to_string(),
        ),
        None => warnings.push(
            "GitHub did not report a reviewDecision; branch protection may not require one"
                .to_string(),
        ),
        Some(other) => warnings.push(format!("GitHub reviewDecision is {other}")),
    }

    if outdated_review_comments > 0 {
        warnings.push(format!(
            "{outdated_review_comments} outdated review comment(s) captured; hosted review-thread state remains the readiness authority"
        ));
    }
    if active_review_comments > 0 && state.pr.review_threads.unresolved == 0 {
        warnings.push(format!(
            "{active_review_comments} active review comment(s) captured, but hosted review threads are resolved"
        ));
    }

    let status = if !blockers.is_empty() {
        PrAgentReadinessStatus::Blocked
    } else if !wait_reasons.is_empty() {
        PrAgentReadinessStatus::Waiting
    } else {
        PrAgentReadinessStatus::Ready
    };

    readiness_attempt(
        attempt,
        state,
        ReadinessAttemptParts {
            status,
            blockers,
            wait_reasons,
            warnings,
            failing_checks,
            pending_checks,
            active_review_comments,
            outdated_review_comments,
        },
    )
}

struct ReadinessAttemptParts {
    status: PrAgentReadinessStatus,
    blockers: Vec<String>,
    wait_reasons: Vec<String>,
    warnings: Vec<String>,
    failing_checks: Vec<PrAgentReadinessCheck>,
    pending_checks: Vec<PrAgentReadinessCheck>,
    active_review_comments: u64,
    outdated_review_comments: u64,
}

fn readiness_attempt(
    attempt: u64,
    state: &PrAgentStateReport,
    parts: ReadinessAttemptParts,
) -> Result<PrAgentReadinessAttempt> {
    Ok(PrAgentReadinessAttempt {
        attempt,
        checked_at: state.checked_at,
        status: parts.status,
        pr: state.pr.clone(),
        blockers: parts.blockers,
        wait_reasons: parts.wait_reasons,
        warnings: parts.warnings,
        failing_checks: parts.failing_checks,
        pending_checks: parts.pending_checks,
        active_review_comments: parts.active_review_comments,
        outdated_review_comments: parts.outdated_review_comments,
        diagnostics: state.diagnostics.clone(),
    })
}

fn check_is_failure(status: &str, conclusion: Option<&str>) -> bool {
    matches!(
        conclusion,
        Some("failure" | "error" | "cancelled" | "canceled" | "timed_out" | "action_required")
    ) || matches!(
        status,
        "failure" | "failed" | "error" | "cancelled" | "canceled" | "timed_out"
    )
}

fn check_is_success(status: &str, conclusion: Option<&str>) -> bool {
    status == "completed" && matches!(conclusion, Some("success" | "neutral" | "skipped"))
}

fn check_is_pending(status: &str) -> bool {
    matches!(
        status,
        "pending" | "queued" | "in_progress" | "requested" | "waiting" | "expected"
    )
}

fn missing_text(value: Option<&str>) -> bool {
    value.map(str::trim).unwrap_or_default().is_empty()
}

fn readiness_check_from_check(
    check: &codex_dev_core::CheckRecord,
    repository: &str,
) -> PrAgentReadinessCheck {
    let run_id = check
        .url
        .as_deref()
        .and_then(|url| extract_github_actions_run_id_for_repo(url, repository));
    let diagnostic_command = if let Some(run_id) = run_id {
        format!("gh run view {run_id} --log-failed")
    } else if let Some(url) = &check.url {
        url.clone()
    } else {
        format!(
            "gh pr checks --json name,state,link --jq '.[] | select(.name == {:?})'",
            check.name
        )
    };
    PrAgentReadinessCheck {
        name: check.name.clone(),
        status: check.status.clone(),
        conclusion: check.conclusion.clone(),
        url: check.url.clone(),
        run_id,
        diagnostic_command,
    }
}

fn extract_github_actions_run_id_for_repo(url: &str, repository: &str) -> Option<u64> {
    let lower_url = url.to_ascii_lowercase();
    let lower_repository = repository.to_ascii_lowercase();
    let html_marker = format!("github.com/{lower_repository}/actions/runs/");
    let api_marker = format!("api.github.com/repos/{lower_repository}/actions/runs/");
    let marker = if let Some(start) = lower_url.find(&html_marker) {
        Some((start, html_marker))
    } else {
        lower_url.find(&api_marker).map(|start| (start, api_marker))
    }?;
    let start = marker.0 + marker.1.len();
    let digits = lower_url[start..]
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    digits.parse().ok()
}

fn review_comment_counts(state: &PrAgentStateReport) -> Result<(u64, u64)> {
    let Some(path) = source_path(state, "gh-review-comments") else {
        return Ok((0, 0));
    };
    let value = read_json::<Value>(&path)?;
    let mut active = 0;
    let mut outdated = 0;
    count_review_comments(&value, &mut active, &mut outdated);
    Ok((active, outdated))
}

fn count_review_comments(value: &Value, active: &mut u64, outdated: &mut u64) {
    match value {
        Value::Array(values) => {
            for value in values {
                count_review_comments(value, active, outdated);
            }
        }
        Value::Object(map) if map.get("body").is_some() || map.get("id").is_some() => {
            if map
                .get("outdated")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                *outdated += 1;
            } else {
                *active += 1;
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                count_review_comments(value, active, outdated);
            }
        }
        _ => {}
    }
}

fn plan_or_apply_failed_check_reruns(
    args: &PrReadinessArgs,
    state: &PrAgentStateReport,
    attempt: &PrAgentReadinessAttempt,
) -> Result<Vec<PrAgentReadinessAction>> {
    let mut actions = Vec::new();
    if attempt.failing_checks.is_empty() {
        actions.push(PrAgentReadinessAction {
            id: "rerun-failed-checks".to_string(),
            kind: "rerun_failed_jobs".to_string(),
            status: PrAgentReadinessActionStatus::Skipped,
            reason: "no failed checks were present in the final readiness attempt".to_string(),
            command: Vec::new(),
            exit_code: None,
            stdout: None,
            stderr: None,
        });
        return Ok(actions);
    }
    let mut run_ids = BTreeSet::new();
    for check in &attempt.failing_checks {
        if let Some(run_id) = check.run_id {
            run_ids.insert(run_id);
        }
    }
    if run_ids.is_empty() {
        actions.push(PrAgentReadinessAction {
            id: "rerun-failed-checks".to_string(),
            kind: "rerun_failed_jobs".to_string(),
            status: PrAgentReadinessActionStatus::Skipped,
            reason: "failed checks did not expose GitHub Actions run ids in their URLs".to_string(),
            command: Vec::new(),
            exit_code: None,
            stdout: None,
            stderr: None,
        });
        return Ok(actions);
    }

    for run_id in run_ids {
        let action_args = PrAgentActionArgs {
            capsule: args.capsule.clone(),
            repo: args.repo.clone(),
            number: args.number,
            plan_id: format!("readiness-rerun-{run_id}"),
            action: PrAgentHostedActionKind::RerunFailedJobs,
            apply: args.apply,
            body: None,
            body_file: None,
            review_comment_id: None,
            thread_id: None,
            labels: Vec::new(),
            run_id: Some(run_id),
            checked_at: Some(next_state_timestamp(state.checked_at)),
            source_dir: None,
        };
        if args.apply {
            let action_checked_at = action_args
                .checked_at
                .expect("readiness rerun checked_at should be set");
            let action_report = run_pr_agent_hosted_action(action_args, action_checked_at)?;
            let execution = action_report.execution.as_ref();
            actions.push(PrAgentReadinessAction {
                id: action_report.plan_id,
                kind: "rerun_failed_jobs".to_string(),
                status: execution
                    .map(|execution| readiness_action_status(execution.status))
                    .unwrap_or(PrAgentReadinessActionStatus::Failed),
                reason: "apply-gated rerun of failed GitHub Actions jobs".to_string(),
                command: execution
                    .map(|execution| execution.command.clone())
                    .unwrap_or(action_report.action.command),
                exit_code: execution.and_then(|execution| execution.exit_code),
                stdout: execution.and_then(|execution| execution.stdout.clone()),
                stderr: execution.and_then(|execution| execution.stderr.clone()),
            });
        } else {
            let (owner, name) = parse_github_repository(&args.repo)?;
            actions.push(PrAgentReadinessAction {
                id: format!("readiness-rerun-{run_id}"),
                kind: "rerun_failed_jobs".to_string(),
                status: PrAgentReadinessActionStatus::Planned,
                reason:
                    "rerun requires --apply; workflow run head SHA is rechecked before execution"
                        .to_string(),
                command: vec![
                    "gh".to_string(),
                    "api".to_string(),
                    "--method".to_string(),
                    "POST".to_string(),
                    format!("repos/{owner}/{name}/actions/runs/{run_id}/rerun-failed-jobs"),
                ],
                exit_code: None,
                stdout: None,
                stderr: None,
            });
        }
    }
    Ok(actions)
}

fn readiness_action_status(status: PrAgentHostedActionStatus) -> PrAgentReadinessActionStatus {
    match status {
        PrAgentHostedActionStatus::Applied => PrAgentReadinessActionStatus::Applied,
        PrAgentHostedActionStatus::SkippedDuplicate => PrAgentReadinessActionStatus::Skipped,
        PrAgentHostedActionStatus::Failed => PrAgentReadinessActionStatus::Failed,
    }
}

fn plan_or_apply_merge(
    args: &PrReadinessArgs,
    state: &PrAgentStateReport,
    attempt: &PrAgentReadinessAttempt,
    generated_at: DateTime<Utc>,
) -> Result<PrAgentReadinessAction> {
    if attempt.status != PrAgentReadinessStatus::Ready {
        return Ok(PrAgentReadinessAction {
            id: "merge-pr".to_string(),
            kind: "merge".to_string(),
            status: PrAgentReadinessActionStatus::Skipped,
            reason: format!(
                "merge requested but final readiness status is {:?}",
                attempt.status
            ),
            command: merge_command(args, &state.pr).unwrap_or_default(),
            exit_code: None,
            stdout: None,
            stderr: None,
        });
    }
    let command = merge_command(args, &state.pr)?;
    if !args.apply {
        return Ok(PrAgentReadinessAction {
            id: "merge-pr".to_string(),
            kind: "merge".to_string(),
            status: PrAgentReadinessActionStatus::Planned,
            reason: "merge requires --apply and a ready final state".to_string(),
            command,
            exit_code: None,
            stdout: None,
            stderr: None,
        });
    }
    let refresh_checked_at = attempt
        .checked_at
        .checked_add_signed(TimeDelta::seconds(1))
        .unwrap_or_else(Utc::now);
    let refreshed_state = run_pr_agent_state(
        PrAgentArgs {
            capsule: args.capsule.clone(),
            repo: args.repo.clone(),
            number: args.number,
            checked_at: Some(refresh_checked_at),
            source_dir: None,
        },
        refresh_checked_at,
    )?;
    let refreshed_attempt =
        evaluate_pr_readiness_attempt(attempt.attempt.saturating_add(1), &refreshed_state)?;
    if refreshed_attempt.status != PrAgentReadinessStatus::Ready {
        let reason = refreshed_attempt
            .blockers
            .first()
            .or_else(|| refreshed_attempt.wait_reasons.first())
            .cloned()
            .unwrap_or_else(|| "fresh PR state is not ready".to_string());
        return Ok(PrAgentReadinessAction {
            id: "merge-pr".to_string(),
            kind: "merge".to_string(),
            status: PrAgentReadinessActionStatus::Failed,
            reason: format!(
                "pre-merge readiness refresh returned {:?}; merge was not executed: {reason}",
                refreshed_attempt.status
            ),
            command,
            exit_code: None,
            stdout: None,
            stderr: None,
        });
    }
    let command = merge_command(args, &refreshed_state.pr)?;
    let output = run_hosted_command(&command)?;
    Ok(PrAgentReadinessAction {
        id: "merge-pr".to_string(),
        kind: "merge".to_string(),
        status: if output.exit_code == Some(0) {
            PrAgentReadinessActionStatus::Applied
        } else {
            PrAgentReadinessActionStatus::Failed
        },
        reason: format!(
            "apply-gated {} merge executed at {}",
            args.merge_method.as_str(),
            generated_at.to_rfc3339_opts(SecondsFormat::Secs, true)
        ),
        command,
        exit_code: output.exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

fn merge_command(args: &PrReadinessArgs, pr: &PrEvidence) -> Result<Vec<String>> {
    let Some(head_sha) = pr.head_sha.as_deref() else {
        bail!("cannot plan merge because PR head SHA was not captured");
    };
    let mut command = vec![
        "gh".to_string(),
        "pr".to_string(),
        "merge".to_string(),
        args.number.to_string(),
        "--repo".to_string(),
        args.repo.clone(),
        args.merge_method.flag().to_string(),
        "--match-head-commit".to_string(),
        head_sha.to_string(),
    ];
    if args.delete_branch {
        command.push("--delete-branch".to_string());
    }
    if let Some(subject) = &args.merge_subject {
        command.push("--subject".to_string());
        command.push(subject.clone());
    }
    if let Some(body) = &args.merge_body {
        command.push("--body".to_string());
        command.push(body.clone());
    }
    Ok(command)
}

fn render_pr_readiness_markdown(report: &PrAgentReadinessReport) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# PR Readiness: {}#{}\n\n",
        report.repository, report.number
    ));
    markdown.push_str(&format!("- Status: {:?}\n", report.final_status));
    markdown.push_str(&format!("- Ready: {}\n", report.ready));
    markdown.push_str(&format!("- Attempts: {}\n", report.attempts.len()));
    markdown.push_str(&format!("- Apply requested: {}\n", report.apply_requested));
    markdown.push_str(&format!(
        "- Rerun failed requested: {}\n",
        report.rerun_failed_requested
    ));
    markdown.push_str(&format!(
        "- Merge requested: {}\n\n",
        report.merge_requested
    ));
    if let Some(final_attempt) = report.attempts.last() {
        markdown.push_str("## Final Attempt\n\n");
        markdown.push_str(&format!("- Checked at: {}\n", final_attempt.checked_at));
        markdown.push_str(&format!("- PR state: {}\n", final_attempt.pr.state));
        markdown.push_str(&format!(
            "- Mergeable: {}\n",
            final_attempt.pr.mergeable.as_deref().unwrap_or("unknown")
        ));
        markdown.push_str(&format!(
            "- Merge state: {}\n",
            final_attempt
                .pr
                .merge_state_status
                .as_deref()
                .unwrap_or("unknown")
        ));
        markdown.push_str(&format!(
            "- Review threads: {} unresolved, authoritative={}\n",
            final_attempt.pr.review_threads.unresolved,
            final_attempt.pr.review_threads.authoritative
        ));
        markdown.push_str(&format!(
            "- Review comments: {} active, {} outdated\n\n",
            final_attempt.active_review_comments, final_attempt.outdated_review_comments
        ));
        markdown.push_str(&markdown_list("Blockers", &final_attempt.blockers));
        markdown.push_str(&markdown_list("Wait Reasons", &final_attempt.wait_reasons));
        markdown.push_str(&markdown_list("Warnings", &final_attempt.warnings));
        if !final_attempt.failing_checks.is_empty() {
            markdown.push_str("## Failing Checks\n\n");
            for check in &final_attempt.failing_checks {
                markdown.push_str(&format!(
                    "- {}: {} / {}; inspect with `{}`\n",
                    check.name,
                    check.status,
                    check.conclusion.as_deref().unwrap_or("none"),
                    check.diagnostic_command
                ));
            }
            markdown.push('\n');
        }
    }
    if !report.actions.is_empty() {
        markdown.push_str("## Actions\n\n");
        for action in &report.actions {
            markdown.push_str(&format!(
                "- {} ({:?}): `{}`\n",
                action.id,
                action.status,
                render_command(&action.command)
            ));
        }
        markdown.push('\n');
    }
    markdown
}

fn markdown_list(title: &str, values: &[String]) -> String {
    if values.is_empty() {
        return String::new();
    }
    let mut markdown = format!("## {title}\n\n");
    for value in values {
        markdown.push_str(&format!("- {value}\n"));
    }
    markdown.push('\n');
    markdown
}

fn readiness_residual_risk(report: &PrAgentReadinessReport) -> Option<String> {
    report
        .attempts
        .last()
        .and_then(|attempt| match report.final_status {
            PrAgentReadinessStatus::Ready | PrAgentReadinessStatus::Merged => None,
            PrAgentReadinessStatus::Blocked => {
                Some(format!("{} blocker(s) remain", attempt.blockers.len()))
            }
            PrAgentReadinessStatus::Waiting => Some(format!(
                "{} wait reason(s) remain",
                attempt.wait_reasons.len()
            )),
            PrAgentReadinessStatus::Stopped => {
                Some("pull request is closed or stopped".to_string())
            }
        })
}

fn render_pr_readiness_command(args: &PrReadinessArgs) -> String {
    let mut parts = vec![
        "codex-dev".to_string(),
        "pr".to_string(),
        "readiness".to_string(),
        "--capsule".to_string(),
        args.capsule.display().to_string(),
        "--repo".to_string(),
        args.repo.clone(),
        "--number".to_string(),
        args.number.to_string(),
        "--poll-attempts".to_string(),
        args.poll_attempts.to_string(),
        "--poll-interval-seconds".to_string(),
        args.poll_interval_seconds.to_string(),
    ];
    if let Some(checked_at) = args.checked_at {
        parts.push("--checked-at".to_string());
        parts.push(checked_at.to_rfc3339_opts(SecondsFormat::Nanos, true));
    }
    if let Some(source_dir) = &args.source_dir {
        parts.push("--source-dir".to_string());
        parts.push(source_dir.display().to_string());
    }
    if args.apply {
        parts.push("--apply".to_string());
    }
    if args.rerun_failed {
        parts.push("--rerun-failed".to_string());
    }
    if args.merge {
        parts.push("--merge".to_string());
        parts.push("--merge-method".to_string());
        parts.push(args.merge_method.as_str().to_string());
    }
    if args.delete_branch {
        parts.push("--delete-branch".to_string());
    }
    if let Some(subject) = &args.merge_subject {
        parts.push("--merge-subject".to_string());
        parts.push(subject.clone());
    }
    if let Some(body) = &args.merge_body {
        parts.push("--merge-body".to_string());
        parts.push(body.clone());
    }
    render_command(&parts)
}

pub fn run_pr_agent_hosted_action(
    args: PrAgentActionArgs,
    generated_at: DateTime<Utc>,
) -> Result<PrAgentHostedActionReport> {
    if args.apply && args.source_dir.is_some() {
        bail!("--source-dir is only allowed for dry-run planning; --apply must capture live state");
    }
    let (owner, name) = parse_github_repository(&args.repo)?;
    validate_plan_id(&args.plan_id)?;
    validate_hosted_action_args(&args)?;
    let body = read_hosted_action_body(&args)?;
    ensure_regular_contract_files(&args.capsule)?;
    let validation = validate_capsule(&args.capsule)?;
    if !validation.valid {
        bail!(
            "invalid capsule at {}: {}",
            args.capsule.display(),
            validation.errors.join("; ")
        );
    }

    let action_dir = prepare_pr_agent_action_dir(&args.capsule, &args.plan_id)?;
    let before_state = run_pr_agent_state(
        PrAgentArgs {
            capsule: args.capsule.clone(),
            repo: args.repo.clone(),
            number: args.number,
            checked_at: Some(generated_at),
            source_dir: args.source_dir.clone(),
        },
        generated_at,
    )?;
    let before_state_path = action_dir.join("before-state.json");
    ensure_pr_agent_report_path_safe(&before_state_path)?;
    write_json(before_state_path.clone(), &before_state)?;

    let intent = PrAgentHostedActionIntent {
        repository: args.repo.clone(),
        number: args.number,
        plan_id: args.plan_id.clone(),
        action: args.action,
        body: body.clone(),
        review_comment_id: args.review_comment_id,
        thread_id: args.thread_id.clone(),
        labels: args.labels.clone(),
        run_id: args.run_id,
    };
    let plan_hash = stable_json_hash(&intent)?;
    let idempotency_key = format!("codex-dev-pr-agent:{plan_hash}");
    let action = build_hosted_action_spec(&args, owner, name, &idempotency_key, body)?;
    let mut diagnostics = before_state.diagnostics.clone();
    diagnostics.extend(permission_diagnostics(&args, generated_at));
    let mut report = PrAgentHostedActionReport {
        schema: PR_AGENT_HOSTED_ACTION_SCHEMA.to_string(),
        repository: args.repo.clone(),
        number: args.number,
        plan_id: args.plan_id.clone(),
        plan_hash,
        generated_at,
        dry_run: !args.apply,
        apply_requested: args.apply,
        action_dir: action_dir.display().to_string(),
        before_state_path: before_state_path.display().to_string(),
        after_state_path: None,
        action,
        diagnostics: diagnostics.clone(),
        execution: None,
    };
    let report_path = action_dir.join("plan.json");
    ensure_pr_agent_report_path_safe(&report_path)?;
    write_json(report_path.clone(), &report)?;

    if !args.apply {
        append_pr_agent_action_evidence(&args, &report, &report_path, None)?;
        return Ok(report);
    }

    if let Some(preflight_execution) = preflight_hosted_action(
        &args,
        &report.action,
        &before_state,
        generated_at,
        &mut diagnostics,
    )? {
        report.diagnostics = diagnostics;
        report.execution = Some(preflight_execution);
        write_json(report_path.clone(), &report)?;
        append_pr_agent_action_evidence(&args, &report, &report_path, report.execution.as_ref())?;
        return Ok(report);
    }

    let execution = execute_hosted_action(&report.action, generated_at, &mut diagnostics);
    report.diagnostics = diagnostics;
    report.execution = Some(execution);

    if matches!(
        report.execution.as_ref().map(|execution| execution.status),
        Some(PrAgentHostedActionStatus::Applied | PrAgentHostedActionStatus::SkippedDuplicate)
    ) {
        let after_checked_at = next_state_timestamp(generated_at);
        match run_pr_agent_state(
            PrAgentArgs {
                capsule: args.capsule.clone(),
                repo: args.repo.clone(),
                number: args.number,
                checked_at: Some(after_checked_at),
                source_dir: None,
            },
            after_checked_at,
        ) {
            Ok(after_state) => {
                let after_state_path = action_dir.join("after-state.json");
                ensure_pr_agent_report_path_safe(&after_state_path)?;
                write_json(after_state_path.clone(), &after_state)?;
                report.after_state_path = Some(after_state_path.display().to_string());
            }
            Err(error) => report.diagnostics.push(PrAgentDiagnostic {
                source: "pr-agent-after-state".to_string(),
                severity: PrAgentSeverity::Error,
                message: format!(
                    "hosted action finished but after-state capture failed: {error:#}"
                ),
                command: None,
                exit_code: None,
                at: after_checked_at,
            }),
        }
    }

    write_json(report_path.clone(), &report)?;
    append_pr_agent_action_evidence(&args, &report, &report_path, report.execution.as_ref())?;
    Ok(report)
}

#[derive(Debug, Serialize)]
struct PrAgentHostedActionIntent {
    repository: String,
    number: u64,
    plan_id: String,
    action: PrAgentHostedActionKind,
    body: Option<String>,
    review_comment_id: Option<u64>,
    thread_id: Option<String>,
    labels: Vec<String>,
    run_id: Option<u64>,
}

#[derive(Debug)]
struct HostedCommandOutput {
    exit_code: Option<i32>,
    raw_stdout: Vec<u8>,
    stdout: Option<String>,
    stderr: Option<String>,
}

fn build_hosted_action_spec(
    args: &PrAgentActionArgs,
    owner: &str,
    name: &str,
    idempotency_key: &str,
    body: Option<String>,
) -> Result<PrAgentHostedActionSpec> {
    let mut duplicate_check_command = Vec::new();
    let mut state_check_command = Vec::new();
    let (target, command, summary, reason) = match args.action {
        PrAgentHostedActionKind::PostIssueComment => {
            let body = append_idempotency_marker(
                body.as_deref()
                    .expect("validated post issue comment body should exist"),
                idempotency_key,
            );
            duplicate_check_command = vec![
                "gh".to_string(),
                "api".to_string(),
                "--paginate".to_string(),
                "--slurp".to_string(),
                format!(
                    "repos/{owner}/{name}/issues/{}/comments?per_page=100",
                    args.number
                ),
            ];
            (
                format!("issue-comment:{}", args.number),
                vec![
                    "gh".to_string(),
                    "api".to_string(),
                    "--method".to_string(),
                    "POST".to_string(),
                    format!("repos/{owner}/{name}/issues/{}/comments", args.number),
                    "-f".to_string(),
                    format!("body={body}"),
                ],
                "Post PR conversation comment".to_string(),
                "reply to hosted PR discussion with explicit evidence".to_string(),
            )
        }
        PrAgentHostedActionKind::ReplyReviewComment => {
            let comment_id = args
                .review_comment_id
                .expect("validated review comment id should exist");
            let body = append_idempotency_marker(
                body.as_deref()
                    .expect("validated review reply body should exist"),
                idempotency_key,
            );
            duplicate_check_command = vec![
                "gh".to_string(),
                "api".to_string(),
                "--paginate".to_string(),
                "--slurp".to_string(),
                format!(
                    "repos/{owner}/{name}/pulls/{}/comments?per_page=100",
                    args.number
                ),
            ];
            (
                format!("review-comment:{comment_id}"),
                vec![
                    "gh".to_string(),
                    "api".to_string(),
                    "--method".to_string(),
                    "POST".to_string(),
                    format!(
                        "repos/{owner}/{name}/pulls/{}/comments/{comment_id}/replies",
                        args.number
                    ),
                    "-f".to_string(),
                    format!("body={body}"),
                ],
                "Reply to review comment".to_string(),
                "answer a specific top-level PR review comment with evidence".to_string(),
            )
        }
        PrAgentHostedActionKind::ResolveReviewThread => {
            let thread_id = args
                .thread_id
                .clone()
                .expect("validated review thread id should exist");
            (
                format!("review-thread:{thread_id}"),
                graph_ql_thread_command(&thread_id, RESOLVE_REVIEW_THREAD_MUTATION),
                "Resolve review thread".to_string(),
                "mark an addressed hosted review thread resolved".to_string(),
            )
        }
        PrAgentHostedActionKind::UnresolveReviewThread => {
            let thread_id = args
                .thread_id
                .clone()
                .expect("validated review thread id should exist");
            (
                format!("review-thread:{thread_id}"),
                graph_ql_thread_command(&thread_id, UNRESOLVE_REVIEW_THREAD_MUTATION),
                "Unresolve review thread".to_string(),
                "reopen a hosted review thread when it still needs work".to_string(),
            )
        }
        PrAgentHostedActionKind::AddLabels => (
            format!("labels:{}", args.labels.join(",")),
            issue_edit_label_command(&args.repo, args.number, "--add-label", &args.labels),
            "Add PR labels".to_string(),
            "apply explicit PR labels through the issue-backed PR surface".to_string(),
        ),
        PrAgentHostedActionKind::RemoveLabels => (
            format!("labels:{}", args.labels.join(",")),
            issue_edit_label_command(&args.repo, args.number, "--remove-label", &args.labels),
            "Remove PR labels".to_string(),
            "remove explicit PR labels through the issue-backed PR surface".to_string(),
        ),
        PrAgentHostedActionKind::RerunFailedJobs => {
            let run_id = args.run_id.expect("validated workflow run id should exist");
            state_check_command = vec![
                "gh".to_string(),
                "api".to_string(),
                format!("repos/{owner}/{name}/actions/runs/{run_id}"),
            ];
            (
                format!("workflow-run:{run_id}"),
                vec![
                    "gh".to_string(),
                    "api".to_string(),
                    "--method".to_string(),
                    "POST".to_string(),
                    format!("repos/{owner}/{name}/actions/runs/{run_id}/rerun-failed-jobs"),
                ],
                "Rerun failed workflow jobs".to_string(),
                "request a GitHub Actions retry for failed jobs in one workflow run".to_string(),
            )
        }
    };

    Ok(PrAgentHostedActionSpec {
        id: args.plan_id.clone(),
        kind: args.action.as_str().to_string(),
        summary,
        reason,
        target,
        idempotency_key: idempotency_key.to_string(),
        command,
        duplicate_check_command,
        state_check_command,
        requires_apply: true,
        network: true,
        secrets: true,
        permission_notes: permission_notes_for_action(args.action),
    })
}

fn preflight_hosted_action(
    args: &PrAgentActionArgs,
    action: &PrAgentHostedActionSpec,
    before_state: &PrAgentStateReport,
    checked_at: DateTime<Utc>,
    diagnostics: &mut Vec<PrAgentDiagnostic>,
) -> Result<Option<PrAgentHostedActionExecution>> {
    if before_state
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == PrAgentSeverity::Error)
    {
        let message =
            "refusing hosted write because live before-state capture has error diagnostics"
                .to_string();
        diagnostics.push(PrAgentDiagnostic {
            source: "pr-agent-preflight".to_string(),
            severity: PrAgentSeverity::Error,
            message: message.clone(),
            command: None,
            exit_code: None,
            at: checked_at,
        });
        return Ok(Some(failed_preflight_execution(
            action, checked_at, message,
        )));
    }

    match args.action {
        PrAgentHostedActionKind::PostIssueComment | PrAgentHostedActionKind::ReplyReviewComment => {
            Ok(None)
        }
        PrAgentHostedActionKind::ResolveReviewThread
        | PrAgentHostedActionKind::UnresolveReviewThread => {
            let thread_id = args
                .thread_id
                .as_deref()
                .expect("validated review thread id should exist");
            let Some(is_resolved) = review_thread_resolution(before_state, thread_id)? else {
                let message = format!(
                    "refusing hosted write because review thread {thread_id} was not found in current PR state"
                );
                diagnostics.push(PrAgentDiagnostic {
                    source: "pr-agent-preflight".to_string(),
                    severity: PrAgentSeverity::Error,
                    message: message.clone(),
                    command: None,
                    exit_code: None,
                    at: checked_at,
                });
                return Ok(Some(failed_preflight_execution(
                    action, checked_at, message,
                )));
            };
            let already_desired = match args.action {
                PrAgentHostedActionKind::ResolveReviewThread => is_resolved,
                PrAgentHostedActionKind::UnresolveReviewThread => !is_resolved,
                _ => unreachable!("thread actions handled only"),
            };
            if already_desired {
                return Ok(Some(skipped_preflight_execution(
                    action,
                    checked_at,
                    format!("review-thread:{thread_id}:already-desired"),
                )));
            }
            Ok(None)
        }
        PrAgentHostedActionKind::AddLabels | PrAgentHostedActionKind::RemoveLabels => {
            let Some(current_labels) = current_pr_labels(before_state)? else {
                let message =
                    "refusing hosted write because current PR labels were not captured".to_string();
                diagnostics.push(PrAgentDiagnostic {
                    source: "pr-agent-preflight".to_string(),
                    severity: PrAgentSeverity::Error,
                    message: message.clone(),
                    command: None,
                    exit_code: None,
                    at: checked_at,
                });
                return Ok(Some(failed_preflight_execution(
                    action, checked_at, message,
                )));
            };
            let requested = args
                .labels
                .iter()
                .map(|label| label.to_ascii_lowercase())
                .collect::<BTreeSet<_>>();
            let already_desired = match args.action {
                PrAgentHostedActionKind::AddLabels => {
                    requested.iter().all(|label| current_labels.contains(label))
                }
                PrAgentHostedActionKind::RemoveLabels => requested
                    .iter()
                    .all(|label| !current_labels.contains(label)),
                _ => unreachable!("label actions handled only"),
            };
            if already_desired {
                return Ok(Some(skipped_preflight_execution(
                    action,
                    checked_at,
                    format!("labels:{}:already-desired", args.labels.join(",")),
                )));
            }
            Ok(None)
        }
        PrAgentHostedActionKind::RerunFailedJobs => {
            let execution =
                preflight_workflow_rerun(args, action, before_state, checked_at, diagnostics)?;
            Ok(execution)
        }
    }
}

fn preflight_workflow_rerun(
    args: &PrAgentActionArgs,
    action: &PrAgentHostedActionSpec,
    before_state: &PrAgentStateReport,
    checked_at: DateTime<Utc>,
    diagnostics: &mut Vec<PrAgentDiagnostic>,
) -> Result<Option<PrAgentHostedActionExecution>> {
    if action.state_check_command.is_empty() {
        let message =
            "refusing hosted write because workflow run state check command is missing".to_string();
        diagnostics.push(PrAgentDiagnostic {
            source: "pr-agent-preflight".to_string(),
            severity: PrAgentSeverity::Error,
            message: message.clone(),
            command: None,
            exit_code: None,
            at: checked_at,
        });
        return Ok(Some(failed_preflight_execution(
            action, checked_at, message,
        )));
    }
    let output = match run_hosted_command(&action.state_check_command) {
        Ok(output) if output.exit_code == Some(0) => output,
        Ok(output) => {
            diagnostics.push(permission_failure_diagnostic(
                "pr-agent-state-check",
                &action.state_check_command,
                output.exit_code,
                output.stderr.as_deref(),
                checked_at,
            ));
            return Ok(Some(PrAgentHostedActionExecution {
                status: PrAgentHostedActionStatus::Failed,
                applied_at: checked_at,
                command: action.state_check_command.clone(),
                exit_code: output.exit_code,
                stdout: output.stdout,
                stderr: output.stderr,
                duplicate_of: None,
            }));
        }
        Err(error) => {
            let message = format!("{error:#}");
            diagnostics.push(PrAgentDiagnostic {
                source: "pr-agent-state-check".to_string(),
                severity: PrAgentSeverity::Error,
                message: message.clone(),
                command: Some(render_command(&action.state_check_command)),
                exit_code: None,
                at: checked_at,
            });
            return Ok(Some(PrAgentHostedActionExecution {
                status: PrAgentHostedActionStatus::Failed,
                applied_at: checked_at,
                command: action.state_check_command.clone(),
                exit_code: None,
                stdout: None,
                stderr: Some(message),
                duplicate_of: None,
            }));
        }
    };
    let value = serde_json::from_slice::<Value>(&output.raw_stdout)
        .context("workflow run state check did not return valid JSON")?;
    if let Some(message) = workflow_run_identity_error(args, before_state, &value) {
        diagnostics.push(PrAgentDiagnostic {
            source: "pr-agent-preflight".to_string(),
            severity: PrAgentSeverity::Error,
            message: message.clone(),
            command: Some(render_command(&action.state_check_command)),
            exit_code: output.exit_code,
            at: checked_at,
        });
        return Ok(Some(failed_preflight_execution(
            action, checked_at, message,
        )));
    }
    let run_head_sha = value.get("head_sha").and_then(Value::as_str);
    let Some(pr_head_sha) = before_state.pr.head_sha.as_deref() else {
        let message =
            "refusing hosted write because current PR head SHA was not captured".to_string();
        diagnostics.push(PrAgentDiagnostic {
            source: "pr-agent-preflight".to_string(),
            severity: PrAgentSeverity::Error,
            message: message.clone(),
            command: Some(render_command(&action.state_check_command)),
            exit_code: output.exit_code,
            at: checked_at,
        });
        return Ok(Some(failed_preflight_execution(
            action, checked_at, message,
        )));
    };
    if run_head_sha != Some(pr_head_sha) {
        let message = format!(
            "refusing hosted write because workflow run head_sha {:?} does not match PR head_sha {pr_head_sha}",
            run_head_sha
        );
        diagnostics.push(PrAgentDiagnostic {
            source: "pr-agent-preflight".to_string(),
            severity: PrAgentSeverity::Error,
            message: message.clone(),
            command: Some(render_command(&action.state_check_command)),
            exit_code: output.exit_code,
            at: checked_at,
        });
        return Ok(Some(failed_preflight_execution(
            action, checked_at, message,
        )));
    }
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let conclusion = value
        .get("conclusion")
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase);
    if status == "completed"
        && matches!(
            conclusion.as_deref(),
            Some("failure" | "cancelled" | "canceled" | "timed_out" | "action_required")
        )
    {
        return Ok(None);
    }
    if matches!(
        status.as_str(),
        "queued" | "in_progress" | "pending" | "requested" | "waiting"
    ) || matches!(
        conclusion.as_deref(),
        Some("success" | "neutral" | "skipped")
    ) {
        return Ok(Some(PrAgentHostedActionExecution {
            status: PrAgentHostedActionStatus::SkippedDuplicate,
            applied_at: checked_at,
            command: action.state_check_command.clone(),
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            duplicate_of: Some(format!(
                "workflow-run:{}:{}",
                status,
                conclusion.as_deref().unwrap_or("none")
            )),
        }));
    }
    let message = format!(
        "refusing hosted write because workflow run is not in a failed completed state: status={status}, conclusion={}",
        conclusion.as_deref().unwrap_or("none")
    );
    diagnostics.push(PrAgentDiagnostic {
        source: "pr-agent-preflight".to_string(),
        severity: PrAgentSeverity::Error,
        message: message.clone(),
        command: Some(render_command(&action.state_check_command)),
        exit_code: output.exit_code,
        at: checked_at,
    });
    Ok(Some(failed_preflight_execution(
        action, checked_at, message,
    )))
}

fn workflow_run_identity_error(
    args: &PrAgentActionArgs,
    before_state: &PrAgentStateReport,
    value: &Value,
) -> Option<String> {
    let expected_run_id = args.run_id?;
    let actual_run_id = value.get("id").and_then(Value::as_u64);
    if actual_run_id != Some(expected_run_id) {
        return Some(format!(
            "refusing hosted write because workflow run id {:?} does not match requested run id {expected_run_id}",
            actual_run_id
        ));
    }
    let repository = json_pointer_string(value, "/repository/full_name");
    if !repository
        .as_deref()
        .is_some_and(|repository| same_repository_name(repository, &args.repo))
    {
        return Some(format!(
            "refusing hosted write because workflow run repository {:?} does not match {}",
            repository, args.repo
        ));
    }
    let head_repository = json_pointer_string(value, "/head_repository/full_name");
    if !head_repository
        .as_deref()
        .is_some_and(|repository| same_repository_name(repository, &args.repo))
    {
        return Some(format!(
            "refusing hosted write because workflow run head repository {:?} does not match {}",
            head_repository, args.repo
        ));
    }
    if value
        .pointer("/head_repository/fork")
        .and_then(Value::as_bool)
        .unwrap_or(true)
    {
        return Some(
            "refusing hosted write because workflow run head repository is a fork or fork status was not captured"
                .to_string(),
        );
    }
    let event = value.get("event").and_then(Value::as_str);
    match event {
        Some("pull_request" | "push") => {}
        Some("pull_request_target" | "workflow_run") => {
            return Some(format!(
                "refusing hosted write because workflow run event {event:?} is privileged"
            ));
        }
        Some(other) => {
            return Some(format!(
                "refusing hosted write because workflow run event {other:?} is not allowed for PR readiness reruns"
            ));
        }
        None => {
            return Some(
                "refusing hosted write because workflow run event was not captured".to_string(),
            );
        }
    }
    let Some(expected_head_ref) = before_state.pr.head_ref_name.as_deref() else {
        return Some(
            "refusing hosted write because current PR head branch was not captured".to_string(),
        );
    };
    let actual_head_ref = value.get("head_branch").and_then(Value::as_str);
    if actual_head_ref != Some(expected_head_ref) {
        return Some(format!(
            "refusing hosted write because workflow run head branch {:?} does not match PR head branch {expected_head_ref}",
            actual_head_ref
        ));
    }
    let Some(pull_requests) = value.get("pull_requests").and_then(Value::as_array) else {
        return Some(
            "refusing hosted write because workflow run pull_requests were not captured"
                .to_string(),
        );
    };
    if pull_requests.is_empty() {
        if event != Some("push") {
            return Some(format!(
                "refusing hosted write because workflow run event {event:?} did not bind to PR {}",
                args.number
            ));
        }
    } else if !pull_requests.iter().any(|pull_request| {
        json_number_field(pull_request, "number")
            .or_else(|| json_pointer_u64(pull_request, "/pull_request/number"))
            == Some(args.number)
    }) {
        return Some(format!(
            "refusing hosted write because workflow run pull_requests do not include PR {}",
            args.number
        ));
    }
    let run_url = value
        .get("html_url")
        .and_then(Value::as_str)
        .or_else(|| value.get("url").and_then(Value::as_str));
    if !run_url.is_some_and(|url| {
        extract_github_actions_run_id_for_repo(url, &args.repo) == Some(expected_run_id)
    }) {
        return Some(format!(
            "refusing hosted write because workflow run URL {:?} does not bind to {} run {expected_run_id}",
            run_url, args.repo
        ));
    }
    None
}

fn json_pointer_string(value: &Value, pointer: &str) -> Option<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn json_pointer_u64(value: &Value, pointer: &str) -> Option<u64> {
    value
        .pointer(pointer)
        .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
}

fn json_number_field(value: &Value, field: &str) -> Option<u64> {
    value
        .get(field)
        .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
}

fn same_repository_name(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn failed_preflight_execution(
    action: &PrAgentHostedActionSpec,
    applied_at: DateTime<Utc>,
    message: String,
) -> PrAgentHostedActionExecution {
    PrAgentHostedActionExecution {
        status: PrAgentHostedActionStatus::Failed,
        applied_at,
        command: action.command.clone(),
        exit_code: None,
        stdout: None,
        stderr: Some(message),
        duplicate_of: None,
    }
}

fn skipped_preflight_execution(
    action: &PrAgentHostedActionSpec,
    applied_at: DateTime<Utc>,
    duplicate_of: String,
) -> PrAgentHostedActionExecution {
    PrAgentHostedActionExecution {
        status: PrAgentHostedActionStatus::SkippedDuplicate,
        applied_at,
        command: if action.state_check_command.is_empty() {
            action.command.clone()
        } else {
            action.state_check_command.clone()
        },
        exit_code: Some(0),
        stdout: None,
        stderr: None,
        duplicate_of: Some(duplicate_of),
    }
}

fn review_thread_resolution(
    before_state: &PrAgentStateReport,
    thread_id: &str,
) -> Result<Option<bool>> {
    let Some(path) = source_path(before_state, "gh-review-threads") else {
        return Ok(None);
    };
    let value = read_json::<Value>(&path)?;
    Ok(find_review_thread_resolution(&value, thread_id))
}

fn find_review_thread_resolution(value: &Value, thread_id: &str) -> Option<bool> {
    match value {
        Value::Array(values) => values
            .iter()
            .find_map(|value| find_review_thread_resolution(value, thread_id)),
        Value::Object(map) => {
            if map.get("id").and_then(Value::as_str) == Some(thread_id) {
                return map.get("isResolved").and_then(Value::as_bool);
            }
            map.values()
                .find_map(|value| find_review_thread_resolution(value, thread_id))
        }
        _ => None,
    }
}

fn current_pr_labels(before_state: &PrAgentStateReport) -> Result<Option<BTreeSet<String>>> {
    let Some(path) = source_path(before_state, "gh-pr-view") else {
        return Ok(None);
    };
    let value = read_json::<Value>(&path)?;
    let Some(labels) = value.get("labels").and_then(Value::as_array) else {
        return Ok(None);
    };
    Ok(Some(
        labels
            .iter()
            .filter_map(|label| {
                label
                    .as_str()
                    .or_else(|| label.get("name").and_then(Value::as_str))
                    .map(str::to_ascii_lowercase)
            })
            .collect(),
    ))
}

fn source_path(before_state: &PrAgentStateReport, id: &str) -> Option<PathBuf> {
    before_state
        .sources
        .iter()
        .find(|source| source.id == id && source.status == PrAgentSourceStatus::Captured)
        .map(|source| PathBuf::from(&source.path))
}

fn execute_hosted_action(
    action: &PrAgentHostedActionSpec,
    applied_at: DateTime<Utc>,
    diagnostics: &mut Vec<PrAgentDiagnostic>,
) -> PrAgentHostedActionExecution {
    if !action.duplicate_check_command.is_empty() {
        match run_hosted_command(&action.duplicate_check_command) {
            Ok(output) if output.exit_code == Some(0) => {
                if let Some(duplicate_of) =
                    duplicate_comment_reference(&output.raw_stdout, &action.idempotency_key)
                {
                    return PrAgentHostedActionExecution {
                        status: PrAgentHostedActionStatus::SkippedDuplicate,
                        applied_at,
                        command: action.duplicate_check_command.clone(),
                        exit_code: Some(0),
                        stdout: output.stdout,
                        stderr: output.stderr,
                        duplicate_of: Some(duplicate_of),
                    };
                }
            }
            Ok(output) => {
                diagnostics.push(permission_failure_diagnostic(
                    "pr-agent-duplicate-check",
                    &action.duplicate_check_command,
                    output.exit_code,
                    output.stderr.as_deref(),
                    applied_at,
                ));
                return PrAgentHostedActionExecution {
                    status: PrAgentHostedActionStatus::Failed,
                    applied_at,
                    command: action.duplicate_check_command.clone(),
                    exit_code: output.exit_code,
                    stdout: output.stdout,
                    stderr: output.stderr,
                    duplicate_of: None,
                };
            }
            Err(error) => {
                let message = format!("{error:#}");
                diagnostics.push(PrAgentDiagnostic {
                    source: "pr-agent-duplicate-check".to_string(),
                    severity: PrAgentSeverity::Error,
                    message: message.clone(),
                    command: Some(render_command(&action.duplicate_check_command)),
                    exit_code: None,
                    at: applied_at,
                });
                return PrAgentHostedActionExecution {
                    status: PrAgentHostedActionStatus::Failed,
                    applied_at,
                    command: action.duplicate_check_command.clone(),
                    exit_code: None,
                    stdout: None,
                    stderr: Some(message),
                    duplicate_of: None,
                };
            }
        }
    }

    match run_hosted_command(&action.command) {
        Ok(output) => {
            let status = if output.exit_code == Some(0) {
                PrAgentHostedActionStatus::Applied
            } else {
                diagnostics.push(permission_failure_diagnostic(
                    "pr-agent-apply",
                    &action.command,
                    output.exit_code,
                    output.stderr.as_deref(),
                    applied_at,
                ));
                PrAgentHostedActionStatus::Failed
            };
            PrAgentHostedActionExecution {
                status,
                applied_at,
                command: action.command.clone(),
                exit_code: output.exit_code,
                stdout: output.stdout,
                stderr: output.stderr,
                duplicate_of: None,
            }
        }
        Err(error) => {
            let message = format!("{error:#}");
            diagnostics.push(PrAgentDiagnostic {
                source: "pr-agent-apply".to_string(),
                severity: PrAgentSeverity::Error,
                message: message.clone(),
                command: Some(render_command(&action.command)),
                exit_code: None,
                at: applied_at,
            });
            PrAgentHostedActionExecution {
                status: PrAgentHostedActionStatus::Failed,
                applied_at,
                command: action.command.clone(),
                exit_code: None,
                stdout: None,
                stderr: Some(message),
                duplicate_of: None,
            }
        }
    }
}

fn run_hosted_command(command: &[String]) -> Result<HostedCommandOutput> {
    let Some((program, arguments)) = command.split_first() else {
        bail!("hosted action command is empty");
    };
    let output = Command::new(program)
        .args(arguments)
        .output()
        .with_context(|| {
            format!(
                "failed to start hosted action command {}",
                render_command(command)
            )
        })?;
    Ok(HostedCommandOutput {
        exit_code: output.status.code(),
        raw_stdout: output.stdout.clone(),
        stdout: diagnostic_excerpt(&output.stdout),
        stderr: diagnostic_excerpt(&output.stderr),
    })
}

fn duplicate_comment_reference(stdout: &[u8], idempotency_key: &str) -> Option<String> {
    let value = serde_json::from_slice::<Value>(stdout).ok()?;
    find_comment_marker(&value, idempotency_key)
}

fn find_comment_marker(value: &Value, idempotency_key: &str) -> Option<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .find_map(|item| find_comment_marker(item, idempotency_key)),
        Value::Object(map) => {
            if map
                .get("body")
                .and_then(Value::as_str)
                .is_some_and(|body| body.contains(idempotency_key))
            {
                return map
                    .get("html_url")
                    .or_else(|| map.get("url"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| map.get("id").map(Value::to_string));
            }
            None
        }
        _ => None,
    }
}

fn append_idempotency_marker(body: &str, idempotency_key: &str) -> String {
    format!("{}\n\n<!-- {idempotency_key} -->", body.trim_end())
}

fn graph_ql_thread_command(thread_id: &str, mutation: &str) -> Vec<String> {
    vec![
        "gh".to_string(),
        "api".to_string(),
        "graphql".to_string(),
        "-f".to_string(),
        format!("threadId={thread_id}"),
        "-f".to_string(),
        format!("query={mutation}"),
    ]
}

fn issue_edit_label_command(
    repository: &str,
    number: u64,
    flag: &str,
    labels: &[String],
) -> Vec<String> {
    let mut command = vec![
        "gh".to_string(),
        "issue".to_string(),
        "edit".to_string(),
        number.to_string(),
        "--repo".to_string(),
        repository.to_string(),
    ];
    for label in labels {
        command.push(flag.to_string());
        command.push(label.clone());
    }
    command
}

fn permission_notes_for_action(action: PrAgentHostedActionKind) -> Vec<String> {
    match action {
        PrAgentHostedActionKind::PostIssueComment => vec![
            "GitHub REST issue comments require Issues write or Pull requests write permissions"
                .to_string(),
            "PR conversation comments trigger notifications and may hit secondary rate limits"
                .to_string(),
        ],
        PrAgentHostedActionKind::ReplyReviewComment => vec![
            "GitHub REST review-comment replies require Pull requests write permissions"
                .to_string(),
            "GitHub only supports replies to top-level review comments, not replies to replies"
                .to_string(),
        ],
        PrAgentHostedActionKind::ResolveReviewThread
        | PrAgentHostedActionKind::UnresolveReviewThread => vec![
            "GitHub GraphQL review-thread mutations require Pull requests write permissions"
                .to_string(),
        ],
        PrAgentHostedActionKind::AddLabels | PrAgentHostedActionKind::RemoveLabels => vec![
            "PR labels are issue-backed and require Issues write or Pull requests write permissions"
                .to_string(),
        ],
        PrAgentHostedActionKind::RerunFailedJobs => vec![
            "Rerunning failed workflow jobs requires Actions write permissions".to_string(),
            "GitHub reruns use the privileges of the actor that triggered the original workflow"
                .to_string(),
        ],
    }
}

fn permission_diagnostics(
    args: &PrAgentActionArgs,
    generated_at: DateTime<Utc>,
) -> Vec<PrAgentDiagnostic> {
    let has_github_token = ["GITHUB_TOKEN", "GITHUB_ENTERPRISE_TOKEN"]
        .iter()
        .any(|name| non_empty_env_var(name));
    let has_gh_token = ["GH_TOKEN", "GH_ENTERPRISE_TOKEN"]
        .iter()
        .any(|name| non_empty_env_var(name));
    let message = if has_github_token {
        "GITHUB_TOKEN or GITHUB_ENTERPRISE_TOKEN is set; workflow tokens and GitHub App tokens may be repository-scoped and can lack PR, issue, or Actions write permissions".to_string()
    } else if has_gh_token {
        "GH_TOKEN or GH_ENTERPRISE_TOKEN is set; verify the token has the write permissions listed in permission_notes before using --apply".to_string()
    } else {
        "no GH_TOKEN, GITHUB_TOKEN, GH_ENTERPRISE_TOKEN, or GITHUB_ENTERPRISE_TOKEN environment variable detected; gh may use a credential store, and permission failures will be captured from hosted command stderr".to_string()
    };
    vec![PrAgentDiagnostic {
        source: "github-auth".to_string(),
        severity: PrAgentSeverity::Info,
        message,
        command: Some(format!(
            "codex-dev pr agent-action --repo {} --number {} --plan-id {} --action {}{}",
            args.repo,
            args.number,
            args.plan_id,
            args.action.as_str(),
            if args.apply { " --apply" } else { "" }
        )),
        exit_code: None,
        at: generated_at,
    }]
}

fn permission_failure_diagnostic(
    source: &str,
    command: &[String],
    exit_code: Option<i32>,
    stderr: Option<&str>,
    at: DateTime<Utc>,
) -> PrAgentDiagnostic {
    let stderr_suffix = stderr
        .filter(|stderr| !stderr.is_empty())
        .map(|stderr| format!(": {}", redact_sensitive_text(stderr)))
        .unwrap_or_default();
    PrAgentDiagnostic {
        source: source.to_string(),
        severity: PrAgentSeverity::Error,
        message: format!(
            "hosted GitHub command failed; verify token type and repository permissions for this action{stderr_suffix}"
        ),
        command: Some(render_command(command)),
        exit_code,
        at,
    }
}

fn append_pr_agent_action_evidence(
    args: &PrAgentActionArgs,
    report: &PrAgentHostedActionReport,
    report_path: &Path,
    execution: Option<&PrAgentHostedActionExecution>,
) -> Result<()> {
    let status = execution
        .map(|execution| format!("{:?}", execution.status))
        .unwrap_or_else(|| "planned".to_string());
    append_evidence(AppendEvidenceArgs {
        capsule: args.capsule.clone(),
        record: EvidenceRecord {
            schema: EVIDENCE_SCHEMA.to_string(),
            kind: if args.apply {
                EvidenceKind::Review
            } else {
                EvidenceKind::Decision
            },
            at: execution
                .map(|execution| execution.applied_at)
                .unwrap_or(report.generated_at),
            summary: format!(
                "PR agent hosted action {} for {}#{}: {status}",
                report.plan_id, report.repository, report.number
            ),
            command: Some(render_pr_agent_action_invocation(args)),
            exit_code: execution.and_then(|execution| execution.exit_code),
            source_ids: vec![
                format!("pr-agent-action:{}", report.plan_id),
                report.plan_hash.clone(),
            ],
            actor: None,
            tool: Some("codex-dev".to_string()),
            confidence: None,
            residual_risk: report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == PrAgentSeverity::Error)
                .then(|| "one or more hosted action diagnostics are errors".to_string()),
            artifacts: action_artifacts(report, report_path),
        },
    })?;
    Ok(())
}

fn render_pr_agent_action_invocation(args: &PrAgentActionArgs) -> String {
    let mut command = vec![
        "codex-dev".to_string(),
        "pr".to_string(),
        "agent-action".to_string(),
        "--capsule".to_string(),
        args.capsule.display().to_string(),
        "--repo".to_string(),
        args.repo.clone(),
        "--number".to_string(),
        args.number.to_string(),
        "--plan-id".to_string(),
        args.plan_id.clone(),
        "--action".to_string(),
        args.action.as_str().to_string(),
    ];
    if args.apply {
        command.push("--apply".to_string());
    }
    render_command(&command)
}

fn action_artifacts(report: &PrAgentHostedActionReport, report_path: &Path) -> Vec<String> {
    let mut artifacts = vec![
        report_path.display().to_string(),
        report.before_state_path.clone(),
    ];
    if let Some(after_state_path) = &report.after_state_path {
        artifacts.push(after_state_path.clone());
    }
    artifacts
}

fn prepare_pr_agent_action_dir(capsule: &Path, plan_id: &str) -> Result<PathBuf> {
    let actions_root = capsule.join("pr-agent-actions");
    create_pr_agent_dir_without_symlink(&actions_root, true)?;
    let action_dir = actions_root.join(plan_id);
    create_pr_agent_dir_without_symlink(&action_dir, true)?;
    Ok(action_dir)
}

fn validate_plan_id(plan_id: &str) -> Result<()> {
    if plan_id.is_empty() {
        bail!("--plan-id must not be empty");
    }
    if !plan_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        bail!("--plan-id must contain only ASCII letters, numbers, '-' or '_': {plan_id}");
    }
    if plan_id.contains('/') || plan_id.contains('\\') {
        bail!("--plan-id must be a single safe path segment: {plan_id}");
    }
    Ok(())
}

fn validate_hosted_action_args(args: &PrAgentActionArgs) -> Result<()> {
    let has_body = args.body.is_some() || args.body_file.is_some();
    match args.action {
        PrAgentHostedActionKind::PostIssueComment => {
            require_body(args, "post-issue-comment")?;
            reject_targets(args, true, false, false, false)?;
        }
        PrAgentHostedActionKind::ReplyReviewComment => {
            require_body(args, "reply-review-comment")?;
            if args.review_comment_id.is_none() {
                bail!("reply-review-comment requires --review-comment-id");
            }
            reject_targets(args, true, true, false, false)?;
        }
        PrAgentHostedActionKind::ResolveReviewThread
        | PrAgentHostedActionKind::UnresolveReviewThread => {
            if has_body {
                bail!(
                    "{} does not accept --body or --body-file",
                    args.action.as_str()
                );
            }
            if args.thread_id.as_deref().is_none_or(str::is_empty) {
                bail!("{} requires --thread-id", args.action.as_str());
            }
            reject_targets(args, false, false, true, false)?;
        }
        PrAgentHostedActionKind::AddLabels | PrAgentHostedActionKind::RemoveLabels => {
            if has_body {
                bail!(
                    "{} does not accept --body or --body-file",
                    args.action.as_str()
                );
            }
            if args.labels.is_empty() {
                bail!("{} requires at least one --label", args.action.as_str());
            }
            for label in &args.labels {
                validate_simple_text("--label", label)?;
            }
            reject_targets(args, false, false, false, false)?;
        }
        PrAgentHostedActionKind::RerunFailedJobs => {
            if has_body {
                bail!("rerun-failed-jobs does not accept --body or --body-file");
            }
            if args.run_id.is_none() {
                bail!("rerun-failed-jobs requires --run-id");
            }
            reject_targets(args, false, false, false, true)?;
        }
    }
    Ok(())
}

fn require_body(args: &PrAgentActionArgs, action: &str) -> Result<()> {
    match (&args.body, &args.body_file) {
        (Some(_), Some(_)) => bail!("{action} accepts only one of --body or --body-file"),
        (None, None) => bail!("{action} requires --body or --body-file"),
        _ => Ok(()),
    }
}

fn reject_targets(
    args: &PrAgentActionArgs,
    allow_body: bool,
    allow_review_comment_id: bool,
    allow_thread_id: bool,
    allow_run_id: bool,
) -> Result<()> {
    if !allow_body && (args.body.is_some() || args.body_file.is_some()) {
        bail!("{} does not accept body input", args.action.as_str());
    }
    if !allow_review_comment_id && args.review_comment_id.is_some() {
        bail!(
            "{} does not accept --review-comment-id",
            args.action.as_str()
        );
    }
    if !allow_thread_id && args.thread_id.is_some() {
        bail!("{} does not accept --thread-id", args.action.as_str());
    }
    if !allow_run_id && args.run_id.is_some() {
        bail!("{} does not accept --run-id", args.action.as_str());
    }
    Ok(())
}

fn read_hosted_action_body(args: &PrAgentActionArgs) -> Result<Option<String>> {
    let body = match (&args.body, &args.body_file) {
        (Some(body), None) => Some(body.clone()),
        (None, Some(path)) => Some(
            fs::read_to_string(path)
                .with_context(|| format!("failed to read --body-file {}", path.display()))?,
        ),
        _ => None,
    };
    if let Some(body) = &body {
        validate_body_text(body)?;
    }
    Ok(body)
}

fn validate_body_text(body: &str) -> Result<()> {
    if body.trim().is_empty() {
        bail!("body must not be empty");
    }
    if body
        .chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
    {
        bail!("body must not contain control characters other than tab or newline");
    }
    Ok(())
}

fn validate_simple_text(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{field} must not be empty");
    }
    if value.chars().any(char::is_control) {
        bail!("{field} must not contain control characters");
    }
    Ok(())
}

fn next_state_timestamp(generated_at: DateTime<Utc>) -> DateTime<Utc> {
    let now = Utc::now();
    if now.timestamp_millis() <= generated_at.timestamp_millis() {
        generated_at + TimeDelta::milliseconds(1)
    } else {
        now
    }
}

#[derive(Clone, Debug)]
struct PrAgentSourceSpec {
    id: String,
    kind: String,
    file_name: String,
    command: Vec<String>,
    source_kind: Option<PrRecordSourceKind>,
    required: bool,
    flatten_paginated_arrays: bool,
}

#[derive(Debug)]
struct CapturedPrAgentSource {
    source: PrAgentSourceRecord,
    path: PathBuf,
    diagnostics: Vec<PrAgentDiagnostic>,
}

fn prepare_pr_agent_output_dir(capsule: &Path, checked_at: DateTime<Utc>) -> Result<PathBuf> {
    let sources_root = capsule.join("pr-agent-sources");
    create_pr_agent_dir_without_symlink(&sources_root, true)?;

    let output_dir = sources_root.join(format!("{}", checked_at.timestamp_millis()));
    create_pr_agent_dir_without_symlink(&output_dir, false)?;
    Ok(output_dir)
}

fn create_pr_agent_dir_without_symlink(path: &Path, allow_existing_empty_dir: bool) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                bail!(
                    "refusing to use symlinked PR agent source directory {}",
                    path.display()
                );
            }
            if !metadata.is_dir() {
                bail!(
                    "refusing to use non-directory PR agent source path {}",
                    path.display()
                );
            }
            if !allow_existing_empty_dir
                && fs::read_dir(path)
                    .with_context(|| {
                        format!(
                            "failed to inspect PR agent source directory {}",
                            path.display()
                        )
                    })?
                    .next()
                    .is_some()
            {
                bail!(
                    "PR agent source directory already exists and is not empty: {}; choose a different --checked-at or remove the directory",
                    path.display()
                );
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(path).with_context(|| {
                format!(
                    "failed to create PR agent source directory {}",
                    path.display()
                )
            })?;
            let metadata = fs::symlink_metadata(path).with_context(|| {
                format!(
                    "failed to inspect PR agent source directory {}",
                    path.display()
                )
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!(
                    "refusing to use unsafe PR agent source directory {}",
                    path.display()
                );
            }
        }
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "failed to inspect PR agent source directory {}",
                    path.display()
                )
            });
        }
    }
    Ok(())
}

fn ensure_pr_agent_report_path_safe(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                bail!(
                    "refusing to write symlinked PR agent state report {}",
                    path.display()
                );
            }
            if !metadata.is_file() {
                bail!(
                    "refusing to overwrite non-file PR agent state report path {}",
                    path.display()
                );
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "failed to inspect PR agent state report path {}",
                    path.display()
                )
            });
        }
    }
    Ok(())
}

fn pr_agent_source_specs(
    repository: &str,
    owner: &str,
    name: &str,
    number: u64,
) -> Vec<PrAgentSourceSpec> {
    let reviews_path = format!("repos/{owner}/{name}/pulls/{number}/reviews?per_page=100");
    let review_comments_path = format!("repos/{owner}/{name}/pulls/{number}/comments?per_page=100");
    vec![
        pr_agent_source_spec(
            "gh-pr-view",
            "github-pr-view",
            "gh-pr-view.json",
            vec![
                "gh",
                "pr",
                "view",
                &number.to_string(),
                "--repo",
                repository,
                "--json",
                GH_PR_VIEW_JSON_FIELDS,
            ],
            Some(PrRecordSourceKind::GhPrView),
        ),
        pr_agent_source_spec(
            "gh-pr-checks",
            "github-pr-checks",
            "gh-pr-checks.json",
            vec![
                "gh",
                "pr",
                "checks",
                &number.to_string(),
                "--repo",
                repository,
                "--json",
                "bucket,completedAt,description,event,link,name,startedAt,state,workflow",
            ],
            Some(PrRecordSourceKind::GhPrChecks),
        ),
        pr_agent_source_spec(
            "gh-reviews",
            "github-rest-reviews",
            "gh-reviews.json",
            vec!["gh", "api", "--paginate", "--slurp", &reviews_path],
            Some(PrRecordSourceKind::GhReviews),
        )
        .flatten_paginated_arrays(),
        pr_agent_source_spec(
            "gh-review-comments",
            "github-rest-review-comments",
            "gh-review-comments.json",
            vec!["gh", "api", "--paginate", "--slurp", &review_comments_path],
            Some(PrRecordSourceKind::GhReviewComments),
        )
        .flatten_paginated_arrays(),
        pr_agent_source_spec(
            "gh-review-threads",
            "github-graphql-review-threads",
            "gh-review-threads.json",
            vec![
                "gh",
                "api",
                "graphql",
                "--paginate",
                "--slurp",
                "-f",
                &format!("owner={owner}"),
                "-f",
                &format!("name={name}"),
                "-F",
                &format!("number={number}"),
                "-f",
                &format!("query={PR_REVIEW_THREADS_QUERY}"),
            ],
            Some(PrRecordSourceKind::GhReviewThreads),
        ),
        pr_agent_source_spec(
            "gh-rate-limit",
            "github-rate-limit",
            "gh-rate-limit.json",
            vec!["gh", "api", "rate_limit"],
            None,
        )
        .optional(),
    ]
}

fn pr_agent_source_spec(
    id: &str,
    kind: &str,
    file_name: &str,
    command: Vec<&str>,
    source_kind: Option<PrRecordSourceKind>,
) -> PrAgentSourceSpec {
    PrAgentSourceSpec {
        id: id.to_string(),
        kind: kind.to_string(),
        file_name: file_name.to_string(),
        command: command.into_iter().map(str::to_string).collect(),
        source_kind,
        required: true,
        flatten_paginated_arrays: false,
    }
}

impl PrAgentSourceSpec {
    fn flatten_paginated_arrays(mut self) -> Self {
        self.flatten_paginated_arrays = true;
        self
    }

    fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

fn capture_pr_agent_source(
    args: &PrAgentArgs,
    spec: &PrAgentSourceSpec,
    output_dir: &Path,
    checked_at: DateTime<Utc>,
) -> Result<CapturedPrAgentSource> {
    let output_path = output_dir.join(&spec.file_name);
    let mut diagnostics = Vec::new();
    let command = render_command(&spec.command);

    let raw_result = if let Some(source_dir) = &args.source_dir {
        let fixture_path = source_dir.join(&spec.file_name);
        fs::read(&fixture_path)
            .map(|bytes| (bytes, Some(0)))
            .with_context(|| {
                format!(
                    "failed to read replay source {} for {}",
                    fixture_path.display(),
                    spec.id
                )
            })
    } else {
        run_pr_agent_source_command(spec)
    };

    let (raw_bytes, exit_code) = match raw_result {
        Ok(result) => result,
        Err(error) => {
            let message = format!("{error:#}");
            diagnostics.push(PrAgentDiagnostic {
                source: spec.id.clone(),
                severity: if spec.required {
                    PrAgentSeverity::Error
                } else {
                    PrAgentSeverity::Warning
                },
                message: message.clone(),
                command: Some(command.clone()),
                exit_code: None,
                at: checked_at,
            });
            write_pr_agent_failure_artifact(&output_path, spec, checked_at, &message, None)?;
            return Ok(CapturedPrAgentSource {
                source: PrAgentSourceRecord {
                    id: spec.id.clone(),
                    kind: spec.kind.clone(),
                    command,
                    path: output_path.display().to_string(),
                    retrieved_at: checked_at,
                    exit_code: None,
                    status: PrAgentSourceStatus::Failed,
                },
                path: output_path,
                diagnostics,
            });
        }
    };

    let parsed = serde_json::from_slice::<Value>(&raw_bytes).with_context(|| {
        format!(
            "captured source {} was not valid JSON for command: {command}",
            spec.id
        )
    });

    let command_failed = exit_code.is_some_and(|code| code != 0);
    let status = if parsed.is_ok() && !(spec.required && command_failed) {
        PrAgentSourceStatus::Captured
    } else {
        PrAgentSourceStatus::Failed
    };

    if let Err(error) = parsed.as_ref() {
        let message = format!("{error:#}");
        diagnostics.push(PrAgentDiagnostic {
            source: spec.id.clone(),
            severity: if spec.required {
                PrAgentSeverity::Error
            } else {
                PrAgentSeverity::Warning
            },
            message: message.clone(),
            command: Some(command.clone()),
            exit_code,
            at: checked_at,
        });
        write_pr_agent_failure_artifact(&output_path, spec, checked_at, &message, exit_code)?;
    }

    if let Some(code) = exit_code
        && code != 0
        && parsed.is_ok()
    {
        diagnostics.push(PrAgentDiagnostic {
            source: spec.id.clone(),
            severity: if spec.required {
                PrAgentSeverity::Error
            } else {
                PrAgentSeverity::Info
            },
            message: if spec.required {
                format!(
                    "source command exited with status {code}; JSON output was captured but required failed sources are not normalized"
                )
            } else {
                format!("optional source command exited with status {code}; JSON output was captured")
            },
            command: Some(command.clone()),
            exit_code,
            at: checked_at,
        });
    }

    if let Ok(value) = parsed {
        let value = if spec.flatten_paginated_arrays {
            flatten_paginated_arrays(value)
        } else {
            value
        };
        write_json(output_path.clone(), &value)?;
    }

    Ok(CapturedPrAgentSource {
        source: PrAgentSourceRecord {
            id: spec.id.clone(),
            kind: spec.kind.clone(),
            command,
            path: output_path.display().to_string(),
            retrieved_at: checked_at,
            exit_code,
            status,
        },
        path: output_path,
        diagnostics,
    })
}

fn write_pr_agent_failure_artifact(
    output_path: &Path,
    spec: &PrAgentSourceSpec,
    checked_at: DateTime<Utc>,
    message: &str,
    exit_code: Option<i32>,
) -> Result<()> {
    write_json(
        output_path.to_path_buf(),
        &json!({
            "schema": "codex-dev.pr-agent-source-failure.v1",
            "source": spec.id,
            "kind": spec.kind,
            "status": "failed",
            "message": message,
            "exit_code": exit_code,
            "captured_at": checked_at,
        }),
    )
}

fn run_pr_agent_source_command(spec: &PrAgentSourceSpec) -> Result<(Vec<u8>, Option<i32>)> {
    let Some((program, arguments)) = spec.command.split_first() else {
        bail!("source command {} is empty", spec.id);
    };
    let output = Command::new(program)
        .args(arguments)
        .output()
        .with_context(|| {
            format!(
                "failed to start source command {}",
                render_command(&spec.command)
            )
        })?;
    if !output.status.success() && output.stdout.is_empty() {
        let stderr = diagnostic_excerpt(&output.stderr);
        bail!(
            "source command {} failed with status {:?}: {}",
            render_command(&spec.command),
            output.status.code(),
            stderr.unwrap_or_else(|| "no stderr".to_string())
        );
    }
    Ok((output.stdout, output.status.code()))
}

fn flatten_paginated_arrays(value: Value) -> Value {
    let Some(pages) = value.as_array() else {
        return value;
    };
    if !pages.iter().all(Value::is_array) {
        return value;
    }
    let flattened = pages
        .iter()
        .flat_map(|page| page.as_array().into_iter().flatten().cloned())
        .collect::<Vec<_>>();
    Value::Array(flattened)
}

fn rate_limit_diagnostics(
    path: &Path,
    spec: &PrAgentSourceSpec,
    checked_at: DateTime<Utc>,
) -> Result<Vec<PrAgentDiagnostic>> {
    let value = read_json::<Value>(path)?;
    let remaining = value
        .pointer("/resources/core/remaining")
        .or_else(|| value.pointer("/rate/remaining"))
        .and_then(Value::as_u64);
    let limit = value
        .pointer("/resources/core/limit")
        .or_else(|| value.pointer("/rate/limit"))
        .and_then(Value::as_u64);

    let Some(remaining) = remaining else {
        return Ok(vec![PrAgentDiagnostic {
            source: spec.id.clone(),
            severity: PrAgentSeverity::Warning,
            message: "rate-limit source did not include core remaining count".to_string(),
            command: Some(render_command(&spec.command)),
            exit_code: Some(0),
            at: checked_at,
        }]);
    };

    let severity = if remaining == 0 {
        PrAgentSeverity::Error
    } else if remaining < 20 {
        PrAgentSeverity::Warning
    } else {
        PrAgentSeverity::Info
    };
    let limit_suffix = limit
        .map(|limit| format!(" of {limit}"))
        .unwrap_or_else(String::new);
    Ok(vec![PrAgentDiagnostic {
        source: spec.id.clone(),
        severity,
        message: format!("GitHub core rate limit remaining: {remaining}{limit_suffix}"),
        command: Some(render_command(&spec.command)),
        exit_code: Some(0),
        at: checked_at,
    }])
}

fn render_pr_agent_command(args: &PrAgentArgs, checked_at: DateTime<Utc>) -> String {
    let mut command = vec![
        "codex-dev".to_string(),
        "pr".to_string(),
        "agent".to_string(),
        "--capsule".to_string(),
        args.capsule.display().to_string(),
        "--repo".to_string(),
        args.repo.clone(),
        "--number".to_string(),
        args.number.to_string(),
        "--checked-at".to_string(),
        checked_at.to_rfc3339_opts(SecondsFormat::AutoSi, true),
    ];
    if let Some(source_dir) = &args.source_dir {
        command.push("--source-dir".to_string());
        command.push(source_dir.display().to_string());
    }
    render_command(&command)
}

fn diagnostic_excerpt(bytes: &[u8]) -> Option<String> {
    let text = redact_sensitive_text(String::from_utf8_lossy(bytes).trim());
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

/// Redact credential-looking content before returning command output in reports.
fn redact_sensitive_text(text: impl AsRef<str>) -> String {
    let text = redact_authorization_lines(text.as_ref());
    let text = redact_key_assignments(&text, GITHUB_TOKEN_ENV_VARS);
    redact_prefixed_tokens(
        &text,
        &[
            "sk-proj-",
            "sk-",
            "github_pat_",
            "ghp_",
            "gho_",
            "ghu_",
            "ghs_",
            "ghr_",
            "Bearer ",
            "bearer ",
        ],
    )
}

fn redact_authorization_lines(text: &str) -> String {
    text.lines()
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            if let Some(index) = lower.find("authorization:") {
                format!("{}authorization: [redacted]", &line[..index])
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn redact_key_assignments(text: &str, keys: &[&str]) -> String {
    let mut redacted = text.to_string();
    for key in keys {
        redacted = redact_assignment_values(&redacted, key);
    }
    redacted
}

fn redact_assignment_values(text: &str, key: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        if rest.starts_with(key)
            && assignment_key_starts_at_boundary(text, index)
            && let Some(value_start) = assignment_value_start(rest, key)
        {
            output.push_str(key);
            output.push_str("=[redacted]");
            index += value_start;
            while index < text.len() {
                let ch = text[index..].chars().next().expect("character");
                if ch.is_whitespace() || matches!(ch, ',' | ';') {
                    break;
                }
                index += ch.len_utf8();
            }
        } else {
            let ch = rest.chars().next().expect("character");
            output.push(ch);
            index += ch.len_utf8();
        }
    }
    output
}

fn assignment_key_starts_at_boundary(text: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }
    text[..index]
        .chars()
        .next_back()
        .is_none_or(|ch| !assignment_key_character(ch))
}

fn assignment_value_start(rest: &str, key: &str) -> Option<usize> {
    let mut offset = key.len();
    while offset < rest.len() {
        let ch = rest[offset..].chars().next().expect("character");
        if !ch.is_whitespace() {
            break;
        }
        offset += ch.len_utf8();
    }
    let equals = rest[offset..].chars().next()?;
    if equals != '=' {
        return None;
    }
    offset += equals.len_utf8();
    while offset < rest.len() {
        let ch = rest[offset..].chars().next().expect("character");
        if !ch.is_whitespace() {
            break;
        }
        offset += ch.len_utf8();
    }
    Some(offset)
}

fn assignment_key_character(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn redact_prefixed_tokens(text: &str, prefixes: &[&str]) -> String {
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        if let Some(prefix) = prefixes.iter().find(|prefix| rest.starts_with(**prefix)) {
            output.push_str("[redacted]");
            index += prefix.len();
            while index < text.len() {
                let ch = text[index..].chars().next().expect("character");
                if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') {
                    index += ch.len_utf8();
                } else {
                    break;
                }
            }
        } else {
            let ch = rest.chars().next().expect("character");
            output.push(ch);
            index += ch.len_utf8();
        }
    }
    output
}

fn parse_github_repository(repository: &str) -> Result<(&str, &str)> {
    let Some((owner, name)) = repository.split_once('/') else {
        bail!("repository must be in OWNER/REPO form: {repository}");
    };
    if owner.is_empty() || name.is_empty() || name.contains('/') {
        bail!("repository must be in OWNER/REPO form: {repository}");
    }
    Ok((owner, name))
}

pub fn import_research_bundle(
    args: ResearchImportBundleArgs,
    default_imported_at: DateTime<Utc>,
) -> Result<ResearchEvidenceImportReport> {
    if args.source_exit_code.is_some() && args.source_command.is_none() {
        bail!("--source-exit-code requires --source-command");
    }

    let bundle: ResearchEvidenceBundleInput = read_json(&args.bundle)
        .with_context(|| format!("failed to read evidence bundle {}", args.bundle.display()))?;
    if bundle.schema != CODEX_RESEARCH_EVIDENCE_BUNDLE_SCHEMA {
        let schema = import_clean_text(&bundle.schema);
        bail!(
            "unsupported evidence bundle schema {schema} (expected {CODEX_RESEARCH_EVIDENCE_BUNDLE_SCHEMA})"
        );
    }

    let imported_at = args.imported_at.unwrap_or(default_imported_at);
    let bundle_summary = research_import_bundle_summary(&bundle);
    let record = research_import_evidence_record(&args, &bundle, &bundle_summary, imported_at);
    let append_result = append_evidence(AppendEvidenceArgs {
        capsule: args.capsule.clone(),
        record,
    })?;

    Ok(ResearchEvidenceImportReport {
        schema: RESEARCH_EVIDENCE_IMPORT_SCHEMA,
        imported_at,
        capsule: append_result.capsule,
        evidence_path: append_result.evidence_path,
        bundle_path: args.bundle,
        bundle: bundle_summary,
        record: append_result.record,
        evidence: append_result.evidence,
    })
}

fn research_import_bundle_summary(
    bundle: &ResearchEvidenceBundleInput,
) -> ResearchEvidenceImportBundleSummary {
    let source_count = bundle
        .ledger
        .source_count
        .max(bundle.ledger.source_ids.len())
        .max(bundle.run.cache_source_ids.len());
    let claim_count = bundle.ledger.claim_count.max(bundle.ledger.claim_ids.len());
    ResearchEvidenceImportBundleSummary {
        schema: import_clean_text(&bundle.schema),
        generated_at: bundle.generated_at,
        status: import_clean_or(&bundle.status, "unknown"),
        strict: bundle.strict,
        query: import_clean_or(&bundle.run.query, "unspecified query"),
        profile: import_clean_or(&bundle.run.profile, "unknown"),
        topic: import_clean_or(&bundle.run.topic, "unknown"),
        run_status: import_clean_or(&bundle.run.status, "unknown"),
        source_count,
        claim_count,
        cited_claims: bundle.citation_coverage.cited_claims,
        uncited_claims: bundle.citation_coverage.uncited_claims,
        missing_source_refs: import_clean_vec(
            &bundle.citation_coverage.missing_source_refs,
            RESEARCH_IMPORT_MAX_LIST_ITEMS,
        ),
        coverage: normalized_coverage(bundle.citation_coverage.coverage),
        source_freshness: bundle
            .source_freshness
            .by_status
            .iter()
            .take(RESEARCH_IMPORT_MAX_FRESHNESS_STATUSES)
            .map(|(status, count)| (import_clean_or(status, "unknown"), *count))
            .collect(),
        unknown_source_ids: import_clean_vec(
            &bundle.source_freshness.unknown_source_ids,
            RESEARCH_IMPORT_MAX_UNKNOWN_SOURCE_IDS,
        ),
        report_path: import_clean_text(&bundle.report.path),
        report_exists: bundle.report.exists,
        artifact_paths: research_import_artifact_paths(bundle, None),
        budget: research_import_budget_summary(bundle),
        provider_error_count: bundle.provider_errors.len(),
        warning_count: bundle.warnings.len(),
        failure_count: bundle.failures.len(),
        warnings: import_clean_vec(&bundle.warnings, RESEARCH_IMPORT_MAX_LIST_ITEMS),
        failures: import_clean_vec(&bundle.failures, RESEARCH_IMPORT_MAX_LIST_ITEMS),
    }
}

fn research_import_budget_summary(
    bundle: &ResearchEvidenceBundleInput,
) -> ResearchEvidenceImportBudgetSummary {
    let providers = bundle
        .budget
        .by_provider
        .iter()
        .take(RESEARCH_IMPORT_MAX_BUDGET_PROVIDERS)
        .map(|provider| ResearchEvidenceImportBudgetProvider {
            provider: import_clean_or(&provider.provider, "unknown"),
            budget: provider.budget,
            spent: provider.spent,
            remaining: provider.remaining,
        })
        .collect::<Vec<_>>();
    let spent_total = providers.iter().map(|provider| provider.spent).sum();
    let remaining_total = providers.iter().map(|provider| provider.remaining).sum();
    let status = if providers.is_empty() {
        "not_reported"
    } else if !bundle.provider_errors.is_empty() {
        "provider_errors"
    } else if providers
        .iter()
        .any(|provider| provider.budget > 0 && provider.remaining == 0)
    {
        "exhausted"
    } else if spent_total > 0 {
        "spent"
    } else {
        "unused"
    }
    .to_string();

    ResearchEvidenceImportBudgetSummary {
        status,
        spent_total,
        remaining_total,
        providers,
    }
}

fn research_import_evidence_record(
    args: &ResearchImportBundleArgs,
    bundle: &ResearchEvidenceBundleInput,
    summary: &ResearchEvidenceImportBundleSummary,
    imported_at: DateTime<Utc>,
) -> EvidenceRecord {
    let failure_fragment = if summary.failure_count == 1 {
        "1 failure".to_string()
    } else {
        format!("{} failures", summary.failure_count)
    };
    let warning_fragment = if summary.warning_count == 1 {
        "1 warning".to_string()
    } else {
        format!("{} warnings", summary.warning_count)
    };
    let summary_text = import_truncate(
        format!(
            "Research bundle {}: {}; {} source(s), {} claim(s), {:.0}% citation coverage, {}, {}",
            summary.status,
            summary.query,
            summary.source_count,
            summary.claim_count,
            summary.coverage * 100.0,
            failure_fragment,
            warning_fragment
        ),
        512,
    );

    EvidenceRecord {
        schema: EVIDENCE_SCHEMA.to_string(),
        kind: EvidenceKind::Research,
        at: imported_at,
        summary: summary_text,
        command: args.source_command.as_deref().map(import_clean_text),
        exit_code: args.source_exit_code,
        source_ids: research_import_source_ids(bundle),
        actor: None,
        tool: Some("codex-research".to_string()),
        confidence: Some(research_import_confidence(summary)),
        residual_risk: research_import_residual_risk(bundle, summary),
        artifacts: research_import_artifact_paths(bundle, Some(&args.bundle)),
    }
}

fn research_import_source_ids(bundle: &ResearchEvidenceBundleInput) -> Vec<String> {
    let mut source_ids = BTreeSet::new();
    for source_id in bundle
        .ledger
        .source_ids
        .iter()
        .chain(bundle.run.cache_source_ids.iter())
        .take(RESEARCH_IMPORT_MAX_SOURCE_IDS)
    {
        let source_id = import_clean_text(source_id);
        if !source_id.is_empty() {
            source_ids.insert(format!("codex-research:source:{source_id}"));
        }
    }
    for claim_id in bundle
        .ledger
        .claim_ids
        .iter()
        .take(RESEARCH_IMPORT_MAX_CLAIM_IDS)
    {
        let claim_id = import_clean_text(claim_id);
        if !claim_id.is_empty() {
            source_ids.insert(format!("codex-research:claim:{claim_id}"));
        }
    }
    source_ids.into_iter().collect()
}

fn research_import_artifact_paths(
    bundle: &ResearchEvidenceBundleInput,
    bundle_path: Option<&Path>,
) -> Vec<String> {
    let mut artifacts = Vec::new();
    if let Some(bundle_path) = bundle_path {
        push_import_unique(
            &mut artifacts,
            bundle_path.display().to_string(),
            RESEARCH_IMPORT_MAX_ARTIFACTS,
        );
    }
    for artifact in &bundle.artifacts {
        push_import_unique(&mut artifacts, artifact, RESEARCH_IMPORT_MAX_ARTIFACTS);
    }
    if bundle.report.exists {
        push_import_unique(
            &mut artifacts,
            &bundle.report.path,
            RESEARCH_IMPORT_MAX_ARTIFACTS,
        );
    }
    artifacts
}

fn research_import_confidence(summary: &ResearchEvidenceImportBundleSummary) -> u8 {
    let coverage = (summary.coverage * 100.0).round().clamp(0.0, 100.0) as u8;
    if summary.status != "passed" || summary.failure_count > 0 {
        coverage.min(50)
    } else if summary.warning_count > 0
        || summary.provider_error_count > 0
        || !summary.report_exists
        || !summary.unknown_source_ids.is_empty()
    {
        coverage.min(80)
    } else {
        coverage
    }
}

fn research_import_residual_risk(
    bundle: &ResearchEvidenceBundleInput,
    summary: &ResearchEvidenceImportBundleSummary,
) -> Option<String> {
    let mut parts = Vec::new();
    if !summary.failures.is_empty() {
        parts.push(format!(
            "failures: {}",
            import_preview_list(&summary.failures, 3)
        ));
    }
    if !summary.warnings.is_empty() {
        parts.push(format!(
            "warnings: {}",
            import_preview_list(&summary.warnings, 3)
        ));
    }
    if !bundle.provider_errors.is_empty() {
        let provider_errors = bundle
            .provider_errors
            .iter()
            .take(3)
            .map(|error| {
                format!(
                    "{}: {}",
                    import_clean_or(&error.provider, "unknown"),
                    import_clean_or(&error.message, "provider error")
                )
            })
            .collect::<Vec<_>>();
        parts.push(format!(
            "provider_errors: {}",
            import_preview_list(&provider_errors, 3)
        ));
    }
    if !summary.unknown_source_ids.is_empty() {
        parts.push(format!(
            "unknown_source_ids: {}",
            import_preview_list(&summary.unknown_source_ids, 5)
        ));
    }
    if !bundle.citation_coverage.uncited_claim_ids.is_empty() {
        let uncited = import_clean_vec(&bundle.citation_coverage.uncited_claim_ids, 5);
        parts.push(format!(
            "uncited_claim_ids: {}",
            import_preview_list(&uncited, 5)
        ));
    }
    if parts.is_empty() {
        None
    } else {
        Some(import_truncate(
            parts.join("; "),
            RESEARCH_IMPORT_MAX_RESIDUAL_RISK_CHARS,
        ))
    }
}

fn normalized_coverage(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn import_clean_or(value: &str, fallback: &str) -> String {
    let value = import_clean_text(value);
    if value.is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn import_clean_vec(values: &[String], limit: usize) -> Vec<String> {
    values
        .iter()
        .map(|value| import_clean_text(value))
        .filter(|value| !value.is_empty())
        .take(limit)
        .collect()
}

fn import_preview_list(values: &[String], limit: usize) -> String {
    let mut preview = values
        .iter()
        .take(limit)
        .map(|value| import_truncate(value, 180))
        .collect::<Vec<_>>();
    if values.len() > limit {
        preview.push(format!("and {} more", values.len() - limit));
    }
    preview.join(" | ")
}

fn push_import_unique(values: &mut Vec<String>, value: impl AsRef<str>, limit: usize) {
    if values.len() >= limit {
        return;
    }
    let value = import_clean_text(value.as_ref());
    if !value.is_empty() && !values.contains(&value) {
        values.push(value);
    }
}

fn import_truncate(value: impl Into<String>, max_chars: usize) -> String {
    let value = value.into();
    let mut output = String::with_capacity(value.len().min(max_chars));
    for (index, ch) in value.chars().enumerate() {
        if index >= max_chars {
            output.push_str("...");
            break;
        }
        output.push(ch);
    }
    output
}

fn import_clean_text(value: &str) -> String {
    let cleaned = value
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect::<String>();
    import_truncate(
        redact_research_import_sensitive_text(cleaned.trim()),
        RESEARCH_IMPORT_MAX_TEXT_CHARS,
    )
}

fn redact_research_import_sensitive_text(text: &str) -> String {
    let text = redact_key_assignments(
        text,
        &[
            "OPENAI_API_KEY",
            "openai_api_key",
            "ANTHROPIC_API_KEY",
            "anthropic_api_key",
            "API_KEY",
            "api_key",
            "ACCESS_TOKEN",
            "access_token",
            "TOKEN",
            "token",
            "SECRET",
            "secret",
            "PASSWORD",
            "password",
            "AUTHORIZATION",
            "authorization",
            "BODY",
            "body",
        ],
    );
    redact_sensitive_text(text)
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
            "bun-platform-core-clippy",
            "bun-platform-core Clippy",
            [
                "cargo",
                "clippy",
                "-p",
                "bun-platform-core",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means Bun platform core has Rust lints or warnings that must be fixed before review.",
        ),
        policy_gate(
            "bun-platform-clippy",
            "bun-platform Clippy",
            [
                "cargo",
                "clippy",
                "-p",
                "bun-platform",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the temporary Bun platform shim has Rust lints or warnings that must be fixed before review.",
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
            "bun-platform-core-check",
            "bun-platform-core cargo check",
            ["cargo", "check", "-p", "bun-platform-core"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means Bun platform core does not typecheck.",
        ),
        policy_gate(
            "bun-platform-check",
            "bun-platform cargo check",
            ["cargo", "check", "-p", "bun-platform"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the temporary Bun platform shim does not typecheck.",
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
            "bun-platform-core-test",
            "bun-platform-core tests",
            ["cargo", "test", "-p", "bun-platform-core"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means Bun platform audit, fix, reference, or fixture behavior regressed.",
        ),
        policy_gate(
            "bun-platform-test",
            "bun-platform tests",
            ["cargo", "test", "-p", "bun-platform"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the temporary Bun platform CLI contract regressed.",
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
            "codex-dev-completion-zsh",
            "codex-dev zsh completion smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-dev",
                "--",
                "completions",
                "zsh",
            ],
            "docs/runbooks/global-cli-workflow.md#completion-and-manpage-smokes",
            ["cargo"],
            "Failure means codex-dev cannot generate shell completions from its Clap contract.",
        ),
        policy_gate(
            "codex-dev-manpage",
            "codex-dev manpage smoke",
            ["cargo", "run", "-q", "-p", "codex-dev", "--", "manpage"],
            "docs/runbooks/global-cli-workflow.md#completion-and-manpage-smokes",
            ["cargo"],
            "Failure means codex-dev cannot generate a manpage from its Clap contract.",
        ),
        policy_gate(
            "bun-platform-help",
            "bun-platform help smoke",
            ["cargo", "run", "-q", "-p", "bun-platform", "--", "--help"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the temporary Bun platform shim cannot render its top-level Clap contract.",
        ),
        policy_gate(
            "bun-platform-completion-zsh",
            "bun-platform zsh completion smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "bun-platform",
                "--",
                "completions",
                "zsh",
            ],
            "docs/runbooks/global-cli-workflow.md#completion-and-manpage-smokes",
            ["cargo"],
            "Failure means bun-platform cannot generate shell completions from its Clap contract.",
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
        policy_profile_explain_gate(PolicyProfile::CodexDev),
        policy_docs_check_gate(),
        policy_gate(
            "codex-dev-skills-inventory-smoke",
            "codex-dev skills inventory smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-dev",
                "--",
                "--json",
                "skills",
                "inventory",
            ],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the skill inventory JSON contract regressed.",
        ),
        policy_gate(
            "codex-dev-bun-doctor-smoke",
            "codex-dev Bun doctor smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-dev",
                "--",
                "--json",
                "bun",
                "doctor",
            ],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the native Bun platform doctor contract regressed.",
        ),
        policy_gate(
            "codex-dev-bun-audit-smoke",
            "codex-dev Bun audit smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-dev",
                "--",
                "--json",
                "bun",
                "audit",
                "--root",
                "crates/bun-platform-core/fixtures/github-actions",
            ],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the native Bun audit JSON contract regressed.",
        ),
        policy_gate(
            "codex-dev-bun-fixes-plan-smoke",
            "codex-dev Bun fixes plan smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-dev",
                "--",
                "--json",
                "bun",
                "fixes",
                "plan",
                "--root",
                "crates/bun-platform-core/fixtures/safe-fixes",
            ],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the native Bun fix-planning JSON contract regressed.",
        ),
        policy_gate(
            "codex-dev-bun-references-status-smoke",
            "codex-dev Bun references status smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-dev",
                "--",
                "--json",
                "bun",
                "references",
                "status",
            ],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means the native Bun reference status contract regressed.",
        ),
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
        policy_gate(
            "codex-dev-tui-completion-zsh",
            "codex-dev-tui zsh completion smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-dev-tui",
                "--",
                "completions",
                "zsh",
            ],
            "docs/runbooks/global-cli-workflow.md#completion-and-manpage-smokes",
            ["cargo"],
            "Failure means codex-dev-tui cannot generate shell completions from its Clap contract.",
        ),
        policy_gate(
            "codex-dev-tui-manpage",
            "codex-dev-tui manpage smoke",
            ["cargo", "run", "-q", "-p", "codex-dev-tui", "--", "manpage"],
            "docs/runbooks/global-cli-workflow.md#completion-and-manpage-smokes",
            ["cargo"],
            "Failure means codex-dev-tui cannot generate a manpage from its Clap contract.",
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
        policy_gate(
            "codex-research-completion-zsh",
            "codex-research zsh completion smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-research",
                "--",
                "completions",
                "zsh",
            ],
            "docs/runbooks/global-cli-workflow.md#completion-and-manpage-smokes",
            ["cargo"],
            "Failure means codex-research cannot generate shell completions from its Clap contract.",
        ),
        policy_gate(
            "codex-research-manpage",
            "codex-research manpage smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-research",
                "--",
                "manpage",
            ],
            "docs/runbooks/global-cli-workflow.md#completion-and-manpage-smokes",
            ["cargo"],
            "Failure means codex-research cannot generate a manpage from its Clap contract.",
        ),
    ]
}

fn gsap_audit_gates() -> Vec<PolicyGate> {
    vec![
        policy_gate(
            "gsap-audit-clippy",
            "gsap-audit Clippy",
            [
                "cargo",
                "clippy",
                "-p",
                "gsap-audit-core",
                "-p",
                "gsap-audit",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            "docs/runbooks/validation.md#gsap-audit-cli",
            ["cargo"],
            "Failure means gsap-audit has Rust lints or warnings that must be fixed before review.",
        ),
        policy_gate(
            "gsap-audit-check",
            "gsap-audit cargo check",
            [
                "cargo",
                "check",
                "-p",
                "gsap-audit-core",
                "-p",
                "gsap-audit",
            ],
            "docs/runbooks/validation.md#gsap-audit-cli",
            ["cargo"],
            "Failure means gsap-audit does not typecheck.",
        ),
        policy_gate(
            "gsap-audit-test",
            "gsap-audit tests",
            ["cargo", "test", "-p", "gsap-audit-core", "-p", "gsap-audit"],
            "docs/runbooks/validation.md#gsap-audit-cli",
            ["cargo"],
            "Failure means gsap-audit rule or CLI behavior regressed.",
        ),
        policy_gate(
            "gsap-audit-doctor",
            "gsap-audit doctor smoke",
            ["cargo", "run", "-q", "-p", "gsap-audit", "--", "doctor"],
            "docs/runbooks/validation.md#gsap-audit-cli",
            ["cargo"],
            "Failure means gsap-audit cannot emit its rule catalog.",
        ),
        policy_gate(
            "gsap-audit-completion-zsh",
            "gsap-audit zsh completion smoke",
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "gsap-audit",
                "--",
                "completions",
                "zsh",
            ],
            "docs/runbooks/global-cli-workflow.md#completion-and-manpage-smokes",
            ["cargo"],
            "Failure means gsap-audit cannot generate shell completions from its Clap contract.",
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
                "subagents/codex/scripts",
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
                "subagents/codex/agents",
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
        bootstrap_status_gate(),
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
            "codex-subagents-release-manifest",
            "validate Codex subagent release manifest",
            [
                "python3",
                "subagents/codex/scripts/sync_agents.py",
                "--validate-release-manifest",
            ],
            "docs/runbooks/validation.md#bootstrap-packs",
            ["python3"],
            "Failure means the Codex subagent release manifest is invalid.",
        ),
        policy_gate(
            "codex-subagents-global-dry-run",
            "Codex subagent global install dry-run",
            [
                "python3",
                "subagents/codex/scripts/sync_agents.py",
                "--global",
                "--all-overlays",
                "--dry-run",
            ],
            "docs/runbooks/validation.md#bootstrap-packs",
            ["python3"],
            "Failure means global subagent installation planning regressed.",
        ),
        policy_gate(
            "codex-subagents-validate-sources",
            "Codex subagent source validation",
            [
                "python3",
                "subagents/codex/scripts/sync_agents.py",
                "--global",
                "--all-overlays",
                "--validate-sources",
            ],
            "docs/runbooks/validation.md#bootstrap-packs",
            ["python3"],
            "Failure means Codex subagent source pack validation regressed.",
        ),
        policy_gate(
            "bootstrap-local-overlays-ignored",
            "prove private overlay paths stay gitignored",
            [
                "bash",
                "-lc",
                "for path in subagents/codex/overlays.local.json subagents/codex/roles.local.json subagents/codex/agents/overlays/private-repo/private_repo_reviewer.toml; do git check-ignore -q -- \"$path\" || exit 1; done",
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
    append_unique_gates(
        &mut gates,
        vec![policy_profile_explain_gate(PolicyProfile::Release)],
    );
    append_unique_gates(&mut gates, codex_dev_tui_gates());
    append_unique_gates(&mut gates, codex_research_gates());
    append_unique_gates(&mut gates, gsap_audit_gates());
    append_unique_gates(&mut gates, docs_gates());
    append_unique_gates(&mut gates, vec![bootstrap_pack_validate_gate()]);
    append_unique_gates(&mut gates, skills_gates());
    append_unique_gates(&mut gates, supply_chain_gates());
    gates
}

fn full_local_gates() -> Vec<PolicyGate> {
    let mut gates = Vec::new();
    append_unique_gates(&mut gates, codex_dev_gates());
    append_unique_gates(
        &mut gates,
        vec![policy_profile_explain_gate(PolicyProfile::FullLocal)],
    );
    append_unique_gates(&mut gates, codex_dev_tui_gates());
    append_unique_gates(&mut gates, codex_research_gates());
    append_unique_gates(&mut gates, gsap_audit_gates());
    append_unique_gates(&mut gates, local_cli_install_smoke_gates());
    append_unique_gates(&mut gates, bootstrap_install_gates());
    append_unique_gates(&mut gates, skills_gates());
    append_unique_gates(&mut gates, docs_gates());
    append_unique_gates(&mut gates, supply_chain_gates());
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

fn bootstrap_status_gate() -> PolicyGate {
    policy_gate(
        "codex-dev-bootstrap-status",
        "codex-dev bootstrap status smoke",
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "codex-dev",
            "--",
            "--json",
            "bootstrap",
            "status",
        ],
        "docs/runbooks/validation.md#bootstrap-packs",
        ["cargo"],
        "Failure means codex-dev cannot emit the read-only bootstrap_status.v1 contract.",
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

fn policy_profile_explain_gate(profile: PolicyProfile) -> PolicyGate {
    let gate_slug = policy_profile_gate_slug(profile);
    let profile_arg = profile.to_string();
    let source = policy_profile_explain_source(profile);
    PolicyGate {
        id: format!("{gate_slug}-policy-explain"),
        name: format!("{gate_slug} policy explain smoke"),
        command: [
            "cargo",
            "run",
            "-q",
            "-p",
            "codex-dev",
            "--",
            "--json",
            "policy",
            "explain",
            "--profile",
        ]
        .into_iter()
        .map(str::to_string)
        .chain(std::iter::once(profile_arg.clone()))
        .collect(),
        source: source.to_string(),
        working_directory: ".".to_string(),
        required_tools: vec!["cargo".to_string()],
        required: true,
        network: false,
        secrets: false,
        failure_interpretation: format!(
            "Failure means the {profile_arg} policy explanation contract cannot be emitted as JSON."
        ),
    }
}

fn policy_profile_explain_source(profile: PolicyProfile) -> &'static str {
    match profile {
        PolicyProfile::Release | PolicyProfile::FullLocal => {
            "docs/runbooks/validation.md#full-local-gate"
        }
        _ => "docs/runbooks/validation.md#codex-dev-operating-layer",
    }
}

fn policy_profile_gate_slug(profile: PolicyProfile) -> &'static str {
    match profile {
        PolicyProfile::CodexDev => "codex-dev",
        PolicyProfile::CodexDevTui => "codex-dev-tui",
        PolicyProfile::CodexResearch => "codex-research",
        PolicyProfile::Skills => "skills",
        PolicyProfile::BootstrapInstall => "bootstrap-install",
        PolicyProfile::Docs => "docs",
        PolicyProfile::Release => "release",
        PolicyProfile::FullLocal => "full-local",
    }
}

fn supply_chain_gates() -> Vec<PolicyGate> {
    vec![
        policy_gate(
            "cargo-metadata-locked",
            "locked Cargo metadata smoke",
            [
                "cargo",
                "metadata",
                "--locked",
                "--no-deps",
                "--format-version",
                "1",
            ],
            "docs/runbooks/local-release-supply-chain.md#release-baseline-gates",
            ["cargo"],
            "Failure means workspace manifests or Cargo.lock no longer resolve with the committed lockfile.",
        ),
        policy_gate(
            "cargo-tree-duplicates",
            "Cargo duplicate dependency report",
            ["cargo", "tree", "-d", "--target", "all"],
            "docs/runbooks/local-release-supply-chain.md#duplicate-dependency-baseline",
            ["cargo"],
            "Failure means Cargo could not build the duplicate dependency report; duplicate output itself is audited in the release runbook.",
        ),
        policy_gate(
            "cargo-deny-policy",
            "cargo-deny license, ban, and source policy",
            ["cargo", "deny", "check", "bans", "licenses", "sources"],
            "docs/runbooks/local-release-supply-chain.md#release-baseline-gates",
            ["cargo", "cargo-deny"],
            "Failure means the configured license, dependency ban, or source allowlist policy rejected the workspace.",
        ),
        policy_gate(
            "cargo-package-codex-dev-core-list",
            "codex-dev-core package file list",
            ["cargo", "package", "--list", "-p", "codex-dev-core"],
            "docs/runbooks/local-release-supply-chain.md#package-dry-runs",
            ["cargo"],
            "Failure means codex-dev-core is missing package metadata or would package unexpected invalid content.",
        ),
        policy_gate(
            "cargo-package-codex-dev-list",
            "codex-dev package file list",
            ["cargo", "package", "--list", "-p", "codex-dev"],
            "docs/runbooks/local-release-supply-chain.md#package-dry-runs",
            ["cargo"],
            "Failure means codex-dev is missing package metadata or would package unexpected invalid content.",
        ),
        policy_gate(
            "cargo-package-bun-platform-core-list",
            "bun-platform-core package file list",
            ["cargo", "package", "--list", "-p", "bun-platform-core"],
            "docs/runbooks/local-release-supply-chain.md#package-dry-runs",
            ["cargo"],
            "Failure means bun-platform-core is missing package metadata or would package unexpected invalid content.",
        ),
        policy_gate(
            "cargo-package-bun-platform-list",
            "bun-platform package file list",
            ["cargo", "package", "--list", "-p", "bun-platform"],
            "docs/runbooks/local-release-supply-chain.md#package-dry-runs",
            ["cargo"],
            "Failure means bun-platform is missing package metadata or would package unexpected invalid content.",
        ),
        policy_gate(
            "cargo-package-codex-dev-tui-list",
            "codex-dev-tui package file list",
            ["cargo", "package", "--list", "-p", "codex-dev-tui"],
            "docs/runbooks/local-release-supply-chain.md#package-dry-runs",
            ["cargo"],
            "Failure means codex-dev-tui is missing package metadata or would package unexpected invalid content.",
        ),
        policy_gate(
            "cargo-package-codex-research-list",
            "codex-research package file list",
            ["cargo", "package", "--list", "-p", "codex-research"],
            "docs/runbooks/local-release-supply-chain.md#package-dry-runs",
            ["cargo"],
            "Failure means codex-research is missing package metadata or would package unexpected invalid content.",
        ),
    ]
}

fn local_cli_install_smoke_gates() -> Vec<PolicyGate> {
    vec![
        local_cli_install_smoke_gate("codex-research", "crates/codex-research"),
        local_cli_install_smoke_gate("codex-dev", "crates/codex-dev"),
        local_cli_install_smoke_gate("bun-platform", "crates/bun-platform"),
        local_cli_install_smoke_gate("codex-dev-tui", "crates/codex-dev-tui"),
        local_cli_install_smoke_gate("gsap-audit", "crates/gsap-audit"),
    ]
}

fn local_cli_install_smoke_gate(binary: &'static str, crate_path: &'static str) -> PolicyGate {
    let artifact_smoke = match binary {
        "bun-platform" => format!("\"$root/bin/{binary}\" completions zsh >/dev/null"),
        "gsap-audit" => {
            format!(
                "\"$root/bin/{binary}\" doctor >/dev/null && \"$root/bin/{binary}\" completions zsh >/dev/null"
            )
        }
        _ => {
            format!(
                "\"$root/bin/{binary}\" completions zsh >/dev/null && \"$root/bin/{binary}\" manpage >/dev/null"
            )
        }
    };
    let command = format!(
        "repo=$(pwd); root=\"$repo/target/codex-dev-install-smoke/{binary}\"; rm -rf \"$root\"; cargo install --path {crate_path} --locked --offline --force --root \"$root\"; (cd /tmp && \"$root/bin/{binary}\" --help >/dev/null && {artifact_smoke})"
    );
    PolicyGate {
        id: format!("cargo-install-{binary}-smoke"),
        name: format!("{binary} isolated cargo install smoke"),
        command: vec!["bash".to_string(), "-lc".to_string(), command],
        source: "docs/runbooks/global-cli-workflow.md#install-smoke-gates".to_string(),
        working_directory: ".".to_string(),
        required_tools: vec!["bash".to_string(), "cargo".to_string()],
        required: true,
        network: false,
        secrets: false,
        failure_interpretation: format!(
            "Failure means {binary} cannot be installed into an isolated Cargo root and executed from another directory."
        ),
    }
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
        source: "codex-dev pr review / gh-pr-review-fix / gh".to_string(),
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
    fn hosted_diagnostics_redact_token_like_values() {
        let excerpt = diagnostic_excerpt(
            b"Authorization: Bearer ghp_secret123\nexport GH_TOKEN=plain-secret\nGITHUB_TOKEN = spaced-secret\nGH_ENTERPRISE_TOKEN=enterprise-secret\nGITHUB_ENTERPRISE_TOKEN = enterprise-spaced-secret\nNOT_GH_TOKEN=kept github_pat_abc123",
        )
        .expect("excerpt");

        assert!(!excerpt.contains("ghp_secret123"));
        assert!(!excerpt.contains("plain-secret"));
        assert!(!excerpt.contains("spaced-secret"));
        assert!(!excerpt.contains("enterprise-secret"));
        assert!(!excerpt.contains("enterprise-spaced-secret"));
        assert!(!excerpt.contains("github_pat_abc123"));
        assert!(excerpt.contains("NOT_GH_TOKEN=kept"));
        assert!(excerpt.contains("[redacted]"));
    }

    #[test]
    fn orchestration_output_allows_missing_agent_on_write_commands() {
        let report = OrchestrationRunReport {
            schema: "orchestration_run.v1".to_string(),
            capsule: PathBuf::from("/tmp/capsule"),
            batch_id: "review".to_string(),
            status: "planned".to_string(),
            task: None,
            mode: None,
            scope: None,
            wait_policy: None,
            rendezvous_required: None,
            expected_roles: vec!["reviewer".to_string(), "test_runner".to_string()],
            agents: Vec::new(),
            completion: codex_dev_core::OrchestrationCompletionReport {
                expected: 2,
                recorded: 1,
                terminal: 1,
                human_verified: 1,
                missing: vec!["test_runner".to_string()],
                extra: Vec::new(),
                synthesis_completed: false,
                complete: false,
            },
            synthesis_status: None,
            registry_issues: Vec::new(),
            diagnostics: vec![codex_dev_core::OrchestrationDiagnostic {
                severity: OrchestrationDiagnosticSeverity::Error,
                code: "missing_agent".to_string(),
                message: "missing expected role: test_runner".to_string(),
                role: Some("test_runner".to_string()),
            }],
            checked_at: "2026-05-09T05:30:00Z".parse().unwrap(),
            stale_after_minutes: 120,
        };

        let write_output =
            orchestration_output("orchestration record", report, false).expect("write output");
        assert!(write_output.ok);
        assert!(
            write_output
                .human
                .contains("with 1 error diagnostic(s), 1/2 role(s) verified")
        );

        let verify_report: OrchestrationRunReport =
            serde_json::from_value(write_output.result).expect("report json");
        let verify_output = orchestration_output("orchestration verify", verify_report, true)
            .expect("verify output");
        assert!(!verify_output.ok);
        assert!(
            verify_output
                .human
                .contains("incomplete: 1 blocking diagnostic(s), 1/2 role(s) verified")
        );
    }

    #[test]
    fn readiness_attempt_checked_at_uses_poll_interval_with_monotonic_floor() {
        let generated_at: DateTime<Utc> = "2026-05-09T05:05:00Z".parse().expect("timestamp");

        assert_eq!(
            readiness_attempt_checked_at(generated_at, 1, 60).expect("attempt 1"),
            generated_at
        );
        assert_eq!(
            readiness_attempt_checked_at(generated_at, 3, 60).expect("attempt 3"),
            generated_at + TimeDelta::seconds(120)
        );
        assert_eq!(
            readiness_attempt_checked_at(generated_at, 2, 0).expect("zero interval"),
            generated_at + TimeDelta::seconds(1)
        );
    }

    #[cfg(unix)]
    #[test]
    fn pr_agent_output_dir_rejects_symlinked_source_root() {
        let temp = tempdir().expect("tempdir");
        let capsule = temp.path().join("capsule");
        let target = temp.path().join("target");
        fs::create_dir_all(&capsule).expect("capsule dir");
        fs::create_dir_all(&target).expect("target dir");
        std::os::unix::fs::symlink(&target, capsule.join("pr-agent-sources"))
            .expect("symlink source root");

        let error = prepare_pr_agent_output_dir(
            &capsule,
            "2026-05-09T05:00:00Z".parse().expect("timestamp"),
        )
        .expect_err("symlink rejected");

        assert!(error.to_string().contains("symlinked PR agent source"));
    }

    #[cfg(unix)]
    #[test]
    fn pr_agent_state_report_rejects_symlinked_path() {
        let temp = tempdir().expect("tempdir");
        let report = temp.path().join("pr-agent-state.json");
        let target = temp.path().join("target.json");
        fs::write(&target, "{}").expect("target");
        std::os::unix::fs::symlink(&target, &report).expect("symlink report");

        let error =
            ensure_pr_agent_report_path_safe(&report).expect_err("symlinked report rejected");

        assert!(error.to_string().contains("symlinked PR agent state"));
    }

    #[test]
    fn pr_agent_required_nonzero_json_source_is_failed() {
        let temp = tempdir().expect("tempdir");
        let args = PrAgentArgs {
            capsule: temp.path().join("capsule"),
            repo: "BjornMelin/dev-skills".to_string(),
            number: 47,
            checked_at: None,
            source_dir: None,
        };
        let output_dir = temp.path().join("sources");
        fs::create_dir_all(&output_dir).expect("source dir");
        let spec = pr_agent_source_spec(
            "gh-pr-view",
            "github-pr-view",
            "gh-pr-view.json",
            vec![
                "sh",
                "-c",
                "printf '{\"number\":47,\"url\":\"https://github.com/BjornMelin/dev-skills/pull/47\",\"state\":\"OPEN\"}'; exit 2",
            ],
            Some(PrRecordSourceKind::GhPrView),
        );

        let capture = capture_pr_agent_source(
            &args,
            &spec,
            &output_dir,
            "2026-05-09T05:00:00Z".parse().expect("timestamp"),
        )
        .expect("capture source");

        assert_eq!(capture.source.status, PrAgentSourceStatus::Failed);
        assert!(capture.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == PrAgentSeverity::Error
                && diagnostic.message.contains("exited with status 2")
        }));
    }

    #[test]
    fn pr_agent_failed_capture_writes_failure_artifact() {
        let temp = tempdir().expect("tempdir");
        let source_dir = temp.path().join("fixtures");
        let output_dir = temp.path().join("sources");
        fs::create_dir_all(&source_dir).expect("fixture dir");
        fs::create_dir_all(&output_dir).expect("source dir");
        let args = PrAgentArgs {
            capsule: temp.path().join("capsule"),
            repo: "BjornMelin/dev-skills".to_string(),
            number: 47,
            checked_at: None,
            source_dir: Some(source_dir),
        };
        let spec = pr_agent_source_spec(
            "gh-pr-view",
            "github-pr-view",
            "gh-pr-view.json",
            vec!["gh", "pr", "view", "47"],
            Some(PrRecordSourceKind::GhPrView),
        );

        let capture = capture_pr_agent_source(
            &args,
            &spec,
            &output_dir,
            "2026-05-09T05:00:00Z".parse().expect("timestamp"),
        )
        .expect("capture source");

        assert_eq!(capture.source.status, PrAgentSourceStatus::Failed);
        assert!(capture.path.is_file());
        let failure: Value =
            read_json(&capture.path).expect("failure artifact should be valid json");
        assert_eq!(failure["schema"], "codex-dev.pr-agent-source-failure.v1");
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
    fn run_from_skills_sync_kimi_emits_filtered_json_envelope() {
        let temp = tempdir().expect("tempdir");
        let codex = temp.path().join(".codex");
        let agents = temp.path().join(".agents");
        let kimi = temp.path().join(".kimi-code");
        let project = temp.path().join("project");
        let enabled = agents.join("skills/shadcn");
        let disabled = agents.join("skills/disabled-global");
        fs::create_dir_all(&enabled).expect("enabled skill dir");
        fs::create_dir_all(&disabled).expect("disabled skill dir");
        fs::create_dir_all(&codex).expect("codex dir");
        fs::create_dir_all(&project).expect("project dir");
        fs::write(
            enabled.join("SKILL.md"),
            "---\nname: shadcn\ndescription: Test skill.\n---\n",
        )
        .expect("enabled skill");
        fs::write(
            disabled.join("SKILL.md"),
            "---\nname: disabled-global\ndescription: Test skill.\n---\n",
        )
        .expect("disabled skill");
        fs::write(
            codex.join("config.toml"),
            r#"[[skills.config]]
name = "disabled-global"
enabled = false
"#,
        )
        .expect("codex config");

        let output = run_from([
            "codex-dev",
            "--json",
            "skills",
            "sync-kimi",
            "--dry-run",
            "--scope",
            "global-only",
            "--codex-home",
            codex.to_str().expect("codex path"),
            "--agents-home",
            agents.to_str().expect("agents path"),
            "--kimi-home",
            kimi.to_str().expect("kimi path"),
            "--project-root",
            project.to_str().expect("project path"),
            "--checked-at",
            "2026-06-03T00:00:00Z",
        ])
        .expect("run");
        let value: Value = serde_json::from_str(&output).expect("json output");

        assert_eq!(value["schema"], OUTPUT_SCHEMA);
        assert_eq!(value["ok"], true);
        assert_eq!(value["command"], "skills sync-kimi");
        assert_eq!(value["result"]["schema"], "codex-dev.kimi-sync.v1");
        assert_eq!(value["result"]["dryRun"], true);
        assert_eq!(value["result"]["summary"]["included"], 1);
        assert_eq!(value["result"]["summary"]["excluded"], 1);
        assert_eq!(value["result"]["included"][0]["name"], "shadcn");
        assert_eq!(value["result"]["excluded"][0]["name"], "disabled-global");
        assert!(!value["result"]["mirrorRoot"].as_str().unwrap().is_empty());
        assert!(!kimi.join("codex-sync").exists());
    }

    #[test]
    fn run_from_skills_sync_kimi_reports_flag_invariant_errors() {
        let temp = tempdir().expect("tempdir");
        let codex = temp.path().join(".codex");
        let agents = temp.path().join(".agents");
        let kimi = temp.path().join(".kimi-code");
        let project = temp.path().join("project");
        fs::create_dir_all(&codex).expect("codex dir");
        fs::create_dir_all(&agents).expect("agents dir");
        fs::create_dir_all(&kimi).expect("kimi dir");
        fs::create_dir_all(&project).expect("project dir");

        let cases: Vec<(&str, Vec<String>, &str)> = vec![
            (
                "install wrapper without apply",
                vec!["--install-wrapper".to_string()],
                "--install-wrapper writes ~/.local/bin/kimi-codex and requires --apply",
            ),
            (
                "launch without apply",
                vec!["--launch".to_string()],
                "--launch requires --apply",
            ),
            (
                "launch with json",
                vec!["--apply".to_string(), "--launch".to_string()],
                "--launch is interactive and cannot be combined with --json",
            ),
            (
                "passthrough without launch",
                vec!["--".to_string(), "--version".to_string()],
                "Kimi passthrough arguments require --launch",
            ),
        ];

        for (label, extra_args, expected_message) in cases {
            let mut args = vec![
                "codex-dev".to_string(),
                "--json".to_string(),
                "skills".to_string(),
                "sync-kimi".to_string(),
                "--codex-home".to_string(),
                codex.display().to_string(),
                "--agents-home".to_string(),
                agents.display().to_string(),
                "--kimi-home".to_string(),
                kimi.display().to_string(),
                "--project-root".to_string(),
                project.display().to_string(),
            ];
            args.extend(extra_args);

            let output = run_from(args).unwrap_or_else(|error| panic!("{label}: {error:#}"));
            let value: Value = serde_json::from_str(&output).expect("json output");
            assert_eq!(value["ok"], false, "{label}");
            assert_eq!(value["command"], "skills sync-kimi", "{label}");
            assert!(
                value["result"]["error"]["message"]
                    .as_str()
                    .expect("message")
                    .contains(expected_message),
                "{label}: {}",
                value["result"]["error"]["message"]
            );
        }
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
        let all_commands = policy_doc_block_expected_commands(PolicyDocBlockKind::AllProfiles);
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
    fn policy_docs_expected_commands_include_explain_smoke() {
        let commands = policy_doc_block_expected_commands(PolicyDocBlockKind::Smoke);

        assert_eq!(
            commands,
            vec![
                policy_manifest_command(PolicyProfile::CodexDev),
                policy_explain_command(PolicyProfile::CodexDev),
                policy_manifest_command(PolicyProfile::FullLocal),
                policy_explain_command(PolicyProfile::FullLocal),
            ]
        );
    }

    #[test]
    fn policy_explain_doc_error_preserves_sanitized_failure_reason() {
        let raw_error = "failed to read /home/example/dev-skills/docs/runbooks/validation.md: permission denied";
        let error = policy_explain_doc_error(
            Some(raw_error.to_string()),
            "docs/runbooks/validation.md",
            false,
        );

        assert_eq!(
            error.as_deref(),
            Some("failed to read docs/runbooks/validation.md: permission denied")
        );

        let opted_in_error = policy_explain_doc_error(
            Some(raw_error.to_string()),
            "docs/runbooks/validation.md",
            true,
        );
        assert_eq!(opted_in_error.as_deref(), Some(raw_error));
    }

    #[test]
    fn policy_explain_error_redaction_preserves_reasons() {
        let error = anyhow::anyhow!(
            "failed to read /home/example/dev-skills/docs/runbooks/validation.md: permission denied"
        );
        let message = policy_explain_redacted_error_message(&error);
        assert!(message.contains("failed to read <local-path>: permission denied"));
        assert!(message.contains("--include-local-paths"));
        assert!(!message.contains("/home/example"));

        let windows_error = anyhow::anyhow!(
            "failed to read C:\\Users\\example\\dev-skills\\AGENTS.md: access denied"
        );
        let windows_message = policy_explain_redacted_error_message(&windows_error);
        assert!(windows_message.contains("failed to read <local-path>: access denied"));
        assert!(!windows_message.contains("C:\\Users\\example"));

        let unc_error = anyhow::anyhow!(
            "failed to read \\\\server\\share\\dev-skills\\AGENTS.md: access denied"
        );
        let unc_message = policy_explain_redacted_error_message(&unc_error);
        assert!(unc_message.contains("failed to read <local-path>: access denied"));
        assert!(!unc_message.contains("\\\\server\\share"));

        let verbatim_error = anyhow::anyhow!(
            "failed to read \\\\?\\C:\\Users\\example\\dev-skills\\AGENTS.md: access denied"
        );
        let verbatim_message = policy_explain_redacted_error_message(&verbatim_error);
        assert!(verbatim_message.contains("failed to read <local-path>: access denied"));
        assert!(!verbatim_message.contains("\\\\?\\C:\\Users\\example"));

        let relative_path_error = anyhow::anyhow!(
            "repo root must contain docs/runbooks/validation.md: /home/example/dev-skills missing"
        );
        let relative_path_message = policy_explain_redacted_error_message(&relative_path_error);
        assert!(
            relative_path_message.contains(
                "repo root must contain docs/runbooks/validation.md: <local-path> missing"
            )
        );
        assert!(!relative_path_message.contains("docs<local-path>"));
        assert!(!relative_path_message.contains("/home/example"));
    }

    #[test]
    fn policy_explain_redacts_direct_api_errors_by_default() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("missing-policy-docs");
        fs::create_dir_all(&root).expect("repo root");
        fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("cargo toml");
        let checked_at = "2026-05-09T05:00:00Z".parse().unwrap();

        let error = policy_explain(
            PolicyExplainArgs {
                profile: PolicyProfile::CodexDev,
                repo_root: Some(root.clone()),
                include_local_paths: false,
                checked_at: None,
            },
            checked_at,
        )
        .expect_err("missing policy docs should fail");
        let message = format!("{error:#}");
        assert!(message.contains("repo root must contain docs/runbooks/validation.md"));
        assert!(message.contains("<local-path>"));
        assert!(message.contains("--include-local-paths"));
        assert!(!message.contains("docs<local-path>"));
        assert!(!message.contains(&root.display().to_string()));

        let error_with_paths = policy_explain(
            PolicyExplainArgs {
                profile: PolicyProfile::CodexDev,
                repo_root: Some(root.clone()),
                include_local_paths: true,
                checked_at: None,
            },
            checked_at,
        )
        .expect_err("missing policy docs should fail with paths");
        let message_with_paths = format!("{error_with_paths:#}");
        assert!(message_with_paths.contains(&root.display().to_string()));
        assert!(!message_with_paths.contains("--include-local-paths"));
    }

    #[test]
    fn policy_explain_reports_read_only_gate_context() {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = crate_dir
            .parent()
            .and_then(Path::parent)
            .expect("repo root")
            .to_path_buf();
        let checked_at = "2026-05-09T05:00:00Z".parse().unwrap();

        let report = policy_explain(
            PolicyExplainArgs {
                profile: PolicyProfile::CodexDev,
                repo_root: Some(repo_root.clone()),
                include_local_paths: false,
                checked_at: None,
            },
            checked_at,
        )
        .expect("policy explain");
        let manifest = policy_manifest(PolicyProfile::CodexDev, checked_at);

        assert_eq!(report.schema, POLICY_EXPLAIN_SCHEMA);
        assert_eq!(report.profile, PolicyProfile::CodexDev);
        assert_eq!(report.checked_at, checked_at);
        assert_eq!(report.manifest_schema, POLICY_GATES_SCHEMA);
        assert_eq!(report.gate_count, manifest.gates.len());
        assert_eq!(report.docs_mirror.repo_root, None);
        assert_eq!(report.docs_mirror.status, "current");
        assert!(report.docs_mirror.passed);
        assert!(!report.required_tools.is_empty());
        assert!(report.required_tools.iter().all(|tool| tool.path.is_none()));
        assert!(report.gates.iter().all(|gate| !gate.purpose.is_empty()
            && !gate.expected_artifacts.is_empty()
            && gate.network_posture == "local_only"
            && gate.secrets_posture == "no_secrets_required"));

        let explain_gate = report
            .gates
            .iter()
            .find(|gate| gate.id == "codex-dev-policy-explain")
            .expect("policy explain gate");
        assert_eq!(explain_gate.docs_mirror_status, "current");
        assert!(
            explain_gate
                .expected_artifacts
                .contains(&"policy_explain.v1 JSON on stdout".to_string())
        );
        let completion_gate = report
            .gates
            .iter()
            .find(|gate| gate.id == "codex-dev-completion-zsh")
            .expect("completion gate");
        assert_eq!(completion_gate.docs_mirror_status, "not_mirrored");

        let full_local_report = policy_explain(
            PolicyExplainArgs {
                profile: PolicyProfile::FullLocal,
                repo_root: Some(repo_root.clone()),
                include_local_paths: false,
                checked_at: None,
            },
            checked_at,
        )
        .expect("full local policy explain");
        let install_gate = full_local_report
            .gates
            .iter()
            .find(|gate| gate.id == "cargo-install-codex-dev-smoke")
            .expect("cargo install gate");
        assert!(
            install_gate
                .expected_artifacts
                .contains(&"isolated install root under target/codex-dev-install-smoke/codex-dev on filesystem".to_string())
        );
        let release_report = policy_explain(
            PolicyExplainArgs {
                profile: PolicyProfile::Release,
                repo_root: Some(repo_root.clone()),
                include_local_paths: false,
                checked_at: None,
            },
            checked_at,
        )
        .expect("release policy explain");
        let release_explain_gate = release_report
            .gates
            .iter()
            .find(|gate| gate.id == "release-policy-explain")
            .expect("release policy explain gate");
        assert_eq!(
            release_explain_gate.source,
            "docs/runbooks/validation.md#full-local-gate"
        );
        assert_eq!(release_explain_gate.docs_mirror_status, "current");
        let full_local_explain_gate = full_local_report
            .gates
            .iter()
            .find(|gate| gate.id == "full-local-policy-explain")
            .expect("full local policy explain gate");
        assert_eq!(
            full_local_explain_gate.command,
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "codex-dev",
                "--",
                "--json",
                "policy",
                "explain",
                "--profile",
                "full_local"
            ]
        );

        let report_with_paths = policy_explain(
            PolicyExplainArgs {
                profile: PolicyProfile::CodexDev,
                repo_root: Some(repo_root.clone()),
                include_local_paths: true,
                checked_at: None,
            },
            checked_at,
        )
        .expect("policy explain with paths");
        assert_eq!(report_with_paths.docs_mirror.repo_root, Some(repo_root));
        assert!(
            report_with_paths
                .required_tools
                .iter()
                .any(|tool| tool.available && tool.path.is_some())
        );
    }

    #[test]
    fn policy_explain_docs_passed_is_scoped_to_profile_blocks() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path();
        fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("cargo toml");
        fs::create_dir_all(root.join("docs/reference")).expect("reference docs dir");
        fs::create_dir_all(root.join("docs/runbooks")).expect("runbook docs dir");

        let stale_smoke_block = format!(
            "\n# {}\n{}\n# {}\n",
            policy_doc_marker(POLICY_DOCS_SMOKE_MARKER, "start"),
            "cargo run -q -p codex-dev -- --json policy manifest --profile stale",
            policy_doc_marker(POLICY_DOCS_SMOKE_MARKER, "end")
        );
        for path in ["AGENTS.md", "README.md", "docs/reference/codex-dev-cli.md"] {
            fs::write(root.join(path), &stale_smoke_block).expect("stale smoke docs");
        }

        let all_profile_commands =
            policy_doc_block_expected_commands(PolicyDocBlockKind::AllProfiles).join("\n");
        fs::write(
            root.join("docs/runbooks/validation.md"),
            format!(
                "\n# {}\n{}\n# {}\n\n# {}\n{}\n# {}\n",
                policy_doc_marker(POLICY_DOCS_SMOKE_MARKER, "start"),
                "cargo run -q -p codex-dev -- --json policy manifest --profile stale",
                policy_doc_marker(POLICY_DOCS_SMOKE_MARKER, "end"),
                policy_doc_marker(POLICY_DOCS_ALL_MARKER, "start"),
                all_profile_commands,
                policy_doc_marker(POLICY_DOCS_ALL_MARKER, "end")
            ),
        )
        .expect("validation docs");

        let docs_check = policy_docs_check(Some(root)).expect("docs check");
        assert!(!docs_check.passed);

        let report = policy_explain(
            PolicyExplainArgs {
                profile: PolicyProfile::CodexDevTui,
                repo_root: Some(root.to_path_buf()),
                include_local_paths: false,
                checked_at: None,
            },
            "2026-05-09T05:00:00Z".parse().unwrap(),
        )
        .expect("profile-scoped policy explain");
        assert_eq!(report.docs_mirror.status, "current");
        assert!(report.docs_mirror.passed);
        assert!(
            report
                .docs_mirror
                .blocks
                .iter()
                .all(|block| block.profiles.contains(&PolicyProfile::CodexDevTui))
        );
        assert_eq!(report.docs_mirror.blocks.len(), 1);
    }

    #[test]
    fn policy_explain_gate_docs_status_matches_validation_anchor() {
        let docs_check = PolicyDocsCheckResult {
            schema: POLICY_DOCS_CHECK_SCHEMA,
            repo_root: PathBuf::from("."),
            passed: false,
            blocks: vec![
                PolicyDocsBlockResult {
                    path: "docs/runbooks/validation.md".to_string(),
                    marker: POLICY_DOCS_SMOKE_MARKER.to_string(),
                    profiles: vec![PolicyProfile::CodexDev, PolicyProfile::FullLocal],
                    expected_commands: Vec::new(),
                    actual_commands: Vec::new(),
                    passed: true,
                    error: None,
                },
                PolicyDocsBlockResult {
                    path: "docs/runbooks/validation.md".to_string(),
                    marker: POLICY_DOCS_ALL_MARKER.to_string(),
                    profiles: all_policy_profiles().to_vec(),
                    expected_commands: Vec::new(),
                    actual_commands: Vec::new(),
                    passed: false,
                    error: Some("stale all-profiles block".to_string()),
                },
            ],
        };
        let operating_layer_gate = policy_gate(
            "operating-layer",
            "operating layer",
            ["cargo", "--version"],
            "docs/runbooks/validation.md#codex-dev-operating-layer",
            ["cargo"],
            "Failure means test fixture failed.",
        );
        let full_local_gate = policy_gate(
            "full-local",
            "full local",
            ["cargo", "--version"],
            "docs/runbooks/validation.md#full-local-gate",
            ["cargo"],
            "Failure means test fixture failed.",
        );
        let validation_matrix_gate = policy_gate(
            "validation-matrix",
            "validation matrix",
            ["cargo", "--version"],
            "docs/runbooks/validation.md#validation-matrix-ownership",
            ["cargo"],
            "Failure means test fixture failed.",
        );

        assert_eq!(
            policy_explain_gate_docs_status(
                PolicyProfile::FullLocal,
                &operating_layer_gate,
                &docs_check
            ),
            "current"
        );
        assert_eq!(
            policy_explain_gate_docs_status(
                PolicyProfile::FullLocal,
                &full_local_gate,
                &docs_check
            ),
            "stale_or_missing"
        );
        assert_eq!(
            policy_explain_gate_docs_status(
                PolicyProfile::FullLocal,
                &validation_matrix_gate,
                &docs_check
            ),
            "not_mirrored"
        );
    }

    #[test]
    fn policy_explain_missing_prerequisites_groups_gate_ids() {
        let gates = vec![
            policy_gate(
                "first",
                "first",
                ["missing-tool-for-test", "--flag"],
                "test-source",
                ["missing-tool-for-test"],
                "Failure means first failed.",
            ),
            policy_gate(
                "second",
                "second",
                ["missing-tool-for-test", "subcommand"],
                "test-source",
                ["missing-tool-for-test"],
                "Failure means second failed.",
            ),
        ];
        let statuses = vec![PolicyExplainToolStatus {
            name: "missing-tool-for-test".to_string(),
            available: false,
            path: None,
        }];

        let missing = policy_explain_missing_prerequisites(&gates, &statuses);

        assert_eq!(
            missing,
            vec![PolicyExplainMissingPrerequisite {
                tool: "missing-tool-for-test".to_string(),
                gate_ids: vec!["first".to_string(), "second".to_string()],
                detail: "required command `missing-tool-for-test` was not found on PATH"
                    .to_string(),
            }]
        );
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
                "bun-platform-core-clippy",
                "bun-platform-clippy",
                "codex-dev-core-check",
                "codex-dev-check",
                "bun-platform-core-check",
                "bun-platform-check",
                "codex-dev-core-test",
                "codex-dev-test",
                "bun-platform-core-test",
                "bun-platform-test",
                "codex-dev-help",
                "codex-dev-completion-zsh",
                "codex-dev-manpage",
                "bun-platform-help",
                "bun-platform-completion-zsh",
                "codex-dev-policy-manifest",
                "codex-dev-policy-explain",
                "codex-dev-policy-docs-check",
                "codex-dev-skills-inventory-smoke",
                "codex-dev-bun-doctor-smoke",
                "codex-dev-bun-audit-smoke",
                "codex-dev-bun-fixes-plan-smoke",
                "codex-dev-bun-references-status-smoke",
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
                "codex-dev-tui-completion-zsh",
                "codex-dev-tui-manpage",
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
                "codex-research-completion-zsh",
                "codex-research-manpage",
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
                "codex-dev-bootstrap-status",
                "bootstrap-pack-validate",
                "bootstrap-pack-render-smoke",
                "codex-subagents-release-manifest",
                "codex-subagents-global-dry-run",
                "codex-subagents-validate-sources",
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
                "bun-platform-core-clippy",
                "bun-platform-clippy",
                "codex-dev-core-check",
                "codex-dev-check",
                "bun-platform-core-check",
                "bun-platform-check",
                "codex-dev-core-test",
                "codex-dev-test",
                "bun-platform-core-test",
                "bun-platform-test",
                "codex-dev-help",
                "codex-dev-completion-zsh",
                "codex-dev-manpage",
                "bun-platform-help",
                "bun-platform-completion-zsh",
                "codex-dev-policy-manifest",
                "codex-dev-policy-explain",
                "codex-dev-policy-docs-check",
                "codex-dev-skills-inventory-smoke",
                "codex-dev-bun-doctor-smoke",
                "codex-dev-bun-audit-smoke",
                "codex-dev-bun-fixes-plan-smoke",
                "codex-dev-bun-references-status-smoke",
                "codex-dev-pr-plan-smoke",
                "docs-links",
                "diff-check",
                "release-policy-explain",
                "codex-dev-tui-clippy",
                "codex-dev-tui-check",
                "codex-dev-tui-test",
                "codex-dev-tui-help",
                "codex-dev-tui-completion-zsh",
                "codex-dev-tui-manpage",
                "codex-research-clippy",
                "codex-research-check",
                "codex-research-test",
                "codex-research-doctor",
                "codex-research-eval",
                "codex-research-eval-list",
                "codex-research-eval-strict",
                "codex-research-plan-quick",
                "codex-research-completion-zsh",
                "codex-research-manpage",
                "gsap-audit-clippy",
                "gsap-audit-check",
                "gsap-audit-test",
                "gsap-audit-doctor",
                "gsap-audit-completion-zsh",
                "docs-no-todo",
                "bootstrap-pack-validate",
                "skills-quick-validate-all",
                "python-helpers-compile",
                "subagent-templates-validate",
                "subspawn-roles-validate",
                "subspawn-plan-research-smoke",
                "skill-subagent-eval",
                "cargo-metadata-locked",
                "cargo-tree-duplicates",
                "cargo-deny-policy",
                "cargo-package-codex-dev-core-list",
                "cargo-package-codex-dev-list",
                "cargo-package-bun-platform-core-list",
                "cargo-package-bun-platform-list",
                "cargo-package-codex-dev-tui-list",
                "cargo-package-codex-research-list",
            ],
        );
        assert_profile_ids(
            PolicyProfile::FullLocal,
            &[
                "cargo-fmt",
                "codex-dev-core-clippy",
                "codex-dev-clippy",
                "bun-platform-core-clippy",
                "bun-platform-clippy",
                "codex-dev-core-check",
                "codex-dev-check",
                "bun-platform-core-check",
                "bun-platform-check",
                "codex-dev-core-test",
                "codex-dev-test",
                "bun-platform-core-test",
                "bun-platform-test",
                "codex-dev-help",
                "codex-dev-completion-zsh",
                "codex-dev-manpage",
                "bun-platform-help",
                "bun-platform-completion-zsh",
                "codex-dev-policy-manifest",
                "codex-dev-policy-explain",
                "codex-dev-policy-docs-check",
                "codex-dev-skills-inventory-smoke",
                "codex-dev-bun-doctor-smoke",
                "codex-dev-bun-audit-smoke",
                "codex-dev-bun-fixes-plan-smoke",
                "codex-dev-bun-references-status-smoke",
                "codex-dev-pr-plan-smoke",
                "docs-links",
                "diff-check",
                "full-local-policy-explain",
                "codex-dev-tui-clippy",
                "codex-dev-tui-check",
                "codex-dev-tui-test",
                "codex-dev-tui-help",
                "codex-dev-tui-completion-zsh",
                "codex-dev-tui-manpage",
                "codex-research-clippy",
                "codex-research-check",
                "codex-research-test",
                "codex-research-doctor",
                "codex-research-eval",
                "codex-research-eval-list",
                "codex-research-eval-strict",
                "codex-research-plan-quick",
                "codex-research-completion-zsh",
                "codex-research-manpage",
                "gsap-audit-clippy",
                "gsap-audit-check",
                "gsap-audit-test",
                "gsap-audit-doctor",
                "gsap-audit-completion-zsh",
                "cargo-install-codex-research-smoke",
                "cargo-install-codex-dev-smoke",
                "cargo-install-bun-platform-smoke",
                "cargo-install-codex-dev-tui-smoke",
                "cargo-install-gsap-audit-smoke",
                "codex-dev-bootstrap-status",
                "bootstrap-pack-validate",
                "bootstrap-pack-render-smoke",
                "codex-subagents-release-manifest",
                "codex-subagents-global-dry-run",
                "codex-subagents-validate-sources",
                "bootstrap-local-overlays-ignored",
                "skills-quick-validate-all",
                "python-helpers-compile",
                "subagent-templates-validate",
                "subspawn-roles-validate",
                "subspawn-plan-research-smoke",
                "skill-subagent-eval",
                "docs-no-todo",
                "cargo-metadata-locked",
                "cargo-tree-duplicates",
                "cargo-deny-policy",
                "cargo-package-codex-dev-core-list",
                "cargo-package-codex-dev-list",
                "cargo-package-bun-platform-core-list",
                "cargo-package-bun-platform-list",
                "cargo-package-codex-dev-tui-list",
                "cargo-package-codex-research-list",
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
