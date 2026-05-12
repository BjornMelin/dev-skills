use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write as _};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const CAPSULE_SCHEMA: &str = "codex-dev.task-capsule.v1";
pub const EVIDENCE_SCHEMA: &str = "codex-dev.evidence.v1";
pub const VERIFICATION_SCHEMA: &str = "codex-dev.verification.v1";
pub const SUBAGENTS_SCHEMA: &str = "codex-dev.subagents.v1";
pub const PR_SCHEMA: &str = "codex-dev.pr.v1";
pub const PR_SOURCE_PARSER_VERSION: &str = "codex-dev.pr-source-parser.v1";
pub const PR_CONTROL_PLAN_SCHEMA: &str = "codex-dev.pr-control-plan.v1";
pub const PR_AGENT_STATE_SCHEMA: &str = "codex-dev.pr-agent-state.v1";
pub const PR_AGENT_HOSTED_ACTION_SCHEMA: &str = "codex-dev.pr-agent-hosted-action.v1";
pub const PR_AGENT_READINESS_SCHEMA: &str = "codex-dev.pr-agent-readiness.v1";
pub const OUTPUT_SCHEMA: &str = "codex-dev.output.v1";
pub const POLICY_GATES_SCHEMA: &str = "codex-dev.policy-gates.v1";
pub const TASK_INDEX_SCHEMA: &str = "task_index.v1";

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleStatus {
    Active,
    Blocked,
    ReadyForPr,
    InReview,
    Merged,
    Closed,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyProfile {
    CodexDev,
    CodexDevTui,
    CodexResearch,
    Skills,
    BootstrapInstall,
    Docs,
    Release,
    FullLocal,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub schema: String,
    pub kind: EvidenceKind,
    pub at: DateTime<Utc>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub residual_risk: Option<String>,
    pub artifacts: Vec<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Command,
    Subagent,
    Review,
    Ci,
    Decision,
    Research,
    Manual,
    Output,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceSummary {
    pub total: u64,
    pub by_kind: Vec<EvidenceKindSummary>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceKindSummary {
    pub kind: EvidenceKind,
    pub count: u64,
    pub latest_at: DateTime<Utc>,
    pub latest_summary: String,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subagents {
    pub schema: String,
    pub batches: Vec<SubagentBatch>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentBatch {
    pub id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rendezvous_required: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registry_issues: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub duplicate_roles_ignored: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prompts: Vec<SubagentPromptRecord>,
    pub agents: Vec<SubagentRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synthesis: Option<SubagentSynthesisRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentPromptRecord {
    pub role: String,
    pub prompt_id: String,
    pub prompt_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentRecord {
    pub role: String,
    pub task: String,
    pub status: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub human_verified: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentSynthesisRecord {
    pub status: String,
    pub summary: String,
    pub human_verified: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrEvidence {
    pub schema: String,
    pub repository: Option<String>,
    pub number: Option<u64>,
    pub url: Option<String>,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_draft: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mergeable: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_state_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_decision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_ref_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_ref_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_ref_oid: Option<String>,
    pub checks: Vec<CheckRecord>,
    pub review_threads: ReviewThreadSummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<PrEvidenceSource>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrEvidenceSource {
    pub kind: String,
    pub parser_version: String,
    pub retrieved_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckRecord {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub url: Option<String>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewThreadSummary {
    pub unresolved: u64,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub resolved: u64,
    #[serde(default)]
    pub outdated: u64,
    #[serde(default)]
    pub authoritative: bool,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PrRecordSourceKind {
    #[default]
    Normalized,
    GhPrView,
    GhPrChecks,
    GhReviews,
    GhReviewThreads,
    GhReviewComments,
    ReviewPackRemaining,
}

impl PrRecordSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normalized => "normalized",
            Self::GhPrView => "gh-pr-view",
            Self::GhPrChecks => "gh-pr-checks",
            Self::GhReviews => "gh-reviews",
            Self::GhReviewThreads => "gh-review-threads",
            Self::GhReviewComments => "gh-review-comments",
            Self::ReviewPackRemaining => "review-pack-remaining",
        }
    }
}

impl fmt::Display for PrRecordSourceKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for PrRecordSourceKind {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "normalized" => Ok(Self::Normalized),
            "gh-pr-view" => Ok(Self::GhPrView),
            "gh-pr-checks" => Ok(Self::GhPrChecks),
            "gh-reviews" => Ok(Self::GhReviews),
            "gh-review-threads" => Ok(Self::GhReviewThreads),
            "gh-review-comments" => Ok(Self::GhReviewComments),
            "review-pack-remaining" => Ok(Self::ReviewPackRemaining),
            _ => Err(format!(
                "unsupported PR evidence source kind '{value}' (expected one of: normalized, gh-pr-view, gh-pr-checks, gh-reviews, gh-review-threads, gh-review-comments, review-pack-remaining)"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrRecordArgs {
    pub capsule: PathBuf,
    pub source: PathBuf,
    pub source_kind: PrRecordSourceKind,
    pub repository: Option<String>,
    pub number: Option<u64>,
    pub retrieved_at: Option<DateTime<Utc>>,
    pub source_command: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendEvidenceArgs {
    pub capsule: PathBuf,
    pub record: EvidenceRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSubagentPlanArgs {
    pub capsule: PathBuf,
    pub batch_id: String,
    pub source: PathBuf,
    pub command: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubagentOutcomeStatus {
    Planned,
    Running,
    Completed,
    Failed,
    TimedOut,
    Closed,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubagentDisposition {
    Accepted,
    Rejected,
    Mixed,
    Informational,
    Pending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubagentSynthesisStatus {
    Completed,
    Partial,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSubagentOutcomeArgs {
    pub capsule: PathBuf,
    pub batch_id: String,
    pub role: String,
    pub status: SubagentOutcomeStatus,
    pub summary: String,
    pub disposition: SubagentDisposition,
    pub human_verified: bool,
    pub source_ids: Vec<String>,
    pub artifacts: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSubagentSynthesisArgs {
    pub capsule: PathBuf,
    pub batch_id: String,
    pub status: SubagentSynthesisStatus,
    pub summary: String,
    pub human_verified: bool,
    pub source_ids: Vec<String>,
    pub artifacts: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppendEvidenceResult {
    pub capsule: PathBuf,
    pub evidence_path: PathBuf,
    pub record: EvidenceRecord,
    pub evidence: EvidenceSummary,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordSubagentPlanResult {
    pub capsule: PathBuf,
    pub subagents_path: PathBuf,
    pub evidence_path: PathBuf,
    pub batch: SubagentBatch,
    pub evidence: EvidenceSummary,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordSubagentOutcomeResult {
    pub capsule: PathBuf,
    pub subagents_path: PathBuf,
    pub evidence_path: PathBuf,
    pub batch: SubagentBatch,
    pub agent: SubagentRecord,
    pub evidence: EvidenceSummary,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordSubagentSynthesisResult {
    pub capsule: PathBuf,
    pub subagents_path: PathBuf,
    pub evidence_path: PathBuf,
    pub batch: SubagentBatch,
    pub synthesis: SubagentSynthesisRecord,
    pub evidence: EvidenceSummary,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrAgentStateReport {
    pub schema: String,
    pub repository: String,
    pub number: u64,
    pub checked_at: DateTime<Utc>,
    pub dry_run: bool,
    pub pr: PrEvidence,
    pub sources: Vec<PrAgentSourceRecord>,
    pub diagnostics: Vec<PrAgentDiagnostic>,
    pub actions: Vec<PrAgentAction>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrAgentHostedActionReport {
    pub schema: String,
    pub repository: String,
    pub number: u64,
    pub plan_id: String,
    pub plan_hash: String,
    pub generated_at: DateTime<Utc>,
    pub dry_run: bool,
    pub apply_requested: bool,
    pub action_dir: String,
    pub before_state_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_state_path: Option<String>,
    pub action: PrAgentHostedActionSpec,
    pub diagnostics: Vec<PrAgentDiagnostic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<PrAgentHostedActionExecution>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrAgentHostedActionSpec {
    pub id: String,
    pub kind: String,
    pub summary: String,
    pub reason: String,
    pub target: String,
    pub idempotency_key: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub duplicate_check_command: Vec<String>,
    #[serde(default)]
    pub state_check_command: Vec<String>,
    pub requires_apply: bool,
    pub network: bool,
    pub secrets: bool,
    pub permission_notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrAgentHostedActionExecution {
    pub status: PrAgentHostedActionStatus,
    pub applied_at: DateTime<Utc>,
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duplicate_of: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrAgentHostedActionStatus {
    Applied,
    SkippedDuplicate,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrAgentReadinessReport {
    pub schema: String,
    pub repository: String,
    pub number: u64,
    pub generated_at: DateTime<Utc>,
    pub apply_requested: bool,
    pub rerun_failed_requested: bool,
    pub merge_requested: bool,
    pub ready: bool,
    pub final_status: PrAgentReadinessStatus,
    pub attempts: Vec<PrAgentReadinessAttempt>,
    pub actions: Vec<PrAgentReadinessAction>,
    pub markdown_path: String,
    pub report_path: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrAgentReadinessAttempt {
    pub attempt: u64,
    pub checked_at: DateTime<Utc>,
    pub status: PrAgentReadinessStatus,
    pub pr: PrEvidence,
    pub blockers: Vec<String>,
    pub wait_reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub failing_checks: Vec<PrAgentReadinessCheck>,
    pub pending_checks: Vec<PrAgentReadinessCheck>,
    pub active_review_comments: u64,
    pub outdated_review_comments: u64,
    pub diagnostics: Vec<PrAgentDiagnostic>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrAgentReadinessStatus {
    Ready,
    Waiting,
    Blocked,
    Merged,
    Stopped,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrAgentReadinessCheck {
    pub name: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<u64>,
    pub diagnostic_command: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrAgentReadinessAction {
    pub id: String,
    pub kind: String,
    pub status: PrAgentReadinessActionStatus,
    pub reason: String,
    pub command: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrAgentReadinessActionStatus {
    Planned,
    Applied,
    Skipped,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrAgentSourceRecord {
    pub id: String,
    pub kind: String,
    pub command: String,
    pub path: String,
    pub retrieved_at: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub status: PrAgentSourceStatus,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrAgentSourceStatus {
    Captured,
    Failed,
    Skipped,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrAgentDiagnostic {
    pub source: String,
    pub severity: PrAgentSeverity,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PrAgentSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrAgentAction {
    pub id: String,
    pub priority: PrAgentActionPriority,
    pub summary: String,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrAgentActionPriority {
    Blocked,
    Required,
    Wait,
    Ready,
    Info,
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
    is_draft: Option<bool>,
    #[serde(default)]
    mergeable: Option<String>,
    #[serde(default)]
    merge_state_status: Option<String>,
    #[serde(default)]
    review_decision: Option<String>,
    #[serde(default)]
    head_sha: Option<String>,
    #[serde(default)]
    head_ref_name: Option<String>,
    #[serde(default)]
    base_ref_name: Option<String>,
    #[serde(default)]
    base_ref_oid: Option<String>,
    #[serde(default)]
    checks: Vec<CheckSnapshotInput>,
    review_threads: ReviewThreadSnapshotInput,
    #[serde(default)]
    sources: Vec<PrEvidenceSource>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubspawnPlanInput {
    task: String,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    scope_items: Vec<String>,
    #[serde(default)]
    wait_policy: Option<String>,
    #[serde(default)]
    rendezvous_required: Option<bool>,
    #[serde(default)]
    roles: Vec<SubspawnPlanRole>,
    #[serde(default)]
    prompts: Vec<SubspawnPlanPrompt>,
    #[serde(default)]
    registry_issues: Vec<String>,
    #[serde(default)]
    duplicate_roles_ignored: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    synthesis_checklist: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubspawnPlanRole {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    reasoning: Option<String>,
    #[serde(default)]
    return_headings: Vec<String>,
    #[serde(default)]
    sandbox: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubspawnPlanPrompt {
    role: String,
    prompt: String,
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
    total: u64,
    #[serde(default)]
    resolved: u64,
    #[serde(default)]
    outdated: u64,
    #[serde(default)]
    authoritative: Option<bool>,
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
    #[serde(default = "default_policy_working_directory")]
    pub working_directory: String,
    #[serde(default = "default_policy_required_tools")]
    pub required_tools: Vec<String>,
    pub required: bool,
    pub network: bool,
    pub secrets: bool,
    #[serde(default = "default_policy_failure_interpretation")]
    pub failure_interpretation: String,
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

#[derive(Debug)]
pub struct InitArgs {
    pub title: String,
    pub objective: String,
    pub branch: String,
    pub base_branch: String,
    pub issues: Vec<u64>,
    pub pull_requests: Vec<u64>,
    pub root: PathBuf,
    pub slug: Option<String>,
    pub id: Option<String>,
    pub status: CapsuleStatus,
    pub created_at: DateTime<Utc>,
    pub policy_manifest: PolicyManifest,
    pub force: bool,
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
    pub evidence: EvidenceSummary,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenderResult {
    pub path: PathBuf,
    pub markdown: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskIndexReport {
    pub schema: String,
    pub root: PathBuf,
    pub total: u64,
    pub valid: u64,
    pub invalid: u64,
    pub diagnostics: Vec<String>,
    pub tasks: Vec<TaskIndexEntry>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskIndexEntry {
    pub path: PathBuf,
    pub valid: bool,
    pub errors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capsule: Option<StatusResult>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskShowReport {
    pub schema: String,
    pub root: PathBuf,
    pub task: TaskIndexEntry,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskExportReport {
    pub schema: String,
    pub root: PathBuf,
    pub task: TaskIndexEntry,
    pub capsule: Capsule,
    pub evidence: Vec<EvidenceRecord>,
    pub verification: Verification,
    pub subagents: Subagents,
    pub pr: PrEvidence,
    pub policy: PolicyManifest,
    pub markdown: BTreeMap<String, String>,
}

pub fn record_pr_snapshot(args: PrRecordArgs, checked_at: DateTime<Utc>) -> Result<PrRecordResult> {
    validate_capsule_for_pr_record(&args.capsule)?;

    let mut pr = normalize_pr_record_source(&args, checked_at)?;
    if args.source_kind != PrRecordSourceKind::Normalized {
        let existing: PrEvidence = read_json(&args.capsule.join("pr.json"))?;
        pr = merge_provider_pr_evidence(existing, pr, args.source_kind)?;
    }
    write_json(args.capsule.join("pr.json"), &pr)?;

    let mut capsule: Capsule = read_json(&args.capsule.join("capsule.json"))?;
    if let Some(number) = pr.number
        && !capsule.pull_requests.contains(&number)
    {
        capsule.pull_requests.push(number);
    }
    capsule.updated_at = std::cmp::max(capsule.updated_at, checked_at);
    write_json(args.capsule.join("capsule.json"), &capsule)?;

    let evidence_command = args.command;
    let evidence_exit_code = evidence_command.as_ref().map(|_| 0);
    append_jsonl(
        args.capsule.join("evidence.jsonl"),
        &EvidenceRecord {
            schema: EVIDENCE_SCHEMA.to_string(),
            kind: EvidenceKind::Review,
            at: checked_at,
            summary: format!(
                "PR snapshot recorded for {}; {}; {} check(s)",
                render_pr_label(&pr),
                render_review_thread_summary(&pr.review_threads),
                pr.checks.len()
            ),
            command: evidence_command,
            exit_code: evidence_exit_code,
            source_ids: Vec::new(),
            actor: None,
            tool: None,
            confidence: None,
            residual_risk: None,
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

pub fn append_evidence(args: AppendEvidenceArgs) -> Result<AppendEvidenceResult> {
    ensure_regular_contract_files(&args.capsule)?;
    let validation = validate_capsule(&args.capsule)?;
    if !validation.valid {
        bail!(
            "invalid capsule at {}: {}",
            args.capsule.display(),
            validation.errors.join("; ")
        );
    }

    let errors = validate_evidence_record(&args.record);
    if !errors.is_empty() {
        bail!("invalid evidence record: {}", errors.join("; "));
    }

    append_jsonl(args.capsule.join("evidence.jsonl"), &args.record)?;

    let mut capsule: Capsule = read_json(&args.capsule.join("capsule.json"))?;
    capsule.updated_at = std::cmp::max(capsule.updated_at, args.record.at);
    write_json(args.capsule.join("capsule.json"), &capsule)?;

    let evidence = evidence_summary(&args.capsule)?;

    Ok(AppendEvidenceResult {
        evidence_path: args.capsule.join("evidence.jsonl"),
        capsule: args.capsule,
        record: args.record,
        evidence,
    })
}

pub fn record_subagent_plan(args: RecordSubagentPlanArgs) -> Result<RecordSubagentPlanResult> {
    validate_capsule_for_subagent_record(&args.capsule)?;
    validate_stable_id("batch_id", &args.batch_id)?;
    let plan: SubspawnPlanInput = read_json(&args.source)
        .with_context(|| format!("failed to read subspawn plan {}", args.source.display()))?;
    let batch = subspawn_plan_to_batch(plan, &args.batch_id, args.recorded_at)?;

    let mut subagents: Subagents = read_json(&args.capsule.join("subagents.json"))?;
    if subagents
        .batches
        .iter()
        .any(|batch| batch.id == args.batch_id)
    {
        bail!("subagent batch already exists: {}", args.batch_id);
    }
    let evidence_record = EvidenceRecord {
        schema: EVIDENCE_SCHEMA.to_string(),
        kind: EvidenceKind::Subagent,
        at: args.recorded_at,
        summary: format!(
            "Recorded subspawn plan {} with {} role(s)",
            batch.id,
            batch.agents.len()
        ),
        command: args.command,
        exit_code: None,
        source_ids: vec![format!("subagents:{}", batch.id)],
        actor: None,
        tool: Some("subspawn".to_string()),
        confidence: None,
        residual_risk: None,
        artifacts: vec![
            "subagents.json".to_string(),
            args.source.display().to_string(),
        ],
    };
    validate_subagent_evidence_record(&evidence_record)?;
    subagents.batches.push(batch.clone());
    ensure_valid_subagents_value(&subagents)?;
    write_json(args.capsule.join("subagents.json"), &subagents)?;
    append_subagent_evidence(&args.capsule, evidence_record)?;
    touch_capsule(&args.capsule, args.recorded_at)?;

    let evidence = evidence_summary(&args.capsule)?;
    Ok(RecordSubagentPlanResult {
        capsule: args.capsule.clone(),
        subagents_path: args.capsule.join("subagents.json"),
        evidence_path: args.capsule.join("evidence.jsonl"),
        batch,
        evidence,
    })
}

pub fn record_subagent_outcome(
    args: RecordSubagentOutcomeArgs,
) -> Result<RecordSubagentOutcomeResult> {
    validate_capsule_for_subagent_record(&args.capsule)?;
    validate_stable_id("batch_id", &args.batch_id)?;
    validate_role_name(&args.role)?;
    validate_human_verified(args.human_verified)?;
    validate_outcome_disposition(args.status, args.disposition)?;
    let mut errors = Vec::new();
    validate_non_empty_text("summary", &args.summary, &mut errors);
    validate_required_repeated_text("source_ids", &args.source_ids, &mut errors);
    validate_required_repeated_text("artifacts", &args.artifacts, &mut errors);
    if !errors.is_empty() {
        bail!("invalid subagent outcome: {}", errors.join("; "));
    }

    let mut subagents: Subagents = read_json(&args.capsule.join("subagents.json"))?;
    let batch = subagents
        .batches
        .iter_mut()
        .find(|batch| batch.id == args.batch_id)
        .with_context(|| format!("unknown subagent batch: {}", args.batch_id))?;
    let agent = batch
        .agents
        .iter_mut()
        .find(|agent| agent.role == args.role)
        .with_context(|| {
            format!(
                "role {} is not recorded in subagent batch {}",
                args.role, args.batch_id
            )
        })?;
    agent.status = args.status.to_string();
    agent.summary = args.summary.clone();
    agent.disposition = Some(args.disposition.to_string());
    agent.human_verified = args.human_verified;
    agent.source_ids = args.source_ids.clone();
    agent.artifacts = args.artifacts.clone();
    agent.updated_at = Some(args.recorded_at);
    let agent_result = agent.clone();
    batch.updated_at = Some(args.recorded_at);
    refresh_batch_status(batch);
    let batch_result = batch.clone();
    let evidence_record = EvidenceRecord {
        schema: EVIDENCE_SCHEMA.to_string(),
        kind: EvidenceKind::Subagent,
        at: args.recorded_at,
        summary: format!("Subagent {} {}: {}", args.role, args.status, args.summary),
        command: None,
        exit_code: None,
        source_ids: subagent_source_ids(&args.batch_id, &args.role, &args.source_ids),
        actor: Some(args.role),
        tool: Some("subspawn".to_string()),
        confidence: None,
        residual_risk: None,
        artifacts: subagent_artifacts(&args.artifacts),
    };
    validate_subagent_evidence_record(&evidence_record)?;
    ensure_valid_subagents_value(&subagents)?;
    write_json(args.capsule.join("subagents.json"), &subagents)?;
    append_subagent_evidence(&args.capsule, evidence_record)?;
    touch_capsule(&args.capsule, args.recorded_at)?;

    let evidence = evidence_summary(&args.capsule)?;
    Ok(RecordSubagentOutcomeResult {
        capsule: args.capsule.clone(),
        subagents_path: args.capsule.join("subagents.json"),
        evidence_path: args.capsule.join("evidence.jsonl"),
        batch: batch_result,
        agent: agent_result,
        evidence,
    })
}

pub fn record_subagent_synthesis(
    args: RecordSubagentSynthesisArgs,
) -> Result<RecordSubagentSynthesisResult> {
    validate_capsule_for_subagent_record(&args.capsule)?;
    validate_stable_id("batch_id", &args.batch_id)?;
    validate_human_verified(args.human_verified)?;
    let mut errors = Vec::new();
    validate_non_empty_text("summary", &args.summary, &mut errors);
    validate_required_repeated_text("source_ids", &args.source_ids, &mut errors);
    validate_required_repeated_text("artifacts", &args.artifacts, &mut errors);
    if !errors.is_empty() {
        bail!("invalid subagent synthesis: {}", errors.join("; "));
    }

    let mut subagents: Subagents = read_json(&args.capsule.join("subagents.json"))?;
    let batch = subagents
        .batches
        .iter_mut()
        .find(|batch| batch.id == args.batch_id)
        .with_context(|| format!("unknown subagent batch: {}", args.batch_id))?;
    if args.status == SubagentSynthesisStatus::Completed {
        ensure_completed_synthesis_ready(batch)?;
    }
    let synthesis = SubagentSynthesisRecord {
        status: args.status.to_string(),
        summary: args.summary.clone(),
        human_verified: args.human_verified,
        source_ids: args.source_ids.clone(),
        artifacts: args.artifacts.clone(),
        updated_at: args.recorded_at,
    };
    batch.synthesis = Some(synthesis.clone());
    batch.updated_at = Some(args.recorded_at);
    apply_synthesis_status(batch, args.status);
    let batch_result = batch.clone();
    let evidence_record = EvidenceRecord {
        schema: EVIDENCE_SCHEMA.to_string(),
        kind: EvidenceKind::Subagent,
        at: args.recorded_at,
        summary: format!("Subagent synthesis {}: {}", args.status, args.summary),
        command: None,
        exit_code: None,
        source_ids: synthesis_source_ids(&args.batch_id, &args.source_ids),
        actor: None,
        tool: Some("subspawn".to_string()),
        confidence: None,
        residual_risk: None,
        artifacts: subagent_artifacts(&args.artifacts),
    };
    validate_subagent_evidence_record(&evidence_record)?;
    ensure_valid_subagents_value(&subagents)?;
    write_json(args.capsule.join("subagents.json"), &subagents)?;
    append_subagent_evidence(&args.capsule, evidence_record)?;
    touch_capsule(&args.capsule, args.recorded_at)?;

    let evidence = evidence_summary(&args.capsule)?;
    Ok(RecordSubagentSynthesisResult {
        capsule: args.capsule.clone(),
        subagents_path: args.capsule.join("subagents.json"),
        evidence_path: args.capsule.join("evidence.jsonl"),
        batch: batch_result,
        synthesis,
        evidence,
    })
}

fn ensure_regular_contract_file(capsule_path: &Path, file: &str) -> Result<()> {
    let path = capsule_path.join(file);
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bail!("missing required file: {file}");
        }
        Err(error) => {
            return Err(error).with_context(|| format!("failed to inspect {}", path.display()));
        }
    };
    if metadata.file_type().is_symlink() {
        bail!(
            "refusing to write through symlinked capsule contract file: {}",
            path.display()
        );
    }
    if !metadata.is_file() {
        bail!("capsule contract path is not a file: {}", path.display());
    }
    Ok(())
}

pub fn ensure_regular_contract_files(capsule_path: &Path) -> Result<()> {
    for file in REQUIRED_FILES
        .iter()
        .copied()
        .filter(|file| file.ends_with(".json") || file.ends_with(".jsonl"))
    {
        ensure_regular_contract_file(capsule_path, file)?;
    }
    Ok(())
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
    ensure_regular_contract_files(capsule_path)?;
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
    let created_at = args.created_at;
    let slug = args.slug.unwrap_or_else(|| slugify(&args.title));
    let id = args
        .id
        .unwrap_or_else(|| format!("{}-{}", created_at.format("%Y%m%d-%H%M%S"), slug));
    validate_capsule_id(&id)?;
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
        objective: args.objective,
        branch: args.branch,
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
        source_ids: Vec::new(),
        actor: None,
        tool: None,
        confidence: None,
        residual_risk: None,
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
            is_draft: None,
            mergeable: None,
            merge_state_status: None,
            review_decision: None,
            head_sha: None,
            head_ref_name: None,
            base_ref_name: None,
            base_ref_oid: None,
            checks: Vec::new(),
            review_threads: ReviewThreadSummary {
                unresolved: 0,
                total: 0,
                resolved: 0,
                outdated: 0,
                authoritative: false,
                last_checked_at: created_at,
            },
            sources: Vec::new(),
        },
    )?;
    write_json(path.join("policy.json"), &args.policy_manifest)?;

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
    let invalid_contract_files = validate_required_file_paths(path, &mut errors);

    if can_validate_contract_file("capsule.json", &missing_files, &invalid_contract_files) {
        match read_json::<Capsule>(&path.join("capsule.json")) {
            Ok(capsule) => {
                if capsule.schema != CAPSULE_SCHEMA {
                    errors.push(format!("capsule.json schema must be {CAPSULE_SCHEMA}"));
                }
            }
            Err(error) => errors.push(format!("invalid capsule.json: {error:#}")),
        }
    }

    if can_validate_contract_file("evidence.jsonl", &missing_files, &invalid_contract_files) {
        match validate_evidence(&path.join("evidence.jsonl")) {
            Ok(file_errors) => errors.extend(file_errors),
            Err(error) => errors.push(format!("invalid evidence.jsonl: {error:#}")),
        }
    }

    if can_validate_contract_file("verification.json", &missing_files, &invalid_contract_files) {
        validate_schema_file::<Verification, _>(
            &path.join("verification.json"),
            VERIFICATION_SCHEMA,
            |value| &value.schema,
            &mut errors,
        );
    }
    if can_validate_contract_file("subagents.json", &missing_files, &invalid_contract_files) {
        match validate_subagents(&path.join("subagents.json")) {
            Ok(file_errors) => errors.extend(file_errors),
            Err(error) => errors.push(format!("invalid subagents.json: {error:#}")),
        }
    }
    if can_validate_contract_file("pr.json", &missing_files, &invalid_contract_files) {
        validate_schema_file::<PrEvidence, _>(
            &path.join("pr.json"),
            PR_SCHEMA,
            |value| &value.schema,
            &mut errors,
        );
    }
    if can_validate_contract_file("policy.json", &missing_files, &invalid_contract_files) {
        match validate_policy_manifest(&path.join("policy.json")) {
            Ok(file_errors) => errors.extend(file_errors),
            Err(error) => errors.push(format!("invalid policy.json: {error:#}")),
        }
    }

    Ok(ValidationResult {
        path: path.to_path_buf(),
        valid: errors.is_empty(),
        errors,
    })
}

fn can_validate_contract_file(
    file: &str,
    missing_files: &[&str],
    invalid_contract_files: &BTreeSet<String>,
) -> bool {
    !missing_files.contains(&file) && !invalid_contract_files.contains(file)
}

fn validate_required_file_paths(path: &Path, errors: &mut Vec<String>) -> BTreeSet<String> {
    let mut invalid = BTreeSet::new();
    for file in REQUIRED_FILES.iter().copied() {
        let file_path = path.join(file);
        let metadata = match file_path.symlink_metadata() {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => {
                errors.push(format!(
                    "failed to inspect {}: {error}",
                    file_path.display()
                ));
                invalid.insert(file.to_string());
                continue;
            }
        };
        if metadata.file_type().is_symlink() {
            errors.push(format!(
                "refusing to validate symlinked capsule contract file: {}",
                file_path.display()
            ));
            invalid.insert(file.to_string());
        } else if !metadata.is_file() {
            errors.push(format!(
                "capsule contract path is not a file: {}",
                file_path.display()
            ));
            invalid.insert(file.to_string());
        }
    }
    invalid
}

fn default_policy_working_directory() -> String {
    ".".to_string()
}

fn default_policy_required_tools() -> Vec<String> {
    vec!["legacy-unspecified".to_string()]
}

fn default_policy_failure_interpretation() -> String {
    "Legacy v1 policy manifest omitted failure interpretation.".to_string()
}

fn validate_policy_manifest(path: &Path) -> Result<Vec<String>> {
    let manifest = read_json::<PolicyManifest>(path)?;
    let mut errors = Vec::new();
    if manifest.schema != POLICY_GATES_SCHEMA {
        errors.push(format!("policy.json schema must be {POLICY_GATES_SCHEMA}"));
    }
    validate_policy_manifest_value(&manifest, &mut errors);
    Ok(errors)
}

fn validate_policy_manifest_value(manifest: &PolicyManifest, errors: &mut Vec<String>) {
    if manifest.gates.is_empty() {
        errors.push("policy gates must not be empty".to_string());
    }
    let mut seen = BTreeSet::new();
    for (index, gate) in manifest.gates.iter().enumerate() {
        let prefix = format!("policy.gates[{index}]");
        if let Err(error) = validate_stable_id(&format!("{prefix}.id"), &gate.id) {
            errors.push(error.to_string());
        }
        if !gate.id.is_empty() && !seen.insert(gate.id.as_str()) {
            errors.push(format!("{prefix}.id {} is duplicated", gate.id));
        }
        validate_non_empty_text(&format!("{prefix}.name"), &gate.name, errors);
        validate_required_repeated_text(&format!("{prefix}.command"), &gate.command, errors);
        validate_non_empty_text(&format!("{prefix}.source"), &gate.source, errors);
        validate_required_repeated_text(
            &format!("{prefix}.required_tools"),
            &gate.required_tools,
            errors,
        );
        validate_non_empty_text(
            &format!("{prefix}.failure_interpretation"),
            &gate.failure_interpretation,
            errors,
        );
        validate_policy_working_directory(
            &format!("{prefix}.working_directory"),
            &gate.working_directory,
            errors,
        );
    }
}

fn validate_policy_working_directory(field: &str, value: &str, errors: &mut Vec<String>) {
    validate_non_empty_text(field, value, errors);
    if value.trim().is_empty() {
        return;
    }
    let path = Path::new(value);
    if path.is_absolute() {
        errors.push(format!("{field} must be a repo-relative path"));
        return;
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        errors.push(format!(
            "{field} must stay within the repository and cannot contain '..'"
        ));
    }
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
    let evidence = evidence_summary(path)?;
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
        evidence,
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
    writeln!(markdown, "## Evidence")?;
    writeln!(markdown)?;
    writeln!(markdown, "- Total records: {}", status.evidence.total)?;
    for kind in &status.evidence.by_kind {
        writeln!(
            markdown,
            "- `{}`: {} record(s); latest `{}` - {}",
            kind.kind, kind.count, kind.latest_at, kind.latest_summary
        )?;
    }
    writeln!(markdown)?;
    writeln!(markdown, "## Objective")?;
    writeln!(markdown)?;
    writeln!(markdown, "{}", status.objective)?;

    Ok(RenderResult {
        path: path.to_path_buf(),
        markdown,
    })
}

pub fn task_index(root: &Path) -> Result<TaskIndexReport> {
    let mut diagnostics = Vec::new();
    let mut tasks = Vec::new();

    let metadata = match root.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            diagnostics.push(format!("task root does not exist: {}", root.display()));
            return Ok(TaskIndexReport {
                schema: TASK_INDEX_SCHEMA.to_string(),
                root: root.to_path_buf(),
                total: 0,
                valid: 0,
                invalid: 0,
                diagnostics,
                tasks,
            });
        }
        Err(error) => {
            diagnostics.push(format!(
                "failed to inspect task root {}: {error}",
                root.display()
            ));
            return Ok(TaskIndexReport {
                schema: TASK_INDEX_SCHEMA.to_string(),
                root: root.to_path_buf(),
                total: 0,
                valid: 0,
                invalid: 0,
                diagnostics,
                tasks,
            });
        }
    };

    if metadata.file_type().is_symlink() {
        diagnostics.push(format!(
            "refusing to scan symlinked task root: {}",
            root.display()
        ));
        return Ok(TaskIndexReport {
            schema: TASK_INDEX_SCHEMA.to_string(),
            root: root.to_path_buf(),
            total: 0,
            valid: 0,
            invalid: 0,
            diagnostics,
            tasks,
        });
    }

    if !metadata.is_dir() {
        diagnostics.push(format!("task root is not a directory: {}", root.display()));
        return Ok(TaskIndexReport {
            schema: TASK_INDEX_SCHEMA.to_string(),
            root: root.to_path_buf(),
            total: 0,
            valid: 0,
            invalid: 0,
            diagnostics,
            tasks,
        });
    }

    let mut paths = fs::read_dir(root)
        .with_context(|| format!("failed to read task root {}", root.display()))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .with_context(|| format!("failed to read task root entry in {}", root.display()))
        })
        .collect::<Result<Vec<_>>>()?;
    paths.sort();

    for path in paths {
        tasks.push(task_index_entry(&path));
    }

    let valid = tasks.iter().filter(|task| task.valid).count() as u64;
    let total = tasks.len() as u64;
    Ok(TaskIndexReport {
        schema: TASK_INDEX_SCHEMA.to_string(),
        root: root.to_path_buf(),
        total,
        valid,
        invalid: total.saturating_sub(valid),
        diagnostics,
        tasks,
    })
}

pub fn task_show(root: &Path, selector: &Path) -> Result<TaskShowReport> {
    let path = resolve_task_selector(root, selector)?;
    Ok(TaskShowReport {
        schema: TASK_INDEX_SCHEMA.to_string(),
        root: root.to_path_buf(),
        task: task_index_entry(&path),
    })
}

pub fn task_export(root: &Path, selector: &Path) -> Result<TaskExportReport> {
    let show = task_show(root, selector)?;
    if !show.task.valid {
        bail!(
            "invalid task capsule at {}: {}",
            show.task.path.display(),
            show.task.errors.join("; ")
        );
    }

    let task_path = show.task.path.clone();
    Ok(TaskExportReport {
        schema: TASK_INDEX_SCHEMA.to_string(),
        root: show.root,
        task: show.task,
        capsule: read_json(&task_path.join("capsule.json"))?,
        evidence: read_evidence_records(&task_path.join("evidence.jsonl"))?,
        verification: read_json(&task_path.join("verification.json"))?,
        subagents: read_json(&task_path.join("subagents.json"))?,
        pr: read_json(&task_path.join("pr.json"))?,
        policy: read_json(&task_path.join("policy.json"))?,
        markdown: read_markdown_exports(&task_path)?,
    })
}

fn task_index_entry(path: &Path) -> TaskIndexEntry {
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) => {
            return TaskIndexEntry {
                path: path.to_path_buf(),
                valid: false,
                errors: vec![format!("failed to inspect task entry: {error}")],
                capsule: None,
            };
        }
    };

    if metadata.file_type().is_symlink() {
        return TaskIndexEntry {
            path: path.to_path_buf(),
            valid: false,
            errors: vec![format!(
                "refusing to scan symlinked task entry: {}",
                path.display()
            )],
            capsule: None,
        };
    }

    if !metadata.is_dir() {
        return TaskIndexEntry {
            path: path.to_path_buf(),
            valid: false,
            errors: vec![format!("task entry is not a directory: {}", path.display())],
            capsule: None,
        };
    }

    let validation = match validate_capsule(path) {
        Ok(validation) => validation,
        Err(error) => {
            return TaskIndexEntry {
                path: path.to_path_buf(),
                valid: false,
                errors: vec![format!("{error:#}")],
                capsule: None,
            };
        }
    };

    if !validation.valid {
        return TaskIndexEntry {
            path: path.to_path_buf(),
            valid: false,
            errors: validation.errors,
            capsule: None,
        };
    }

    match capsule_status(path) {
        Ok(status) => TaskIndexEntry {
            path: path.to_path_buf(),
            valid: true,
            errors: Vec::new(),
            capsule: Some(status),
        },
        Err(error) => TaskIndexEntry {
            path: path.to_path_buf(),
            valid: false,
            errors: vec![format!("{error:#}")],
            capsule: None,
        },
    }
}

fn resolve_task_selector(root: &Path, selector: &Path) -> Result<PathBuf> {
    if selector.is_absolute() {
        return Ok(selector.to_path_buf());
    }
    if is_single_normal_component(selector) {
        validate_task_root_for_selector(root)?;
        Ok(root.join(selector))
    } else {
        Ok(selector.to_path_buf())
    }
}

fn is_single_normal_component(path: &Path) -> bool {
    let mut components = path.components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

fn validate_task_root_for_selector(root: &Path) -> Result<()> {
    let metadata = match root.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bail!("task root does not exist: {}", root.display());
        }
        Err(error) => {
            bail!("failed to inspect task root {}: {error}", root.display());
        }
    };
    if metadata.file_type().is_symlink() {
        bail!("refusing to scan symlinked task root: {}", root.display());
    }
    if !metadata.is_dir() {
        bail!("task root is not a directory: {}", root.display());
    }
    Ok(())
}

fn read_evidence_records(path: &Path) -> Result<Vec<EvidenceRecord>> {
    let mut records = Vec::new();
    for_each_evidence_record(path, |_, record| {
        records.push(record);
        Ok(())
    })?;
    Ok(records)
}

fn read_markdown_exports(path: &Path) -> Result<BTreeMap<String, String>> {
    let mut markdown = BTreeMap::new();
    for file in ["plan.md", "decisions.md", "output.md", "retrospective.md"] {
        let file_path = path.join(file);
        let metadata = file_path
            .symlink_metadata()
            .with_context(|| format!("failed to inspect {}", file_path.display()))?;
        if metadata.file_type().is_symlink() {
            bail!(
                "refusing to read symlinked capsule markdown file: {}",
                file_path.display()
            );
        }
        if !metadata.is_file() {
            bail!(
                "capsule markdown path is not a file: {}",
                file_path.display()
            );
        }
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("failed to open {}", file_path.display()))?;
        markdown.insert(file.to_string(), content);
    }
    Ok(markdown)
}

pub fn evidence_summary(capsule_path: &Path) -> Result<EvidenceSummary> {
    let mut by_kind: BTreeMap<EvidenceKind, EvidenceKindSummary> = BTreeMap::new();
    let mut total = 0;
    for_each_evidence_record(&capsule_path.join("evidence.jsonl"), |_, record| {
        total += 1;
        by_kind
            .entry(record.kind)
            .and_modify(|summary| {
                summary.count += 1;
                if record.at >= summary.latest_at {
                    summary.latest_at = record.at;
                    summary.latest_summary = record.summary.clone();
                }
            })
            .or_insert_with(|| EvidenceKindSummary {
                kind: record.kind,
                count: 1,
                latest_at: record.at,
                latest_summary: record.summary.clone(),
            });
        Ok(())
    })?;

    Ok(EvidenceSummary {
        total,
        by_kind: by_kind.into_values().collect(),
    })
}

fn normalize_pr_record_source(
    args: &PrRecordArgs,
    checked_at: DateTime<Utc>,
) -> Result<PrEvidence> {
    let retrieved_at = args.retrieved_at.unwrap_or(checked_at);
    let mut pr = match args.source_kind {
        PrRecordSourceKind::Normalized => {
            let snapshot: PrSnapshotInput = read_json(&args.source)?;
            snapshot.into_pr_evidence(checked_at)
        }
        PrRecordSourceKind::GhPrView => normalize_gh_pr_view(&args.source, checked_at)?,
        PrRecordSourceKind::GhPrChecks => normalize_gh_pr_checks(&args.source, checked_at)?,
        PrRecordSourceKind::GhReviews => normalize_gh_reviews(&args.source, checked_at)?,
        PrRecordSourceKind::GhReviewThreads => {
            normalize_gh_review_threads(&args.source, checked_at)?
        }
        PrRecordSourceKind::GhReviewComments => {
            normalize_gh_review_comments(&args.source, checked_at)?
        }
        PrRecordSourceKind::ReviewPackRemaining => {
            normalize_review_pack_remaining(&args.source, checked_at)?
        }
    };

    apply_pr_record_identity(
        &mut pr,
        args.source_kind,
        args.repository.as_deref(),
        args.number,
    )?;
    pr.sources.push(PrEvidenceSource {
        kind: args.source_kind.to_string(),
        parser_version: PR_SOURCE_PARSER_VERSION.to_string(),
        retrieved_at,
        command: args.source_command.clone(),
        path: Some(args.source.display().to_string()),
    });
    Ok(pr)
}

fn apply_pr_record_identity(
    pr: &mut PrEvidence,
    source_kind: PrRecordSourceKind,
    repository: Option<&str>,
    number: Option<u64>,
) -> Result<()> {
    if let Some(repository) = pr.url.as_deref().and_then(repository_from_pr_url) {
        merge_pr_string_field(&mut pr.repository, Some(repository), "repository")?;
    }
    if let Some(number) = pr.url.as_deref().and_then(number_from_pr_url) {
        merge_pr_number_field(&mut pr.number, Some(number), "number")?;
    }
    if let Some(repository) = repository {
        merge_pr_string_field(
            &mut pr.repository,
            Some(repository.to_string()),
            "repository",
        )?;
    }
    if let Some(number) = number {
        merge_pr_number_field(&mut pr.number, Some(number), "number")?;
    }
    if source_kind != PrRecordSourceKind::Normalized
        && (pr.repository.is_none() || pr.number.is_none())
    {
        bail!(
            "PR evidence source kind {source_kind} requires explicit PR identity; pass --repo and --number or provide a GitHub pull request URL"
        );
    }
    Ok(())
}

fn normalize_gh_pr_view(path: &Path, checked_at: DateTime<Utc>) -> Result<PrEvidence> {
    let value = read_json::<Value>(path)?;
    let number = optional_u64(&value, "number");
    let url = optional_string(&value, "url");
    let repository = url.as_deref().and_then(repository_from_pr_url);
    let is_draft = optional_bool(&value, "isDraft");
    let state = if is_draft.unwrap_or(false) {
        "draft".to_string()
    } else {
        optional_string(&value, "state")
            .unwrap_or_else(|| "unknown".to_string())
            .to_ascii_lowercase()
    };
    let mergeable = optional_string(&value, "mergeable").map(|value| value.to_ascii_lowercase());
    let merge_state_status =
        optional_string(&value, "mergeStateStatus").map(|value| value.to_ascii_lowercase());
    let review_decision =
        optional_string(&value, "reviewDecision").map(|value| value.to_ascii_lowercase());
    let head_sha = optional_string(&value, "headRefOid");
    let head_ref_name = optional_string(&value, "headRefName");
    let base_ref_name = optional_string(&value, "baseRefName");
    let base_ref_oid = optional_string(&value, "baseRefOid");
    let checks = value
        .get("statusCheckRollup")
        .map(|rollup| checks_from_status_rollup(rollup, checked_at))
        .transpose()?
        .unwrap_or_default();
    let review_threads = value
        .get("reviewThreads")
        .map(|threads| review_thread_summary_from_graphql(threads, checked_at))
        .transpose()?
        .unwrap_or_else(|| empty_review_threads(checked_at));

    Ok(PrEvidence {
        schema: PR_SCHEMA.to_string(),
        repository,
        number,
        url,
        state,
        is_draft,
        mergeable,
        merge_state_status,
        review_decision,
        head_sha,
        head_ref_name,
        base_ref_name,
        base_ref_oid,
        checks,
        review_threads,
        sources: Vec::new(),
    })
}

fn normalize_gh_pr_checks(path: &Path, checked_at: DateTime<Utc>) -> Result<PrEvidence> {
    let value = read_json::<Value>(path)?;
    let checks = checks_from_gh_pr_checks(&value, checked_at)?;
    Ok(partial_pr_evidence(
        "unknown",
        checks,
        empty_review_threads(checked_at),
    ))
}

fn normalize_gh_reviews(path: &Path, checked_at: DateTime<Utc>) -> Result<PrEvidence> {
    let value = read_json::<Value>(path)?;
    let reviews = array_or_paginated_arrays(&value, "GitHub reviews")?;
    let mut latest_by_reviewer: BTreeMap<String, (DateTime<Utc>, usize, String)> = BTreeMap::new();
    for (index, review) in reviews.iter().enumerate() {
        let state = optional_string(review, "state")
            .with_context(|| format!("GitHub review is missing state: {review}"))?
            .to_ascii_lowercase();
        let submitted_at = datetime_from_fields(
            review,
            &["submitted_at", "submittedAt", "updated_at", "updatedAt"],
        )
        .unwrap_or(checked_at);
        let reviewer = review_author_key(review, index);
        match latest_by_reviewer.get(&reviewer) {
            Some((latest_at, latest_index, _))
                if submitted_at < *latest_at
                    || (submitted_at == *latest_at && index <= *latest_index) => {}
            Some((_, _, existing_state))
                if state == "commented" && review_state_is_decisive(existing_state) => {}
            _ => {
                latest_by_reviewer.insert(reviewer, (submitted_at, index, state));
            }
        }
    }

    let mut pr = partial_pr_evidence("unknown", Vec::new(), empty_review_threads(checked_at));
    pr.review_decision = review_decision_from_reviewer_states(latest_by_reviewer.values());
    Ok(pr)
}

fn review_author_key(review: &Value, index: usize) -> String {
    for pointer in ["/user/login", "/author/login", "/user/id", "/author/id"] {
        if let Some(value) = json_scalar_key(review.pointer(pointer)) {
            return format!("{pointer}:{value}");
        }
    }
    json_scalar_key(review.get("id"))
        .map(|id| format!("review:{id}"))
        .unwrap_or_else(|| format!("review-index:{index}"))
}

fn review_state_is_decisive(state: &str) -> bool {
    matches!(state, "approved" | "changes_requested")
}

fn review_decision_from_reviewer_states<'a>(
    states: impl Iterator<Item = &'a (DateTime<Utc>, usize, String)>,
) -> Option<String> {
    let mut approved = false;
    let mut commented = false;
    for (_, _, state) in states {
        match state.as_str() {
            "changes_requested" => return Some("changes_requested".to_string()),
            "approved" => approved = true,
            "commented" => commented = true,
            "dismissed" | "pending" => {}
            _ => {}
        }
    }
    if approved {
        Some("approved".to_string())
    } else if commented {
        Some("commented".to_string())
    } else {
        None
    }
}

fn normalize_gh_review_threads(path: &Path, checked_at: DateTime<Utc>) -> Result<PrEvidence> {
    let value = read_json::<Value>(path)?;
    let review_threads = review_thread_summary_from_graphql(&value, checked_at)?;
    Ok(partial_pr_evidence("unknown", Vec::new(), review_threads))
}

fn normalize_gh_review_comments(path: &Path, checked_at: DateTime<Utc>) -> Result<PrEvidence> {
    let value = read_json::<Value>(path)?;
    let comments = array_or_paginated_arrays(&value, "GitHub review comments")?;
    let review_threads = review_thread_summary_from_rest_comments(&comments, checked_at);
    Ok(partial_pr_evidence("unknown", Vec::new(), review_threads))
}

fn review_thread_summary_from_rest_comments(
    comments: &[Value],
    checked_at: DateTime<Utc>,
) -> ReviewThreadSummary {
    let mut thread_outdated: BTreeMap<String, bool> = BTreeMap::new();
    for (index, comment) in comments.iter().enumerate() {
        let thread_key = review_comment_thread_key(comment, index);
        let is_outdated = review_comment_is_outdated(comment);
        thread_outdated
            .entry(thread_key)
            .and_modify(|all_outdated| *all_outdated &= is_outdated)
            .or_insert(is_outdated);
    }
    ReviewThreadSummary {
        unresolved: 0,
        total: thread_outdated.len() as u64,
        resolved: 0,
        outdated: thread_outdated
            .values()
            .filter(|all_outdated| **all_outdated)
            .count() as u64,
        authoritative: false,
        last_checked_at: checked_at,
    }
}

fn review_comment_thread_key(comment: &Value, index: usize) -> String {
    json_scalar_key(comment.get("in_reply_to_id"))
        .or_else(|| json_scalar_key(comment.get("inReplyToId")))
        .or_else(|| json_scalar_key(comment.get("id")))
        .map(|id| format!("thread:{id}"))
        .unwrap_or_else(|| format!("comment-index:{index}"))
}

fn review_comment_is_outdated(comment: &Value) -> bool {
    value_is_nullish(comment.get("position"))
        && (comment.get("original_position").is_some()
            || comment.get("originalPosition").is_some()
            || comment.get("originalLine").is_some()
            || comment.get("original_line").is_some())
}

fn normalize_review_pack_remaining(path: &Path, checked_at: DateTime<Utc>) -> Result<PrEvidence> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to open {}", path.display()))?;
    let unresolved = parse_review_pack_remaining(&contents)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(partial_pr_evidence(
        "unknown",
        Vec::new(),
        ReviewThreadSummary {
            unresolved,
            total: unresolved,
            resolved: 0,
            outdated: 0,
            authoritative: true,
            last_checked_at: checked_at,
        },
    ))
}

fn partial_pr_evidence(
    state: &str,
    checks: Vec<CheckRecord>,
    review_threads: ReviewThreadSummary,
) -> PrEvidence {
    PrEvidence {
        schema: PR_SCHEMA.to_string(),
        repository: None,
        number: None,
        url: None,
        state: state.to_string(),
        is_draft: None,
        mergeable: None,
        merge_state_status: None,
        review_decision: None,
        head_sha: None,
        head_ref_name: None,
        base_ref_name: None,
        base_ref_oid: None,
        checks,
        review_threads,
        sources: Vec::new(),
    }
}

fn merge_provider_pr_evidence(
    mut existing: PrEvidence,
    incoming: PrEvidence,
    source_kind: PrRecordSourceKind,
) -> Result<PrEvidence> {
    if source_kind == PrRecordSourceKind::Normalized {
        return Ok(incoming);
    }
    if existing.schema != PR_SCHEMA || incoming.schema != PR_SCHEMA {
        bail!("cannot merge PR evidence with unexpected schema");
    }

    merge_pr_string_field(&mut existing.repository, incoming.repository, "repository")?;
    merge_pr_number_field(&mut existing.number, incoming.number, "number")?;
    merge_pr_string_field(&mut existing.url, incoming.url, "url")?;

    if incoming.state != "unknown" {
        existing.state = incoming.state;
    } else if existing.state == "not_created" {
        existing.state = "unknown".to_string();
    }

    if incoming.mergeable.is_some() {
        existing.mergeable = incoming.mergeable;
    }
    if incoming.is_draft.is_some() {
        existing.is_draft = incoming.is_draft;
    }
    if incoming.merge_state_status.is_some() {
        existing.merge_state_status = incoming.merge_state_status;
    }
    if incoming.review_decision.is_some()
        && (source_kind != PrRecordSourceKind::GhReviews || existing.review_decision.is_none())
    {
        existing.review_decision = incoming.review_decision;
    }
    if incoming.head_sha.is_some() {
        existing.head_sha = incoming.head_sha;
    }
    if incoming.head_ref_name.is_some() {
        existing.head_ref_name = incoming.head_ref_name;
    }
    if incoming.base_ref_name.is_some() {
        existing.base_ref_name = incoming.base_ref_name;
    }
    if incoming.base_ref_oid.is_some() {
        existing.base_ref_oid = incoming.base_ref_oid;
    }

    match source_kind {
        PrRecordSourceKind::GhPrView => {
            if !incoming.checks.is_empty() {
                existing.checks = incoming.checks;
            }
            if incoming.review_threads.authoritative {
                existing.review_threads = incoming.review_threads;
            }
        }
        PrRecordSourceKind::GhPrChecks => {
            existing.checks = incoming.checks;
        }
        PrRecordSourceKind::GhReviews => {}
        PrRecordSourceKind::GhReviewThreads => {
            existing.review_threads = incoming.review_threads;
        }
        PrRecordSourceKind::GhReviewComments => {
            merge_review_comment_summary(&mut existing.review_threads, incoming.review_threads);
        }
        PrRecordSourceKind::ReviewPackRemaining => {
            merge_review_pack_summary(&mut existing.review_threads, incoming.review_threads);
        }
        PrRecordSourceKind::Normalized => unreachable!("handled before merge"),
    }

    existing.sources.extend(incoming.sources);
    Ok(existing)
}

fn merge_pr_string_field(
    existing: &mut Option<String>,
    incoming: Option<String>,
    field: &str,
) -> Result<()> {
    let Some(incoming) = incoming else {
        return Ok(());
    };
    if let Some(existing_value) = existing.as_deref()
        && existing_value != incoming
    {
        bail!("conflicting PR {field}: existing {existing_value}, incoming {incoming}");
    }
    *existing = Some(incoming);
    Ok(())
}

fn merge_pr_number_field(
    existing: &mut Option<u64>,
    incoming: Option<u64>,
    field: &str,
) -> Result<()> {
    let Some(incoming) = incoming else {
        return Ok(());
    };
    if let Some(existing_value) = *existing
        && existing_value != incoming
    {
        bail!("conflicting PR {field}: existing {existing_value}, incoming {incoming}");
    }
    *existing = Some(incoming);
    Ok(())
}

fn merge_review_pack_summary(existing: &mut ReviewThreadSummary, incoming: ReviewThreadSummary) {
    existing.unresolved = incoming.unresolved;
    existing.total = existing
        .total
        .max(incoming.total)
        .max(existing.unresolved + existing.resolved + existing.outdated);
    existing.authoritative = true;
    existing.last_checked_at = incoming.last_checked_at;
}

fn merge_review_comment_summary(existing: &mut ReviewThreadSummary, incoming: ReviewThreadSummary) {
    if existing.authoritative {
        existing.total = existing
            .total
            .max(incoming.total)
            .max(existing.unresolved + existing.resolved + existing.outdated);
        existing.last_checked_at = incoming.last_checked_at;
        return;
    }
    existing.outdated = incoming.outdated;
    existing.total = existing
        .total
        .max(incoming.total)
        .max(existing.unresolved + existing.resolved + existing.outdated);
    existing.authoritative = existing.authoritative || incoming.authoritative;
    existing.last_checked_at = incoming.last_checked_at;
}

pub fn recommend_pr_agent_actions(
    pr: &PrEvidence,
    diagnostics: &[PrAgentDiagnostic],
) -> Vec<PrAgentAction> {
    let mut actions = Vec::new();

    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == PrAgentSeverity::Error)
    {
        actions.push(PrAgentAction {
            id: "resolve_state_collection_errors".to_string(),
            priority: PrAgentActionPriority::Blocked,
            summary: "Resolve PR state collection errors before hosted action".to_string(),
            reason: "one or more required hosted-state sources failed to capture or normalize"
                .to_string(),
        });
    }

    let failed_checks = pr
        .checks
        .iter()
        .filter(|check| {
            matches!(
                check.conclusion.as_deref(),
                Some("failure" | "error" | "cancelled" | "canceled" | "timed_out")
            )
        })
        .count();
    if failed_checks > 0 {
        actions.push(PrAgentAction {
            id: "diagnose_failed_checks".to_string(),
            priority: PrAgentActionPriority::Required,
            summary: format!("Inspect {failed_checks} failed PR check(s)"),
            reason: "checks must be understood before review cleanup or merge".to_string(),
        });
    }

    let pending_checks = pr
        .checks
        .iter()
        .filter(|check| matches!(check.status.as_str(), "pending" | "queued" | "in_progress"))
        .count();
    if pending_checks > 0 && failed_checks == 0 {
        actions.push(PrAgentAction {
            id: "wait_for_checks".to_string(),
            priority: PrAgentActionPriority::Wait,
            summary: format!("Wait for {pending_checks} pending PR check(s)"),
            reason: "CI state is not final yet".to_string(),
        });
    }
    let unknown_check_outcomes = pr
        .checks
        .iter()
        .filter(|check| check.status == "completed" && check.conclusion.is_none())
        .count();
    if unknown_check_outcomes > 0 {
        actions.push(PrAgentAction {
            id: "inspect_check_outcomes".to_string(),
            priority: PrAgentActionPriority::Required,
            summary: format!("Inspect {unknown_check_outcomes} check(s) with unknown outcome"),
            reason: "completed checks require an explicit success, neutral, or skipped conclusion before merge".to_string(),
        });
    }

    if pr.review_threads.authoritative && pr.review_threads.unresolved > 0 {
        actions.push(PrAgentAction {
            id: "process_review_threads".to_string(),
            priority: PrAgentActionPriority::Required,
            summary: format!(
                "Process {} unresolved review thread(s)",
                pr.review_threads.unresolved
            ),
            reason: "hosted review threads are authoritative and not clean".to_string(),
        });
    } else if !pr.review_threads.authoritative {
        actions.push(PrAgentAction {
            id: "refresh_review_threads".to_string(),
            priority: PrAgentActionPriority::Info,
            summary: "Refresh authoritative review-thread state".to_string(),
            reason: "current review-thread source is partial or missing pagination closure"
                .to_string(),
        });
    }

    let review_decision_ready = match pr.review_decision.as_deref() {
        Some("changes_requested") => {
            actions.push(PrAgentAction {
                id: "process_requested_changes".to_string(),
                priority: PrAgentActionPriority::Required,
                summary: "Address or rebut requested changes".to_string(),
                reason: "latest review decision is changes_requested".to_string(),
            });
            false
        }
        Some("review_required") => {
            actions.push(PrAgentAction {
                id: "wait_for_required_review".to_string(),
                priority: PrAgentActionPriority::Wait,
                summary: "Wait for required approving review".to_string(),
                reason: "latest review decision is review_required".to_string(),
            });
            false
        }
        Some("approved") => true,
        None => {
            actions.push(PrAgentAction {
                id: "refresh_review_decision".to_string(),
                priority: PrAgentActionPriority::Wait,
                summary: "Refresh review decision before merge".to_string(),
                reason: "merge readiness requires an explicit review decision".to_string(),
            });
            false
        }
        Some(other) => {
            actions.push(PrAgentAction {
                id: "inspect_review_decision".to_string(),
                priority: PrAgentActionPriority::Info,
                summary: format!("Inspect review decision {other}"),
                reason: "review decision is not explicitly approved".to_string(),
            });
            false
        }
    };

    let mergeable_ready = match pr.mergeable.as_deref() {
        Some("mergeable" | "clean") => true,
        Some("conflicting" | "dirty") => {
            actions.push(PrAgentAction {
                id: "resolve_merge_conflict".to_string(),
                priority: PrAgentActionPriority::Required,
                summary: "Resolve merge conflict before merge".to_string(),
                reason: "GitHub reports the pull request is not cleanly mergeable".to_string(),
            });
            false
        }
        Some("unknown") | None => {
            actions.push(PrAgentAction {
                id: "wait_for_mergeability".to_string(),
                priority: PrAgentActionPriority::Wait,
                summary: "Wait for GitHub mergeability state".to_string(),
                reason: "GitHub mergeability is not yet known".to_string(),
            });
            false
        }
        Some(other) => {
            actions.push(PrAgentAction {
                id: "inspect_mergeability".to_string(),
                priority: PrAgentActionPriority::Required,
                summary: format!("Inspect mergeability state {other}"),
                reason: "GitHub reports a non-clean mergeability state".to_string(),
            });
            false
        }
    };

    if pr.state == "draft" {
        actions.push(PrAgentAction {
            id: "mark_ready_when_complete".to_string(),
            priority: PrAgentActionPriority::Info,
            summary: "Mark draft PR ready when local gates are complete".to_string(),
            reason: "draft pull requests do not enter the normal review-to-merge path".to_string(),
        });
    }

    if actions.is_empty()
        && pr.state == "open"
        && pr.checks.iter().all(|check| {
            check.status == "completed"
                && matches!(
                    check.conclusion.as_deref(),
                    Some("success" | "neutral" | "skipped")
                )
        })
        && unknown_check_outcomes == 0
        && pr.review_threads.authoritative
        && pr.review_threads.unresolved == 0
        && review_decision_ready
        && mergeable_ready
    {
        actions.push(PrAgentAction {
            id: "merge_when_policy_allows".to_string(),
            priority: PrAgentActionPriority::Ready,
            summary: "PR state is clean for policy-controlled merge".to_string(),
            reason:
                "checks, review threads, review decision, and mergeability have no blocking state"
                    .to_string(),
        });
    }

    actions
}

impl PrSnapshotInput {
    fn into_pr_evidence(self, checked_at: DateTime<Utc>) -> PrEvidence {
        PrEvidence {
            schema: PR_SCHEMA.to_string(),
            repository: self.repository,
            number: self.number,
            url: self.url,
            state: self.state.to_ascii_lowercase(),
            is_draft: self.is_draft,
            mergeable: self.mergeable.map(|value| value.to_ascii_lowercase()),
            merge_state_status: self
                .merge_state_status
                .map(|value| value.to_ascii_lowercase()),
            review_decision: self.review_decision.map(|value| value.to_ascii_lowercase()),
            head_sha: self.head_sha,
            head_ref_name: self.head_ref_name,
            base_ref_name: self.base_ref_name,
            base_ref_oid: self.base_ref_oid,
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
                total: self.review_threads.total,
                resolved: self.review_threads.resolved,
                outdated: self.review_threads.outdated,
                authoritative: self.review_threads.authoritative.unwrap_or(true),
                last_checked_at: self.review_threads.last_checked_at.unwrap_or(checked_at),
            },
            sources: self.sources,
        }
    }
}

fn checks_from_status_rollup(value: &Value, checked_at: DateTime<Utc>) -> Result<Vec<CheckRecord>> {
    array_from_value(value, "GitHub statusCheckRollup")?
        .iter()
        .map(|check| check_from_github_value(check, checked_at))
        .collect()
}

fn checks_from_gh_pr_checks(value: &Value, checked_at: DateTime<Utc>) -> Result<Vec<CheckRecord>> {
    array_from_value(value, "gh pr checks")?
        .iter()
        .map(|check| check_from_github_value(check, checked_at))
        .collect()
}

fn check_from_github_value(value: &Value, checked_at: DateTime<Utc>) -> Result<CheckRecord> {
    let name = optional_string(value, "name")
        .or_else(|| optional_string(value, "context"))
        .with_context(|| format!("GitHub check is missing name/context: {value}"))?;
    let status = check_lifecycle_status(value)
        .with_context(|| format!("GitHub check {name} is missing status/state/bucket"))?;
    let conclusion = optional_string(value, "conclusion")
        .or_else(|| optional_string(value, "state").and_then(state_to_conclusion))
        .or_else(|| optional_string(value, "bucket").and_then(bucket_to_conclusion))
        .map(|conclusion| conclusion.to_ascii_lowercase());
    let url = optional_string(value, "detailsUrl")
        .or_else(|| optional_string(value, "targetUrl"))
        .or_else(|| optional_string(value, "link"))
        .or_else(|| optional_string(value, "url"))
        .filter(|url| !url.is_empty());
    let checked_at = datetime_from_fields(
        value,
        &["completedAt", "startedAt", "updatedAt", "createdAt"],
    )
    .unwrap_or(checked_at);

    Ok(CheckRecord {
        name,
        status,
        conclusion,
        url,
        checked_at,
    })
}

fn check_lifecycle_status(value: &Value) -> Option<String> {
    if let Some(status) = optional_string(value, "status") {
        return Some(normalize_lifecycle_status(&status));
    }
    if let Some(state) = optional_string(value, "state") {
        return Some(lifecycle_status_from_state(&state));
    }
    optional_string(value, "bucket").map(|bucket| lifecycle_status_from_bucket(&bucket))
}

fn normalize_lifecycle_status(value: &str) -> String {
    match value.to_ascii_lowercase().as_str() {
        "completed" | "complete" => "completed".to_string(),
        "in_progress" | "in progress" | "running" => "in_progress".to_string(),
        "queued" => "queued".to_string(),
        "pending" => "pending".to_string(),
        other => other.to_string(),
    }
}

fn lifecycle_status_from_state(value: &str) -> String {
    match value.to_ascii_lowercase().as_str() {
        "success" | "failure" | "error" | "cancelled" | "canceled" | "skipped" => {
            "completed".to_string()
        }
        "pending" => "pending".to_string(),
        "queued" => "queued".to_string(),
        "in_progress" | "in progress" | "running" => "in_progress".to_string(),
        other => other.to_string(),
    }
}

fn lifecycle_status_from_bucket(value: &str) -> String {
    match value.to_ascii_lowercase().as_str() {
        "pass" | "fail" | "cancel" | "skipping" => "completed".to_string(),
        "pending" => "pending".to_string(),
        other => other.to_string(),
    }
}

fn review_thread_summary_from_graphql(
    value: &Value,
    checked_at: DateTime<Utc>,
) -> Result<ReviewThreadSummary> {
    if let Some(pages) = value.as_array()
        && pages.iter().any(Value::is_object)
    {
        return review_thread_summary_from_graphql_pages(pages, checked_at);
    }

    let nodes = review_thread_nodes(value)
        .with_context(|| "GitHub review-thread source is missing reviewThreads.nodes")?;
    let (unresolved, resolved, outdated) = count_review_thread_nodes(nodes)?;

    Ok(ReviewThreadSummary {
        unresolved,
        total: nodes.len() as u64,
        resolved,
        outdated,
        authoritative: matches!(review_threads_has_next_page(value), Some(false)),
        last_checked_at: checked_at,
    })
}

fn review_thread_summary_from_graphql_pages(
    pages: &[Value],
    checked_at: DateTime<Utc>,
) -> Result<ReviewThreadSummary> {
    let mut unresolved = 0;
    let mut resolved = 0;
    let mut outdated = 0;
    let mut total = 0;
    let mut last_has_next_page = None;

    for page in pages {
        let nodes = review_thread_nodes(page)
            .with_context(|| "GitHub review-thread page is missing reviewThreads.nodes")?;
        let (page_unresolved, page_resolved, page_outdated) = count_review_thread_nodes(nodes)?;
        unresolved += page_unresolved;
        resolved += page_resolved;
        outdated += page_outdated;
        total += nodes.len() as u64;
        last_has_next_page = review_threads_has_next_page(page);
    }

    Ok(ReviewThreadSummary {
        unresolved,
        total,
        resolved,
        outdated,
        authoritative: matches!(last_has_next_page, Some(false)),
        last_checked_at: checked_at,
    })
}

fn count_review_thread_nodes(nodes: &[Value]) -> Result<(u64, u64, u64)> {
    let mut unresolved = 0;
    let mut resolved = 0;
    let mut outdated = 0;

    for node in nodes {
        let is_resolved = optional_bool(node, "isResolved")
            .with_context(|| format!("GitHub review thread is missing isResolved: {node}"))?;
        let is_outdated = optional_bool(node, "isOutdated").unwrap_or(false);
        if is_resolved {
            resolved += 1;
        } else if is_outdated {
            outdated += 1;
        } else {
            unresolved += 1;
        }
    }

    Ok((unresolved, resolved, outdated))
}

fn review_thread_nodes(value: &Value) -> Option<&Vec<Value>> {
    value
        .pointer("/data/repository/pullRequest/reviewThreads/nodes")
        .or_else(|| value.pointer("/repository/pullRequest/reviewThreads/nodes"))
        .or_else(|| value.pointer("/pullRequest/reviewThreads/nodes"))
        .or_else(|| value.pointer("/reviewThreads/nodes"))
        .or_else(|| value.pointer("/nodes"))
        .and_then(Value::as_array)
}

fn review_threads_has_next_page(value: &Value) -> Option<bool> {
    value
        .pointer("/data/repository/pullRequest/reviewThreads/pageInfo/hasNextPage")
        .or_else(|| value.pointer("/repository/pullRequest/reviewThreads/pageInfo/hasNextPage"))
        .or_else(|| value.pointer("/pullRequest/reviewThreads/pageInfo/hasNextPage"))
        .or_else(|| value.pointer("/reviewThreads/pageInfo/hasNextPage"))
        .or_else(|| value.pointer("/pageInfo/hasNextPage"))
        .and_then(Value::as_bool)
}

fn parse_review_pack_remaining(contents: &str) -> Result<u64> {
    let trimmed = contents.trim();
    if let Ok(value) = serde_json::from_str::<Value>(trimmed)
        && let Some(unresolved) = review_pack_remaining_from_json(&value)
    {
        return Ok(unresolved);
    }
    if let Ok(unresolved) = trimmed.parse::<u64>() {
        return Ok(unresolved);
    }
    for line in contents.lines() {
        if let Some(unresolved) = review_pack_count_from_line(line) {
            return Ok(unresolved);
        }
    }
    bail!("review-pack remaining output did not include an unresolved count")
}

fn review_pack_remaining_from_json(value: &Value) -> Option<u64> {
    [
        "/unresolved",
        "/unresolved_threads",
        "/remaining",
        "/review_threads/unresolved",
        "/result/unresolved",
        "/result/unresolved_threads",
        "/summary/unresolved",
    ]
    .iter()
    .find_map(|pointer| value.pointer(pointer).and_then(Value::as_u64))
}

fn review_pack_count_from_line(value: &str) -> Option<u64> {
    let lower = value.to_ascii_lowercase();
    for key in [
        "unresolved_threads",
        "unresolved threads",
        "unresolved",
        "remaining",
    ] {
        let Some(index) = lower.find(key) else {
            continue;
        };
        if let Some(count) = trailing_u64(value[..index].trim()) {
            return Some(count);
        }
        if let Some(count) = leading_u64_after_key(&value[index + key.len()..]) {
            return Some(count);
        }
    }
    None
}

fn trailing_u64(value: &str) -> Option<u64> {
    let token = value
        .split(|ch: char| !ch.is_ascii_digit())
        .rfind(|part| !part.is_empty())?;
    if value.trim() == token {
        token.parse().ok()
    } else {
        None
    }
}

fn leading_u64_after_key(value: &str) -> Option<u64> {
    let value =
        value.trim_start_matches(|ch: char| ch.is_whitespace() || matches!(ch, ':' | '=' | '-'));
    let digits = value
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn empty_review_threads(checked_at: DateTime<Utc>) -> ReviewThreadSummary {
    ReviewThreadSummary {
        unresolved: 0,
        total: 0,
        resolved: 0,
        outdated: 0,
        authoritative: false,
        last_checked_at: checked_at,
    }
}

fn optional_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn optional_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn optional_bool(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn json_scalar_key(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) if !value.trim().is_empty() => Some(value.trim().to_string()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn array_from_value<'a>(value: &'a Value, label: &str) -> Result<&'a Vec<Value>> {
    value
        .as_array()
        .with_context(|| format!("{label} source must be a JSON array"))
}

fn array_or_paginated_arrays(value: &Value, label: &str) -> Result<Vec<Value>> {
    let array = array_from_value(value, label)?;
    if array.iter().all(Value::is_array) {
        return Ok(array
            .iter()
            .flat_map(|page| page.as_array().into_iter().flatten().cloned())
            .collect());
    }
    Ok(array.clone())
}

fn datetime_from_fields(value: &Value, fields: &[&str]) -> Option<DateTime<Utc>> {
    fields.iter().find_map(|field| {
        value
            .get(*field)
            .and_then(Value::as_str)
            .and_then(parse_github_datetime)
    })
}

fn parse_github_datetime(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if value.is_empty() || value.starts_with("0001-01-01T00:00:00") {
        return None;
    }
    value.parse().ok()
}

fn bucket_to_conclusion(value: String) -> Option<String> {
    match value.as_str() {
        "pass" => Some("success".to_string()),
        "fail" => Some("failure".to_string()),
        "cancel" => Some("cancelled".to_string()),
        "skipping" => Some("skipped".to_string()),
        _ => None,
    }
}

fn state_to_conclusion(value: String) -> Option<String> {
    match value.to_ascii_lowercase().as_str() {
        "success" | "failure" | "error" | "cancelled" | "canceled" | "skipped" => {
            Some(value.to_ascii_lowercase())
        }
        _ => None,
    }
}

fn value_is_nullish(value: Option<&Value>) -> bool {
    matches!(value, None | Some(Value::Null))
}

fn repository_from_pr_url(url: &str) -> Option<String> {
    let mut parts = github_pr_url_parts(url)?;
    Some(format!("{}/{}", parts.next()?, parts.next()?))
}

fn number_from_pr_url(url: &str) -> Option<u64> {
    let mut parts = github_pr_url_parts(url)?;
    parts.nth(3)?.parse().ok()
}

fn github_pr_url_parts(url: &str) -> Option<impl Iterator<Item = &str>> {
    let path = url.split("github.com/").nth(1)?;
    let mut parts = path.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    let pull = parts.next()?;
    if owner.is_empty() || repo.is_empty() || pull != "pull" {
        return None;
    }
    Some(path.split('/'))
}

pub fn render_pr_status(pr: &PrEvidence) -> String {
    format!(
        "{} {}: {}, {} check(s)",
        render_pr_label(pr),
        pr.state,
        render_review_thread_summary(&pr.review_threads),
        pr.checks.len()
    )
}

fn render_review_thread_summary(review_threads: &ReviewThreadSummary) -> String {
    if review_threads.authoritative {
        format!("{} unresolved review thread(s)", review_threads.unresolved)
    } else {
        "review threads not checked".to_string()
    }
}

pub fn render_pr_label(pr: &PrEvidence) -> String {
    match (&pr.repository, pr.number) {
        (Some(repository), Some(number)) => format!("{repository}#{number}"),
        (None, Some(number)) => format!("#{number}"),
        (Some(repository), None) => repository.clone(),
        (None, None) => "unlinked PR".to_string(),
    }
}

fn subspawn_plan_to_batch(
    plan: SubspawnPlanInput,
    batch_id: &str,
    recorded_at: DateTime<Utc>,
) -> Result<SubagentBatch> {
    let mut errors = Vec::new();
    validate_non_empty_text("task", &plan.task, &mut errors);
    validate_optional_text("mode", plan.mode.as_deref(), &mut errors);
    validate_optional_text("scope", plan.scope.as_deref(), &mut errors);
    validate_repeated_text("scope_items", &plan.scope_items, &mut errors);
    validate_optional_text("wait_policy", plan.wait_policy.as_deref(), &mut errors);
    validate_repeated_text("registry_issues", &plan.registry_issues, &mut errors);
    validate_repeated_text(
        "synthesis_checklist",
        &plan.synthesis_checklist,
        &mut errors,
    );
    if plan.roles.is_empty() {
        errors.push("roles must not be empty".to_string());
    }

    let mut seen = BTreeSet::new();
    for role in &plan.roles {
        if let Err(error) = validate_role_name(&role.name) {
            errors.push(error.to_string());
        }
        validate_optional_text("role.description", role.description.as_deref(), &mut errors);
        validate_optional_text("role.model", role.model.as_deref(), &mut errors);
        validate_optional_text("role.path", role.path.as_deref(), &mut errors);
        validate_optional_text("role.reasoning", role.reasoning.as_deref(), &mut errors);
        validate_repeated_text("role.return_headings", &role.return_headings, &mut errors);
        validate_optional_text("role.sandbox", role.sandbox.as_deref(), &mut errors);
        validate_optional_text("role.source", role.source.as_deref(), &mut errors);
        if !seen.insert(role.name.clone()) {
            errors.push(format!("duplicate role in plan: {}", role.name));
        }
    }
    let role_names = seen.clone();
    let mut prompt_roles = BTreeSet::new();
    for prompt in &plan.prompts {
        if let Err(error) = validate_role_name(&prompt.role) {
            errors.push(error.to_string());
        }
        validate_multiline_text("prompt", &prompt.prompt, &mut errors);
        if !role_names.contains(&prompt.role) {
            errors.push(format!(
                "prompt role {} is not present in plan roles",
                prompt.role
            ));
        }
        if !prompt_roles.insert(prompt.role.clone()) {
            errors.push(format!("duplicate prompt for role {}", prompt.role));
        }
    }
    for (role, paths) in &plan.duplicate_roles_ignored {
        if let Err(error) = validate_role_name(role) {
            errors.push(format!("duplicate_roles_ignored {error}"));
        }
        if paths.is_empty() {
            errors.push(format!("duplicate_roles_ignored[{role}] must not be empty"));
        }
        validate_repeated_text("duplicate_roles_ignored", paths, &mut errors);
    }
    if !errors.is_empty() {
        bail!("invalid subspawn plan: {}", errors.join("; "));
    }

    let prompts_by_role = plan
        .prompts
        .into_iter()
        .map(|prompt| (prompt.role, prompt.prompt))
        .collect::<BTreeMap<_, _>>();
    let mut prompts = Vec::new();
    let mut agents = Vec::new();
    for role in plan.roles {
        let prompt = prompts_by_role
            .get(&role.name)
            .with_context(|| format!("missing prompt for role {}", role.name))?;
        let prompt_id = format!("{batch_id}:{}", role.name);
        let prompt_hash = stable_prompt_hash(prompt);
        prompts.push(SubagentPromptRecord {
            role: role.name.clone(),
            prompt_id: prompt_id.clone(),
            prompt_hash: prompt_hash.clone(),
        });
        agents.push(SubagentRecord {
            role: role.name,
            task: plan.task.clone(),
            status: "planned".to_string(),
            summary: "planned by subspawn".to_string(),
            prompt_id: Some(prompt_id),
            prompt_hash: Some(prompt_hash),
            disposition: None,
            human_verified: false,
            source_ids: Vec::new(),
            artifacts: Vec::new(),
            updated_at: None,
        });
    }

    Ok(SubagentBatch {
        id: batch_id.to_string(),
        status: "planned".to_string(),
        task: Some(plan.task),
        mode: plan.mode,
        scope: plan.scope,
        wait_policy: plan.wait_policy,
        rendezvous_required: plan.rendezvous_required,
        registry_issues: plan.registry_issues,
        duplicate_roles_ignored: plan.duplicate_roles_ignored,
        prompts,
        agents,
        synthesis: None,
        recorded_at: Some(recorded_at),
        updated_at: Some(recorded_at),
    })
}

pub fn stable_json_hash<T: Serialize>(value: &T) -> Result<String> {
    let bytes = serde_json::to_vec(value)?;
    Ok(stable_sha256_hash(&bytes))
}

fn stable_prompt_hash(prompt: &str) -> String {
    stable_sha256_hash(prompt.as_bytes())
}

fn stable_sha256_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{digest:x}")
}

fn refresh_batch_status(batch: &mut SubagentBatch) {
    if batch
        .agents
        .iter()
        .any(|agent| agent.status.as_str() == "blocked")
    {
        batch.status = "blocked".to_string();
    } else if batch
        .agents
        .iter()
        .any(|agent| matches!(agent.status.as_str(), "planned" | "running"))
    {
        batch.status = "active".to_string();
    } else if !batch.agents.is_empty()
        && batch.agents.iter().all(|agent| agent.status == "completed")
    {
        batch.status = "completed".to_string();
    } else {
        batch.status = "partial".to_string();
    }
}

fn apply_synthesis_status(batch: &mut SubagentBatch, status: SubagentSynthesisStatus) {
    refresh_batch_status(batch);
    match status {
        SubagentSynthesisStatus::Blocked => {
            batch.status = "blocked".to_string();
        }
        SubagentSynthesisStatus::Partial if batch.status == "completed" => {
            batch.status = "partial".to_string();
        }
        SubagentSynthesisStatus::Partial | SubagentSynthesisStatus::Completed => {}
    }
}

fn validate_outcome_disposition(
    status: SubagentOutcomeStatus,
    disposition: SubagentDisposition,
) -> Result<()> {
    if matches!(
        status,
        SubagentOutcomeStatus::Completed
            | SubagentOutcomeStatus::Failed
            | SubagentOutcomeStatus::TimedOut
            | SubagentOutcomeStatus::Closed
    ) && disposition == SubagentDisposition::Pending
    {
        bail!("terminal subagent outcomes require a final disposition");
    }
    Ok(())
}

fn ensure_completed_synthesis_ready(batch: &SubagentBatch) -> Result<()> {
    if batch.agents.is_empty() {
        bail!("completed synthesis requires at least one planned agent");
    }
    let incomplete = batch
        .agents
        .iter()
        .filter(|agent| !subagent_has_final_verified_outcome(agent))
        .map(|agent| agent.role.as_str())
        .collect::<Vec<_>>();
    if !incomplete.is_empty() {
        bail!(
            "completed synthesis requires terminal human-verified outcomes for every agent; incomplete roles: {}",
            incomplete.join(", ")
        );
    }
    Ok(())
}

fn subagent_has_final_verified_outcome(agent: &SubagentRecord) -> bool {
    is_terminal_subagent_status(&agent.status)
        && agent.human_verified
        && matches!(
            agent.disposition.as_deref(),
            Some("accepted" | "rejected" | "mixed" | "informational")
        )
}

fn is_terminal_subagent_status(value: &str) -> bool {
    matches!(value, "completed" | "failed" | "timed_out" | "closed")
}

fn is_batch_status(value: &str) -> bool {
    matches!(
        value,
        "planned" | "active" | "completed" | "partial" | "blocked"
    )
}

fn subagent_source_ids(batch_id: &str, role: &str, extra: &[String]) -> Vec<String> {
    let mut source_ids = vec![
        format!("subagents:{batch_id}"),
        format!("subagent:{batch_id}:{role}"),
    ];
    source_ids.extend(extra.iter().cloned());
    source_ids
}

fn synthesis_source_ids(batch_id: &str, extra: &[String]) -> Vec<String> {
    let mut source_ids = vec![format!("subagents:{batch_id}:synthesis")];
    source_ids.extend(extra.iter().cloned());
    source_ids
}

fn subagent_artifacts(extra: &[String]) -> Vec<String> {
    let mut artifacts = vec!["subagents.json".to_string()];
    artifacts.extend(extra.iter().cloned());
    artifacts
}

fn append_subagent_evidence(capsule_path: &Path, record: EvidenceRecord) -> Result<()> {
    validate_subagent_evidence_record(&record)?;
    append_jsonl(capsule_path.join("evidence.jsonl"), &record)
}

fn validate_subagent_evidence_record(record: &EvidenceRecord) -> Result<()> {
    let errors = validate_evidence_record(record);
    if !errors.is_empty() {
        bail!("invalid subagent evidence record: {}", errors.join("; "));
    }
    Ok(())
}

fn touch_capsule(capsule_path: &Path, updated_at: DateTime<Utc>) -> Result<()> {
    let mut capsule: Capsule = read_json(&capsule_path.join("capsule.json"))?;
    capsule.updated_at = std::cmp::max(capsule.updated_at, updated_at);
    write_json(capsule_path.join("capsule.json"), &capsule)
}

fn validate_capsule_for_subagent_record(capsule_path: &Path) -> Result<()> {
    ensure_regular_contract_files(capsule_path)?;
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

fn validate_stable_id(field: &str, value: &str) -> Result<()> {
    let mut errors = Vec::new();
    validate_non_empty_text(field, value, &mut errors);
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':'))
    {
        errors.push(format!(
            "{field} may contain only ASCII letters, numbers, '.', ':', '_' or '-'"
        ));
    }
    if !errors.is_empty() {
        bail!("{}", errors.join("; "));
    }
    Ok(())
}

fn validate_role_name(value: &str) -> Result<()> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        bail!("role must not be empty");
    };
    if !first.is_ascii_lowercase() {
        bail!("role {value:?} must start with a lowercase ASCII letter");
    }
    if !chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_') {
        bail!("role {value:?} must be snake_case");
    }
    Ok(())
}

fn validate_human_verified(value: bool) -> Result<()> {
    if !value {
        bail!("human_verified must be set for subagent outcome and synthesis records");
    }
    Ok(())
}

fn validate_multiline_text(field: &str, value: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{field} must not be empty"));
    }
    if value
        .chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
    {
        errors.push(format!(
            "{field} must not contain control characters other than tabs or newlines"
        ));
    }
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

fn validate_subagents(path: &Path) -> Result<Vec<String>> {
    let subagents: Subagents = read_json(path)?;
    Ok(validate_subagents_value(&subagents))
}

fn ensure_valid_subagents_value(subagents: &Subagents) -> Result<()> {
    let errors = validate_subagents_value(subagents);
    if !errors.is_empty() {
        bail!("invalid subagents contract: {}", errors.join("; "));
    }
    Ok(())
}

fn validate_subagents_value(subagents: &Subagents) -> Vec<String> {
    let mut errors = Vec::new();
    if subagents.schema != SUBAGENTS_SCHEMA {
        errors.push(format!("subagents.json schema must be {SUBAGENTS_SCHEMA}"));
    }

    let mut batch_ids = BTreeSet::new();
    for (batch_index, batch) in subagents.batches.iter().enumerate() {
        let prefix = format!("subagents.json batches[{batch_index}]");
        if let Err(error) = validate_stable_id("batch id", &batch.id) {
            errors.push(format!("{prefix} {error}"));
        }
        if !batch_ids.insert(batch.id.clone()) {
            errors.push(format!("{prefix} duplicate batch id {}", batch.id));
        }
        if !is_batch_status(&batch.status) {
            errors.push(format!(
                "{prefix} status {:?} must be planned, active, completed, partial, or blocked",
                batch.status
            ));
        }
        if batch.prompts.is_empty() {
            errors.push(format!("{prefix} prompts must not be empty"));
        }
        if batch.agents.is_empty() {
            errors.push(format!("{prefix} agents must not be empty"));
        }
        validate_optional_text(
            &format!("{prefix} task"),
            batch.task.as_deref(),
            &mut errors,
        );
        validate_optional_text(
            &format!("{prefix} mode"),
            batch.mode.as_deref(),
            &mut errors,
        );
        validate_optional_text(
            &format!("{prefix} scope"),
            batch.scope.as_deref(),
            &mut errors,
        );
        validate_optional_text(
            &format!("{prefix} wait_policy"),
            batch.wait_policy.as_deref(),
            &mut errors,
        );
        validate_repeated_text(
            &format!("{prefix} registry_issues"),
            &batch.registry_issues,
            &mut errors,
        );
        for (role, paths) in &batch.duplicate_roles_ignored {
            if let Err(error) = validate_role_name(role) {
                errors.push(format!("{prefix} duplicate_roles_ignored {error}"));
            }
            if paths.is_empty() {
                errors.push(format!(
                    "{prefix} duplicate_roles_ignored[{role}] must not be empty"
                ));
            }
            validate_repeated_text(
                &format!("{prefix} duplicate_roles_ignored[{role}]"),
                paths,
                &mut errors,
            );
        }

        let mut prompt_roles = BTreeMap::new();
        for (prompt_index, prompt) in batch.prompts.iter().enumerate() {
            let prompt_prefix = format!("{prefix} prompts[{prompt_index}]");
            if let Err(error) = validate_role_name(&prompt.role) {
                errors.push(format!("{prompt_prefix} {error}"));
            }
            if let Err(error) = validate_stable_id("prompt_id", &prompt.prompt_id) {
                errors.push(format!("{prompt_prefix} {error}"));
            }
            validate_prompt_hash(&prompt_prefix, &prompt.prompt_hash, &mut errors);
            if prompt_roles.insert(prompt.role.clone(), prompt).is_some() {
                errors.push(format!(
                    "{prompt_prefix} duplicate prompt for {}",
                    prompt.role
                ));
            }
        }

        let mut agent_roles = BTreeSet::new();
        for (agent_index, agent) in batch.agents.iter().enumerate() {
            let agent_prefix = format!("{prefix} agents[{agent_index}]");
            if let Err(error) = validate_role_name(&agent.role) {
                errors.push(format!("{agent_prefix} {error}"));
            }
            if !agent_roles.insert(agent.role.clone()) {
                errors.push(format!("{agent_prefix} duplicate agent for {}", agent.role));
            }
            if let Err(error) = SubagentOutcomeStatus::from_str(&agent.status) {
                errors.push(format!("{agent_prefix} {error}"));
            }
            validate_non_empty_text(&format!("{agent_prefix} task"), &agent.task, &mut errors);
            validate_non_empty_text(
                &format!("{agent_prefix} summary"),
                &agent.summary,
                &mut errors,
            );
            match agent.prompt_id.as_deref() {
                Some(prompt_id) => {
                    if let Err(error) = validate_stable_id("prompt_id", prompt_id) {
                        errors.push(format!("{agent_prefix} {error}"));
                    }
                }
                None => errors.push(format!("{agent_prefix} prompt_id is required")),
            }
            match agent.prompt_hash.as_deref() {
                Some(prompt_hash) => validate_prompt_hash(&agent_prefix, prompt_hash, &mut errors),
                None => errors.push(format!("{agent_prefix} prompt_hash is required")),
            }
            if let Some(disposition) = agent.disposition.as_deref()
                && let Err(error) = SubagentDisposition::from_str(disposition)
            {
                errors.push(format!("{agent_prefix} {error}"));
            }
            if agent.disposition.is_some() && !agent.human_verified {
                errors.push(format!(
                    "{agent_prefix} disposition requires human_verified=true"
                ));
            }
            if agent.human_verified && agent.disposition.is_none() {
                errors.push(format!(
                    "{agent_prefix} human_verified=true requires disposition"
                ));
            }
            if is_terminal_subagent_status(&agent.status)
                && !subagent_has_final_verified_outcome(agent)
            {
                errors.push(format!(
                    "{agent_prefix} terminal status requires a final human-verified disposition"
                ));
            }
            validate_repeated_text(
                &format!("{agent_prefix} source_ids"),
                &agent.source_ids,
                &mut errors,
            );
            validate_repeated_text(
                &format!("{agent_prefix} artifacts"),
                &agent.artifacts,
                &mut errors,
            );
            if let Some(prompt) = prompt_roles.get(&agent.role) {
                let expected_prompt_id = format!("{}:{}", batch.id, agent.role);
                if agent.prompt_id.as_deref() != Some(expected_prompt_id.as_str()) {
                    errors.push(format!(
                        "{agent_prefix} prompt_id must match {expected_prompt_id}"
                    ));
                }
                if agent.prompt_hash.as_deref() != Some(prompt.prompt_hash.as_str()) {
                    errors.push(format!(
                        "{agent_prefix} prompt_hash must match prompt record"
                    ));
                }
            }
        }
        for role in prompt_roles.keys() {
            if !agent_roles.contains(role) {
                errors.push(format!("{prefix} prompt role {role} has no matching agent"));
            }
        }
        for role in &agent_roles {
            if !prompt_roles.contains_key(role) {
                errors.push(format!("{prefix} agent role {role} has no matching prompt"));
            }
        }
        if batch.status == "completed" {
            for agent in batch
                .agents
                .iter()
                .filter(|agent| !subagent_has_final_verified_outcome(agent))
            {
                errors.push(format!(
                    "{prefix} completed batch has incomplete agent {}",
                    agent.role
                ));
            }
        }
        if let Some(synthesis) = &batch.synthesis {
            let synthesis_prefix = format!("{prefix} synthesis");
            if let Err(error) = SubagentSynthesisStatus::from_str(&synthesis.status) {
                errors.push(format!("{synthesis_prefix} {error}"));
            }
            validate_non_empty_text(
                &format!("{synthesis_prefix} summary"),
                &synthesis.summary,
                &mut errors,
            );
            if !synthesis.human_verified {
                errors.push(format!("{synthesis_prefix} requires human_verified=true"));
            }
            validate_required_repeated_text(
                &format!("{synthesis_prefix} source_ids"),
                &synthesis.source_ids,
                &mut errors,
            );
            validate_required_repeated_text(
                &format!("{synthesis_prefix} artifacts"),
                &synthesis.artifacts,
                &mut errors,
            );
            if synthesis.status == "completed" {
                for agent in batch
                    .agents
                    .iter()
                    .filter(|agent| !subagent_has_final_verified_outcome(agent))
                {
                    errors.push(format!(
                        "{synthesis_prefix} completed with incomplete agent {}",
                        agent.role
                    ));
                }
            }
        }
    }

    errors
}

fn validate_prompt_hash(field: &str, value: &str, errors: &mut Vec<String>) {
    let Some(hex) = value.strip_prefix("sha256:") else {
        errors.push(format!("{field} prompt_hash must start with sha256:"));
        return;
    };
    if hex.len() != 64
        || !hex
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
    {
        errors.push(format!(
            "{field} prompt_hash must be sha256: followed by 64 lowercase hex characters"
        ));
    }
}

fn validate_evidence(path: &Path) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    for_each_evidence_record(path, |line_number, record| {
        for error in validate_evidence_record(&record) {
            errors.push(format!("evidence.jsonl line {line_number} {error}"));
        }
        Ok(())
    })?;
    Ok(errors)
}

fn for_each_evidence_record(
    path: &Path,
    mut visit: impl FnMut(usize, EvidenceRecord) -> Result<()>,
) -> Result<()> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: EvidenceRecord = serde_json::from_str(&line)
            .with_context(|| format!("line {} is not valid evidence JSON", index + 1))?;
        visit(index + 1, record)?;
    }
    Ok(())
}

fn validate_evidence_record(record: &EvidenceRecord) -> Vec<String> {
    let mut errors = Vec::new();
    if record.schema != EVIDENCE_SCHEMA {
        errors.push(format!("schema must be {EVIDENCE_SCHEMA}"));
    }
    validate_non_empty_text("summary", &record.summary, &mut errors);
    validate_optional_text("command", record.command.as_deref(), &mut errors);
    validate_optional_text("actor", record.actor.as_deref(), &mut errors);
    validate_optional_text("tool", record.tool.as_deref(), &mut errors);
    validate_optional_text(
        "residual_risk",
        record.residual_risk.as_deref(),
        &mut errors,
    );
    validate_repeated_text("source_ids", &record.source_ids, &mut errors);
    validate_repeated_text("artifacts", &record.artifacts, &mut errors);
    if record.exit_code.is_some() && record.command.is_none() {
        errors.push("exit_code requires command".to_string());
    }
    if let Some(confidence) = record.confidence
        && confidence > 100
    {
        errors.push("confidence must be between 0 and 100".to_string());
    }
    errors
}

fn validate_optional_text(field: &str, value: Option<&str>, errors: &mut Vec<String>) {
    if let Some(value) = value {
        validate_non_empty_text(field, value, errors);
    }
}

fn validate_repeated_text(field: &str, values: &[String], errors: &mut Vec<String>) {
    for (index, value) in values.iter().enumerate() {
        let item = format!("{field}[{index}]");
        validate_non_empty_text(&item, value, errors);
    }
}

fn validate_required_repeated_text(field: &str, values: &[String], errors: &mut Vec<String>) {
    if values.is_empty() {
        errors.push(format!("{field} must include at least one value"));
    }
    validate_repeated_text(field, values, errors);
}

fn validate_non_empty_text(field: &str, value: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{field} must not be empty"));
        return;
    }
    if value.chars().any(char::is_control) {
        errors.push(format!("{field} must not contain control characters"));
    }
}

pub fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    configure_no_follow(&mut options);
    let mut file = options
        .open(&path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    serde_json::to_writer_pretty(&mut file, value)?;
    writeln!(file)?;
    Ok(())
}

pub fn append_jsonl<T: Serialize>(path: PathBuf, value: &T) -> Result<()> {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    configure_no_follow(&mut options);
    let mut file = options
        .open(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    serde_json::to_writer(&mut file, value)?;
    writeln!(file)?;
    Ok(())
}

#[cfg(unix)]
fn configure_no_follow(options: &mut OpenOptions) {
    options.custom_flags(libc::O_NOFOLLOW);
}

#[cfg(not(unix))]
fn configure_no_follow(_options: &mut OpenOptions) {}

fn write_markdown(path: PathBuf, content: &str) -> Result<()> {
    fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))
}

pub fn read_json<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    serde_json::from_reader(file).with_context(|| format!("failed to parse {}", path.display()))
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

pub fn render_command(command: &[String]) -> String {
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

impl FromStr for CapsuleStatus {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "active" => Ok(Self::Active),
            "blocked" => Ok(Self::Blocked),
            "ready_for_pr" => Ok(Self::ReadyForPr),
            "in_review" => Ok(Self::InReview),
            "merged" => Ok(Self::Merged),
            "closed" => Ok(Self::Closed),
            _ => Err(format!(
                "invalid capsule status {value:?}; expected active, blocked, ready_for_pr, in_review, merged, or closed"
            )),
        }
    }
}

impl std::fmt::Display for EvidenceKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            EvidenceKind::Command => "command",
            EvidenceKind::Subagent => "subagent",
            EvidenceKind::Review => "review",
            EvidenceKind::Ci => "ci",
            EvidenceKind::Decision => "decision",
            EvidenceKind::Research => "research",
            EvidenceKind::Manual => "manual",
            EvidenceKind::Output => "output",
        };
        formatter.write_str(value)
    }
}

impl FromStr for EvidenceKind {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "command" => Ok(Self::Command),
            "subagent" => Ok(Self::Subagent),
            "review" => Ok(Self::Review),
            "ci" => Ok(Self::Ci),
            "decision" => Ok(Self::Decision),
            "research" => Ok(Self::Research),
            "manual" => Ok(Self::Manual),
            "output" => Ok(Self::Output),
            _ => Err(format!(
                "invalid evidence kind {value:?}; expected command, subagent, review, ci, decision, research, manual, or output"
            )),
        }
    }
}

impl std::fmt::Display for PolicyProfile {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyProfile::CodexDev => formatter.write_str("codex_dev"),
            PolicyProfile::CodexDevTui => formatter.write_str("codex_dev_tui"),
            PolicyProfile::CodexResearch => formatter.write_str("codex_research"),
            PolicyProfile::Skills => formatter.write_str("skills"),
            PolicyProfile::BootstrapInstall => formatter.write_str("bootstrap_install"),
            PolicyProfile::Docs => formatter.write_str("docs"),
            PolicyProfile::Release => formatter.write_str("release"),
            PolicyProfile::FullLocal => formatter.write_str("full_local"),
        }
    }
}

impl FromStr for PolicyProfile {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "codex_dev" => Ok(Self::CodexDev),
            "codex_dev_tui" => Ok(Self::CodexDevTui),
            "codex_research" => Ok(Self::CodexResearch),
            "skills" => Ok(Self::Skills),
            "bootstrap_install" => Ok(Self::BootstrapInstall),
            "docs" => Ok(Self::Docs),
            "release" => Ok(Self::Release),
            "full_local" => Ok(Self::FullLocal),
            _ => Err(format!(
                "invalid policy profile {value:?}; expected codex_dev, codex_dev_tui, codex_research, skills, bootstrap_install, docs, release, or full_local"
            )),
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

impl std::fmt::Display for SubagentOutcomeStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            SubagentOutcomeStatus::Planned => "planned",
            SubagentOutcomeStatus::Running => "running",
            SubagentOutcomeStatus::Completed => "completed",
            SubagentOutcomeStatus::Failed => "failed",
            SubagentOutcomeStatus::TimedOut => "timed_out",
            SubagentOutcomeStatus::Closed => "closed",
            SubagentOutcomeStatus::Blocked => "blocked",
        };
        formatter.write_str(value)
    }
}

impl FromStr for SubagentOutcomeStatus {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "planned" => Ok(Self::Planned),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "timed_out" => Ok(Self::TimedOut),
            "closed" => Ok(Self::Closed),
            "blocked" => Ok(Self::Blocked),
            _ => Err(format!(
                "invalid subagent outcome status {value:?}; expected planned, running, completed, failed, timed_out, closed, or blocked"
            )),
        }
    }
}

impl std::fmt::Display for SubagentDisposition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            SubagentDisposition::Accepted => "accepted",
            SubagentDisposition::Rejected => "rejected",
            SubagentDisposition::Mixed => "mixed",
            SubagentDisposition::Informational => "informational",
            SubagentDisposition::Pending => "pending",
        };
        formatter.write_str(value)
    }
}

impl FromStr for SubagentDisposition {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "mixed" => Ok(Self::Mixed),
            "informational" => Ok(Self::Informational),
            "pending" => Ok(Self::Pending),
            _ => Err(format!(
                "invalid subagent disposition {value:?}; expected accepted, rejected, mixed, informational, or pending"
            )),
        }
    }
}

impl std::fmt::Display for SubagentSynthesisStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            SubagentSynthesisStatus::Completed => "completed",
            SubagentSynthesisStatus::Partial => "partial",
            SubagentSynthesisStatus::Blocked => "blocked",
        };
        formatter.write_str(value)
    }
}

impl FromStr for SubagentSynthesisStatus {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "completed" => Ok(Self::Completed),
            "partial" => Ok(Self::Partial),
            "blocked" => Ok(Self::Blocked),
            _ => Err(format!(
                "invalid subagent synthesis status {value:?}; expected completed, partial, or blocked"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
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

    fn merge_json_object(base: &mut Value, extra: Value) {
        let base = base.as_object_mut().expect("base json object");
        for (key, value) in extra.as_object().expect("extra json object") {
            base.insert(key.clone(), value.clone());
        }
    }

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
            policy_manifest: policy_manifest_fixture(created_at),
            force: false,
        }
    }

    fn policy_manifest_fixture(generated_at: DateTime<Utc>) -> PolicyManifest {
        PolicyManifest {
            schema: POLICY_GATES_SCHEMA.to_string(),
            profile: PolicyProfile::CodexDev,
            generated_at,
            gates: vec![PolicyGate {
                id: "test-gate".to_string(),
                name: "test gate".to_string(),
                command: vec!["fixture-command".to_string()],
                source: "test".to_string(),
                working_directory: ".".to_string(),
                required_tools: vec!["fixture-command".to_string()],
                required: true,
                network: false,
                secrets: false,
                failure_interpretation: "fixture failure".to_string(),
            }],
        }
    }

    fn pr_record_args(
        capsule: PathBuf,
        source: PathBuf,
        source_kind: PrRecordSourceKind,
    ) -> PrRecordArgs {
        PrRecordArgs {
            capsule,
            source,
            source_kind,
            repository: None,
            number: None,
            retrieved_at: None,
            source_command: None,
            command: Some("fixture-pr-recorder".to_string()),
        }
    }

    fn write_subspawn_plan_fixture(
        root: &Path,
        file_name: &str,
        task: &str,
        roles: &[&str],
    ) -> PathBuf {
        let prompts = roles
            .iter()
            .map(|role| {
                json!({
                    "role": role,
                    "prompt": format!("Task: {task}\nRole: {role}\nReturn format:\n- Status\n- Risks/blockers")
                })
            })
            .collect::<Vec<_>>();
        let roles = roles
            .iter()
            .map(|role| json!({ "name": role }))
            .collect::<Vec<_>>();
        let path = root.join(file_name);
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "task": task,
                "mode": "read-only",
                "scope": "fixture scope",
                "wait_policy": "strict",
                "rendezvous_required": true,
                "roles": roles,
                "prompts": prompts,
                "registry_issues": [],
                "duplicate_roles_ignored": {
                    "test_runner": [
                        "skills/subagent-creator/templates/agents/test_runner.toml",
                        "skills/subspawn/templates/agents/test_runner.toml"
                    ]
                },
                "synthesis_checklist": [
                    "Wait for every spawned subagent in the planned batch."
                ]
            }))
            .expect("plan json"),
        )
        .expect("write plan fixture");
        path
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
                "id",
                "name",
                "command",
                "source",
                "working_directory",
                "required_tools",
                "required",
                "network",
                "secrets",
                "failure_interpretation",
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
    fn task_index_lists_valid_and_invalid_task_entries() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let mut alpha_args = init_args(root.clone());
        alpha_args.id = Some("alpha-task".to_string());
        alpha_args.title = "Alpha task".to_string();
        let alpha = init_capsule(alpha_args).expect("init alpha");
        fs::create_dir_all(root.join("broken-task")).expect("broken task dir");

        let report = task_index(&root).expect("task index");

        assert_eq!(report.schema, TASK_INDEX_SCHEMA);
        assert_eq!(report.total, 2);
        assert_eq!(report.valid, 1);
        assert_eq!(report.invalid, 1);
        assert_eq!(report.tasks[0].path, alpha.path);
        assert!(report.tasks[0].valid);
        assert_eq!(
            report.tasks[0].capsule.as_ref().expect("status").title,
            "Alpha task"
        );
        assert!(!report.tasks[1].valid);
        assert!(
            report.tasks[1]
                .errors
                .iter()
                .any(|error| error.contains("capsule.json"))
        );
    }

    #[test]
    fn task_show_and_export_resolve_task_ids_from_root() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let mut args = init_args(root.clone());
        args.id = Some("export-task".to_string());
        args.title = "Export task".to_string();
        init_capsule(args).expect("init task");

        let show = task_show(&root, Path::new("export-task")).expect("task show");
        assert_eq!(show.schema, TASK_INDEX_SCHEMA);
        assert!(show.task.valid);
        assert_eq!(
            show.task.capsule.as_ref().expect("status").id,
            "export-task"
        );

        let export = task_export(&root, Path::new("export-task")).expect("task export");
        assert_eq!(export.schema, TASK_INDEX_SCHEMA);
        assert_eq!(export.capsule.title, "Export task");
        assert_eq!(export.evidence.len(), 1);
        assert_eq!(export.verification.schema, VERIFICATION_SCHEMA);
        assert!(export.markdown["plan.md"].contains("# Plan"));
        assert!(export.markdown.contains_key("retrospective.md"));
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

    #[cfg(unix)]
    #[test]
    fn validate_rejects_symlinked_contract_file() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let pr_path = capsule.join("pr.json");
        let outside_path = temp.path().join("outside-pr.json");
        fs::copy(&pr_path, &outside_path).expect("copy pr fixture");
        fs::remove_file(&pr_path).expect("remove pr");
        std::os::unix::fs::symlink(&outside_path, &pr_path).expect("symlink pr");

        let validation = validate_capsule(&capsule).expect("validate");

        assert!(!validation.valid);
        let joined = validation.errors.join("\n");
        assert!(
            joined.contains("refusing to validate symlinked capsule contract file"),
            "{joined}"
        );
        assert!(!joined.contains("invalid pr.json"), "{joined}");
    }

    #[cfg(unix)]
    #[test]
    fn validate_rejects_symlinked_markdown_file() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let plan_path = capsule.join("plan.md");
        let outside_path = temp.path().join("outside-plan.md");
        fs::write(&outside_path, "outside plan\n").expect("write outside plan");
        fs::remove_file(&plan_path).expect("remove plan");
        std::os::unix::fs::symlink(&outside_path, &plan_path).expect("symlink plan");

        let validation = validate_capsule(&capsule).expect("validate");

        assert!(!validation.valid);
        let joined = validation.errors.join("\n");
        assert!(
            joined.contains("refusing to validate symlinked capsule contract file"),
            "{joined}"
        );
        assert!(joined.contains("plan.md"), "{joined}");
    }

    #[cfg(unix)]
    #[test]
    fn task_show_rejects_task_id_under_symlinked_root() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let mut args = init_args(root.clone());
        args.id = Some("root-symlink-task".to_string());
        init_capsule(args).expect("init task");
        let root_link = temp.path().join("tasks-link");
        std::os::unix::fs::symlink(&root, &root_link).expect("symlink root");

        let error = task_show(&root_link, Path::new("root-symlink-task"))
            .expect_err("symlinked root rejected");

        assert!(format!("{error:#}").contains("symlinked task root"));
    }

    #[cfg(unix)]
    #[test]
    fn task_export_rejects_symlinked_markdown_file() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let mut args = init_args(root.clone());
        args.id = Some("markdown-symlink-task".to_string());
        let result = init_capsule(args).expect("init task");
        let plan_path = result.path.join("plan.md");
        let outside_path = temp.path().join("outside-plan.md");
        fs::write(&outside_path, "outside plan\n").expect("write outside plan");
        fs::remove_file(&plan_path).expect("remove plan");
        std::os::unix::fs::symlink(&outside_path, &plan_path).expect("symlink plan");

        let error = task_export(&root, Path::new("markdown-symlink-task"))
            .expect_err("symlinked markdown rejected");

        assert!(
            format!("{error:#}").contains("symlinked capsule contract file"),
            "{error:#}"
        );
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
                source_kind: PrRecordSourceKind::Normalized,
                repository: None,
                number: None,
                retrieved_at: None,
                source_command: None,
                command: Some("fixture-pr-recorder --source pr-snapshot.json".to_string()),
            },
            "2026-05-09T05:00:00Z".parse().unwrap(),
        )
        .expect("record pr");

        assert_eq!(result.pr.schema, PR_SCHEMA);
        assert_eq!(result.pr.state, "open");
        assert_eq!(result.pr.checks[0].status, "completed");
        assert_eq!(result.pr.checks[0].conclusion.as_deref(), Some("success"));
        assert_eq!(result.pr.review_threads.unresolved, 0);
        assert_eq!(result.pr.sources[0].kind, "normalized");
        assert_eq!(
            result.pr.sources[0].parser_version,
            PR_SOURCE_PARSER_VERSION
        );
        assert!(result.pr.review_threads.authoritative);

        let pr: PrEvidence = read_json(&capsule.join("pr.json")).expect("pr json");
        assert_eq!(pr.number, Some(25));

        let capsule_state: Capsule = read_json(&capsule.join("capsule.json")).expect("capsule");
        assert_eq!(capsule_state.pull_requests, vec![25]);

        let evidence = fs::read_to_string(capsule.join("evidence.jsonl")).expect("evidence");
        assert!(evidence.contains("PR snapshot recorded"));
        assert!(evidence.contains("fixture-pr-recorder --source pr-snapshot.json"));
    }

    #[test]
    fn pr_record_normalizes_gh_pr_view_open_draft_and_mergeable_cases() {
        let temp = tempdir().expect("tempdir");
        let cases = [
            ("open.json", false, "MERGEABLE", "APPROVED", "open"),
            (
                "draft.json",
                true,
                "CONFLICTING",
                "REVIEW_REQUIRED",
                "draft",
            ),
        ];

        for (file_name, is_draft, mergeable, review_decision, expected_state) in cases {
            let capsule = init_capsule(init_args(temp.path().join(file_name)))
                .expect("init capsule")
                .path;
            let source = temp.path().join(format!("source-{file_name}"));
            fs::write(
                &source,
                serde_json::to_string_pretty(&json!({
                    "number": 46,
                    "url": "https://github.com/BjornMelin/dev-skills/pull/46",
                    "state": "OPEN",
                        "isDraft": is_draft,
                        "mergeable": mergeable,
                        "mergeStateStatus": "CLEAN",
                        "reviewDecision": review_decision,
                        "headRefOid": "abc123",
                        "headRefName": "feature",
                        "baseRefName": "main",
                        "baseRefOid": "base123",
                        "statusCheckRollup": [{
                        "__typename": "CheckRun",
                        "name": "GitGuardian Security Checks",
                        "status": "COMPLETED",
                        "conclusion": "SUCCESS",
                        "detailsUrl": "https://example.test/check",
                        "completedAt": "2026-05-09T05:01:00Z"
                    }]
                }))
                .expect("fixture json"),
            )
            .expect("write fixture");

            let mut args = pr_record_args(capsule.clone(), source, PrRecordSourceKind::GhPrView);
            args.source_command = Some("gh pr view 46 --json ...".to_string());
            let result = record_pr_snapshot(args, "2026-05-09T05:00:00Z".parse().unwrap())
                .expect("record pr");

            assert_eq!(
                result.pr.repository.as_deref(),
                Some("BjornMelin/dev-skills")
            );
            assert_eq!(result.pr.number, Some(46));
            assert_eq!(result.pr.state, expected_state);
            let expected_mergeable = mergeable.to_ascii_lowercase();
            let expected_review_decision = review_decision.to_ascii_lowercase();
            assert_eq!(
                result.pr.mergeable.as_deref(),
                Some(expected_mergeable.as_str())
            );
            assert_eq!(
                result.pr.review_decision.as_deref(),
                Some(expected_review_decision.as_str())
            );
            assert_eq!(result.pr.head_sha.as_deref(), Some("abc123"));
            assert_eq!(result.pr.merge_state_status.as_deref(), Some("clean"));
            assert_eq!(result.pr.head_ref_name.as_deref(), Some("feature"));
            assert_eq!(result.pr.base_ref_name.as_deref(), Some("main"));
            assert_eq!(result.pr.base_ref_oid.as_deref(), Some("base123"));
            assert_eq!(result.pr.checks[0].conclusion.as_deref(), Some("success"));
            assert!(!result.pr.review_threads.authoritative);
            assert_eq!(result.pr.sources[0].kind, "gh-pr-view");
            assert_eq!(
                result.pr.sources[0].command.as_deref(),
                Some("gh pr view 46 --json ...")
            );
        }
    }

    #[test]
    fn pr_record_normalizes_gh_pr_checks_failing_statuses() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let source = temp.path().join("gh-pr-checks.json");
        fs::write(
            &source,
            serde_json::to_string_pretty(&json!([
                {
                    "bucket": "fail",
                    "completedAt": "2026-05-09T05:02:00Z",
                    "link": "https://example.test/check/fail",
                    "name": "lint",
                    "state": "FAILURE",
                    "workflow": "ci"
                },
                {
                    "bucket": "pending",
                    "completedAt": "0001-01-01T00:00:00Z",
                    "link": "",
                    "name": "test",
                    "startedAt": "0001-01-01T00:00:00Z",
                    "state": "PENDING",
                    "workflow": "ci"
                }
            ]))
            .expect("fixture json"),
        )
        .expect("write fixture");

        let mut args = pr_record_args(capsule, source, PrRecordSourceKind::GhPrChecks);
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, "2026-05-09T05:00:00Z".parse().unwrap())
            .expect("record checks");

        assert_eq!(result.pr.state, "unknown");
        assert_eq!(result.pr.checks.len(), 2);
        assert_eq!(result.pr.checks[0].status, "completed");
        assert_eq!(result.pr.checks[0].conclusion.as_deref(), Some("failure"));
        assert_eq!(result.pr.checks[1].status, "pending");
        assert!(!result.pr.review_threads.authoritative);
        assert_eq!(
            result.pr.checks[1].checked_at,
            "2026-05-09T05:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
    }

    #[test]
    fn pr_record_normalizes_unresolved_and_stale_review_sources() {
        let temp = tempdir().expect("tempdir");
        let checked_at = "2026-05-09T05:00:00Z".parse().unwrap();

        let reviews_capsule = init_capsule(init_args(temp.path().join("reviews")))
            .expect("init capsule")
            .path;
        let reviews_source = temp.path().join("gh-reviews.json");
        fs::write(
            &reviews_source,
            serde_json::to_string_pretty(&json!([
                { "id": 1, "state": "COMMENTED", "submitted_at": "2026-05-09T04:00:00Z" },
                { "id": 2, "state": "CHANGES_REQUESTED", "submitted_at": "2026-05-09T05:00:00Z" }
            ]))
            .expect("fixture json"),
        )
        .expect("write fixture");
        let mut args = pr_record_args(
            reviews_capsule,
            reviews_source,
            PrRecordSourceKind::GhReviews,
        );
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, checked_at).expect("record reviews");
        assert_eq!(
            result.pr.review_decision.as_deref(),
            Some("changes_requested")
        );
        assert!(!result.pr.review_threads.authoritative);

        let threads_capsule = init_capsule(init_args(temp.path().join("threads")))
            .expect("init capsule")
            .path;
        let threads_source = temp.path().join("gh-review-threads.json");
        fs::write(
            &threads_source,
            serde_json::to_string_pretty(&json!({
                "data": {
                    "repository": {
                        "pullRequest": {
                            "reviewThreads": {
                                "nodes": [
                                    { "id": "resolved", "isResolved": true, "isOutdated": false },
                                    { "id": "current", "isResolved": false, "isOutdated": false },
                                    { "id": "stale", "isResolved": false, "isOutdated": true }
                                ],
                                "pageInfo": { "hasNextPage": false }
                            }
                        }
                    }
                }
            }))
            .expect("fixture json"),
        )
        .expect("write fixture");
        let mut args = pr_record_args(
            threads_capsule,
            threads_source,
            PrRecordSourceKind::GhReviewThreads,
        );
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, checked_at).expect("record threads");
        assert_eq!(result.pr.review_threads.total, 3);
        assert_eq!(result.pr.review_threads.resolved, 1);
        assert_eq!(result.pr.review_threads.unresolved, 1);
        assert_eq!(result.pr.review_threads.outdated, 1);
        assert!(result.pr.review_threads.authoritative);

        let comments_capsule = init_capsule(init_args(temp.path().join("comments")))
            .expect("init capsule")
            .path;
        let comments_source = temp.path().join("gh-review-comments.json");
        fs::write(
            &comments_source,
            serde_json::to_string_pretty(&json!([
                { "id": 1, "position": 4, "original_position": 4 },
                { "id": 2, "in_reply_to_id": 1, "position": null, "original_position": 8 },
                { "id": 3, "position": null, "original_position": 12 }
            ]))
            .expect("fixture json"),
        )
        .expect("write fixture");
        let mut args = pr_record_args(
            comments_capsule,
            comments_source,
            PrRecordSourceKind::GhReviewComments,
        );
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, checked_at).expect("record comments");
        assert_eq!(result.pr.review_threads.total, 2);
        assert_eq!(result.pr.review_threads.unresolved, 0);
        assert_eq!(result.pr.review_threads.outdated, 1);
        assert!(!result.pr.review_threads.authoritative);
    }

    #[test]
    fn gh_reviews_collapse_latest_state_per_reviewer() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("reviews")))
            .expect("init capsule")
            .path;
        let source = temp.path().join("gh-reviews.json");
        fs::write(
            &source,
            serde_json::to_string_pretty(&json!([
                {
                    "id": 1,
                    "user": { "login": "alice" },
                    "state": "CHANGES_REQUESTED",
                    "submitted_at": "2026-05-09T04:00:00Z"
                },
                {
                    "id": 2,
                    "user": { "login": "alice" },
                    "state": "APPROVED",
                    "submitted_at": "2026-05-09T05:00:00Z"
                },
                {
                    "id": 3,
                    "user": { "login": "alice" },
                    "state": "COMMENTED",
                    "submitted_at": "2026-05-09T06:00:00Z"
                },
                {
                    "id": 4,
                    "user": { "login": "bob" },
                    "state": "COMMENTED",
                    "submitted_at": "2026-05-09T07:00:00Z"
                }
            ]))
            .expect("fixture json"),
        )
        .expect("write fixture");

        let mut args = pr_record_args(capsule, source, PrRecordSourceKind::GhReviews);
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, "2026-05-09T06:05:00Z".parse().unwrap())
            .expect("record reviews");
        assert_eq!(result.pr.review_decision.as_deref(), Some("approved"));
    }

    #[test]
    fn gh_reviews_keep_change_request_across_later_comment() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("reviews")))
            .expect("init capsule")
            .path;
        let source = temp.path().join("gh-reviews.json");
        fs::write(
            &source,
            serde_json::to_string_pretty(&json!([
                {
                    "id": 1,
                    "user": { "login": "alice" },
                    "state": "CHANGES_REQUESTED",
                    "submitted_at": "2026-05-09T04:00:00Z"
                },
                {
                    "id": 2,
                    "user": { "login": "alice" },
                    "state": "COMMENTED",
                    "submitted_at": "2026-05-09T05:00:00Z"
                }
            ]))
            .expect("fixture json"),
        )
        .expect("write fixture");

        let mut args = pr_record_args(capsule, source, PrRecordSourceKind::GhReviews);
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, "2026-05-09T06:05:00Z".parse().unwrap())
            .expect("record reviews");

        assert_eq!(
            result.pr.review_decision.as_deref(),
            Some("changes_requested")
        );
    }

    #[test]
    fn gh_review_threads_require_complete_page_for_authority() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("threads")))
            .expect("init capsule")
            .path;
        let source = temp.path().join("gh-review-threads.json");
        fs::write(
            &source,
            serde_json::to_string_pretty(&json!({
                "reviewThreads": {
                    "nodes": [
                        { "id": "current", "isResolved": false, "isOutdated": false }
                    ],
                    "pageInfo": { "hasNextPage": true }
                }
            }))
            .expect("fixture json"),
        )
        .expect("write fixture");

        let mut args = pr_record_args(capsule, source, PrRecordSourceKind::GhReviewThreads);
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, "2026-05-09T06:05:00Z".parse().unwrap())
            .expect("record threads");
        assert_eq!(result.pr.review_threads.unresolved, 1);
        assert!(!result.pr.review_threads.authoritative);
    }

    #[test]
    fn gh_review_threads_support_slurped_paginated_graphql_pages() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("threads")))
            .expect("init capsule")
            .path;
        let source = temp.path().join("gh-review-threads.json");
        fs::write(
            &source,
            serde_json::to_string_pretty(&json!([
                {
                    "data": {
                        "repository": {
                            "pullRequest": {
                                "reviewThreads": {
                                    "nodes": [
                                        { "id": "resolved", "isResolved": true, "isOutdated": false }
                                    ],
                                    "pageInfo": { "hasNextPage": true, "endCursor": "cursor-1" }
                                }
                            }
                        }
                    }
                },
                {
                    "data": {
                        "repository": {
                            "pullRequest": {
                                "reviewThreads": {
                                    "nodes": [
                                        { "id": "current", "isResolved": false, "isOutdated": false },
                                        { "id": "stale", "isResolved": false, "isOutdated": true }
                                    ],
                                    "pageInfo": { "hasNextPage": false, "endCursor": null }
                                }
                            }
                        }
                    }
                }
            ]))
            .expect("fixture json"),
        )
        .expect("write fixture");

        let mut args = pr_record_args(capsule, source, PrRecordSourceKind::GhReviewThreads);
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, "2026-05-09T06:05:00Z".parse().unwrap())
            .expect("record paginated threads");

        assert_eq!(result.pr.review_threads.total, 3);
        assert_eq!(result.pr.review_threads.resolved, 1);
        assert_eq!(result.pr.review_threads.unresolved, 1);
        assert_eq!(result.pr.review_threads.outdated, 1);
        assert!(result.pr.review_threads.authoritative);
    }

    #[test]
    fn pr_agent_recommendations_block_on_failures_and_reviews() {
        let checked_at = "2026-05-09T06:05:00Z".parse().unwrap();
        let pr = PrEvidence {
            schema: PR_SCHEMA.to_string(),
            repository: Some("BjornMelin/dev-skills".to_string()),
            number: Some(46),
            url: Some("https://github.com/BjornMelin/dev-skills/pull/46".to_string()),
            state: "open".to_string(),
            is_draft: Some(false),
            mergeable: Some("mergeable".to_string()),
            merge_state_status: Some("unstable".to_string()),
            review_decision: Some("changes_requested".to_string()),
            head_sha: Some("abc123".to_string()),
            head_ref_name: Some("feature".to_string()),
            base_ref_name: Some("main".to_string()),
            base_ref_oid: Some("base123".to_string()),
            checks: vec![CheckRecord {
                name: "ci".to_string(),
                status: "completed".to_string(),
                conclusion: Some("failure".to_string()),
                url: None,
                checked_at,
            }],
            review_threads: ReviewThreadSummary {
                unresolved: 2,
                total: 2,
                resolved: 0,
                outdated: 0,
                authoritative: true,
                last_checked_at: checked_at,
            },
            sources: Vec::new(),
        };
        let diagnostics = vec![PrAgentDiagnostic {
            source: "gh-pr-view".to_string(),
            severity: PrAgentSeverity::Error,
            message: "missing permission".to_string(),
            command: None,
            exit_code: Some(1),
            at: checked_at,
        }];

        let actions = recommend_pr_agent_actions(&pr, &diagnostics);
        let action_ids = actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>();
        assert!(action_ids.contains(&"resolve_state_collection_errors"));
        assert!(action_ids.contains(&"diagnose_failed_checks"));
        assert!(action_ids.contains(&"process_review_threads"));
        assert!(action_ids.contains(&"process_requested_changes"));
    }

    #[test]
    fn pr_agent_recommendations_identify_clean_merge_ready_state() {
        let checked_at = "2026-05-09T06:05:00Z".parse().unwrap();
        let pr = PrEvidence {
            schema: PR_SCHEMA.to_string(),
            repository: Some("BjornMelin/dev-skills".to_string()),
            number: Some(46),
            url: Some("https://github.com/BjornMelin/dev-skills/pull/46".to_string()),
            state: "open".to_string(),
            is_draft: Some(false),
            mergeable: Some("mergeable".to_string()),
            merge_state_status: Some("clean".to_string()),
            review_decision: Some("approved".to_string()),
            head_sha: Some("abc123".to_string()),
            head_ref_name: Some("feature".to_string()),
            base_ref_name: Some("main".to_string()),
            base_ref_oid: Some("base123".to_string()),
            checks: vec![CheckRecord {
                name: "ci".to_string(),
                status: "completed".to_string(),
                conclusion: Some("success".to_string()),
                url: None,
                checked_at,
            }],
            review_threads: ReviewThreadSummary {
                unresolved: 0,
                total: 1,
                resolved: 1,
                outdated: 0,
                authoritative: true,
                last_checked_at: checked_at,
            },
            sources: Vec::new(),
        };

        let actions = recommend_pr_agent_actions(&pr, &[]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "merge_when_policy_allows");
        assert_eq!(actions[0].priority, PrAgentActionPriority::Ready);
    }

    #[test]
    fn pr_agent_recommendations_do_not_mark_review_required_or_unknown_merge_ready() {
        let checked_at = "2026-05-09T06:05:00Z".parse().unwrap();
        let mut pr = PrEvidence {
            schema: PR_SCHEMA.to_string(),
            repository: Some("BjornMelin/dev-skills".to_string()),
            number: Some(46),
            url: Some("https://github.com/BjornMelin/dev-skills/pull/46".to_string()),
            state: "open".to_string(),
            is_draft: Some(false),
            mergeable: Some("unknown".to_string()),
            merge_state_status: Some("unknown".to_string()),
            review_decision: Some("review_required".to_string()),
            head_sha: Some("abc123".to_string()),
            head_ref_name: Some("feature".to_string()),
            base_ref_name: Some("main".to_string()),
            base_ref_oid: Some("base123".to_string()),
            checks: vec![CheckRecord {
                name: "ci".to_string(),
                status: "completed".to_string(),
                conclusion: Some("success".to_string()),
                url: None,
                checked_at,
            }],
            review_threads: ReviewThreadSummary {
                unresolved: 0,
                total: 1,
                resolved: 1,
                outdated: 0,
                authoritative: true,
                last_checked_at: checked_at,
            },
            sources: Vec::new(),
        };

        let actions = recommend_pr_agent_actions(&pr, &[]);
        let action_ids = actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>();
        assert!(action_ids.contains(&"wait_for_required_review"));
        assert!(action_ids.contains(&"wait_for_mergeability"));
        assert!(!action_ids.contains(&"merge_when_policy_allows"));

        pr.review_decision = Some("approved".to_string());
        pr.mergeable = None;
        let actions = recommend_pr_agent_actions(&pr, &[]);
        let action_ids = actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>();
        assert!(action_ids.contains(&"wait_for_mergeability"));
        assert!(!action_ids.contains(&"merge_when_policy_allows"));

        pr.review_decision = None;
        pr.mergeable = Some("mergeable".to_string());
        let actions = recommend_pr_agent_actions(&pr, &[]);
        let action_ids = actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>();
        assert!(action_ids.contains(&"refresh_review_decision"));
        assert!(!action_ids.contains(&"merge_when_policy_allows"));
    }

    #[test]
    fn pr_agent_recommendations_require_explicit_check_conclusions() {
        let checked_at = "2026-05-09T06:05:00Z".parse().unwrap();
        let pr = PrEvidence {
            schema: PR_SCHEMA.to_string(),
            repository: Some("BjornMelin/dev-skills".to_string()),
            number: Some(46),
            url: Some("https://github.com/BjornMelin/dev-skills/pull/46".to_string()),
            state: "open".to_string(),
            is_draft: Some(false),
            mergeable: Some("mergeable".to_string()),
            merge_state_status: Some("clean".to_string()),
            review_decision: Some("approved".to_string()),
            head_sha: Some("abc123".to_string()),
            head_ref_name: Some("feature".to_string()),
            base_ref_name: Some("main".to_string()),
            base_ref_oid: Some("base123".to_string()),
            checks: vec![CheckRecord {
                name: "ci".to_string(),
                status: "completed".to_string(),
                conclusion: None,
                url: None,
                checked_at,
            }],
            review_threads: ReviewThreadSummary {
                unresolved: 0,
                total: 1,
                resolved: 1,
                outdated: 0,
                authoritative: true,
                last_checked_at: checked_at,
            },
            sources: Vec::new(),
        };

        let actions = recommend_pr_agent_actions(&pr, &[]);
        let action_ids = actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>();
        assert!(action_ids.contains(&"inspect_check_outcomes"));
        assert!(!action_ids.contains(&"merge_when_policy_allows"));
    }

    #[test]
    fn pr_record_normalizes_review_pack_zero_thread_output() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let source = temp.path().join("review-pack-remaining.json");
        fs::write(
            &source,
            serde_json::to_string_pretty(&json!({
                "review_threads": { "unresolved": 0 }
            }))
            .expect("fixture json"),
        )
        .expect("write fixture");

        let mut args = pr_record_args(capsule, source, PrRecordSourceKind::ReviewPackRemaining);
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        args.retrieved_at = Some("2026-05-09T04:59:00Z".parse().unwrap());
        let result = record_pr_snapshot(args, "2026-05-09T05:00:00Z".parse().unwrap())
            .expect("record remaining");

        assert_eq!(result.pr.review_threads.unresolved, 0);
        assert!(result.pr.review_threads.authoritative);
        assert_eq!(
            result.pr.sources[0].retrieved_at,
            "2026-05-09T04:59:00Z".parse::<DateTime<Utc>>().unwrap()
        );
    }

    #[test]
    fn pr_record_rejects_non_normalized_sources_without_identity() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let source = temp.path().join("gh-pr-checks.json");
        fs::write(
            &source,
            serde_json::to_string_pretty(&json!([
                { "bucket": "pass", "name": "lint", "state": "SUCCESS" }
            ]))
            .expect("fixture json"),
        )
        .expect("write fixture");

        let error = record_pr_snapshot(
            pr_record_args(capsule, source, PrRecordSourceKind::GhPrChecks),
            "2026-05-09T05:00:00Z".parse().unwrap(),
        )
        .expect_err("identity required");

        assert!(error.to_string().contains("requires explicit PR identity"));
    }

    #[test]
    fn pr_record_rejects_explicit_identity_that_conflicts_with_source_url() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let source = temp.path().join("gh-pr-view.json");
        fs::write(
            &source,
            serde_json::to_string_pretty(&json!({
                "number": 46,
                "url": "https://github.com/BjornMelin/dev-skills/pull/46",
                "state": "OPEN"
            }))
            .expect("fixture json"),
        )
        .expect("write fixture");

        let mut args = pr_record_args(
            capsule.clone(),
            source.clone(),
            PrRecordSourceKind::GhPrView,
        );
        args.repository = Some("Other/repo".to_string());
        args.number = Some(46);
        let error = record_pr_snapshot(args, "2026-05-09T05:00:00Z".parse().unwrap())
            .expect_err("repository mismatch rejected");
        assert!(error.to_string().contains("conflicting PR repository"));

        let mut args = pr_record_args(capsule, source, PrRecordSourceKind::GhPrView);
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(47);
        let error = record_pr_snapshot(args, "2026-05-09T05:00:00Z".parse().unwrap())
            .expect_err("number mismatch rejected");
        assert!(error.to_string().contains("conflicting PR number"));
    }

    #[test]
    fn pr_record_merges_provider_sources_without_dropping_prior_evidence() {
        let temp = tempdir().expect("tempdir");
        let checked_at = "2026-05-09T05:00:00Z".parse().unwrap();
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;

        let pr_view_source = temp.path().join("gh-pr-view.json");
        fs::write(
            &pr_view_source,
            serde_json::to_string_pretty(&json!({
                "number": 46,
                "url": "https://github.com/BjornMelin/dev-skills/pull/46",
                "state": "OPEN",
                    "isDraft": false,
                    "mergeable": "MERGEABLE",
                    "mergeStateStatus": "CLEAN",
                    "reviewDecision": "APPROVED",
                    "headRefOid": "abc123",
                    "headRefName": "feature",
                    "baseRefName": "main",
                    "baseRefOid": "base123",
                    "statusCheckRollup": [{
                    "name": "lint",
                    "status": "COMPLETED",
                    "conclusion": "SUCCESS"
                }]
            }))
            .expect("fixture json"),
        )
        .expect("write fixture");
        let mut args = pr_record_args(
            capsule.clone(),
            pr_view_source,
            PrRecordSourceKind::GhPrView,
        );
        args.source_command = Some("gh pr view 46 --json ...".to_string());
        record_pr_snapshot(args, checked_at).expect("record pr view");

        let thread_source = temp.path().join("gh-review-threads.json");
        fs::write(
            &thread_source,
            serde_json::to_string_pretty(&json!({
                "data": {
                    "repository": {
                        "pullRequest": {
                            "reviewThreads": {
                                "nodes": [
                                    { "id": "resolved", "isResolved": true, "isOutdated": false },
                                    { "id": "current", "isResolved": false, "isOutdated": false },
                                    { "id": "stale", "isResolved": false, "isOutdated": true }
                                ],
                                "pageInfo": { "hasNextPage": false }
                            }
                        }
                    }
                }
            }))
            .expect("fixture json"),
        )
        .expect("write fixture");
        let mut args = pr_record_args(
            capsule.clone(),
            thread_source,
            PrRecordSourceKind::GhReviewThreads,
        );
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        record_pr_snapshot(args, checked_at).expect("record review threads");

        let checks_source = temp.path().join("gh-pr-checks.json");
        fs::write(
            &checks_source,
            serde_json::to_string_pretty(&json!([
                { "bucket": "fail", "name": "lint", "state": "FAILURE" }
            ]))
            .expect("fixture json"),
        )
        .expect("write fixture");
        let mut args = pr_record_args(
            capsule.clone(),
            checks_source,
            PrRecordSourceKind::GhPrChecks,
        );
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        record_pr_snapshot(args, checked_at).expect("record checks");

        let remaining_source = temp.path().join("review-pack-remaining.txt");
        fs::write(&remaining_source, "0 unresolved threads\n").expect("write fixture");
        let mut args = pr_record_args(
            capsule.clone(),
            remaining_source,
            PrRecordSourceKind::ReviewPackRemaining,
        );
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, checked_at).expect("record remaining");

        assert_eq!(
            result.pr.repository.as_deref(),
            Some("BjornMelin/dev-skills")
        );
        assert_eq!(result.pr.number, Some(46));
        assert_eq!(result.pr.state, "open");
        assert_eq!(result.pr.mergeable.as_deref(), Some("mergeable"));
        assert_eq!(result.pr.merge_state_status.as_deref(), Some("clean"));
        assert_eq!(result.pr.review_decision.as_deref(), Some("approved"));
        assert_eq!(result.pr.head_sha.as_deref(), Some("abc123"));
        assert_eq!(result.pr.head_ref_name.as_deref(), Some("feature"));
        assert_eq!(result.pr.base_ref_name.as_deref(), Some("main"));
        assert_eq!(result.pr.base_ref_oid.as_deref(), Some("base123"));
        assert_eq!(result.pr.checks.len(), 1);
        assert_eq!(result.pr.checks[0].name, "lint");
        assert_eq!(result.pr.checks[0].conclusion.as_deref(), Some("failure"));
        assert_eq!(result.pr.review_threads.unresolved, 0);
        assert_eq!(result.pr.review_threads.resolved, 1);
        assert_eq!(result.pr.review_threads.outdated, 1);
        assert_eq!(result.pr.review_threads.total, 3);
        assert!(result.pr.review_threads.authoritative);
        assert_eq!(
            result
                .pr
                .sources
                .iter()
                .map(|source| source.kind.as_str())
                .collect::<Vec<_>>(),
            vec![
                "gh-pr-view",
                "gh-review-threads",
                "gh-pr-checks",
                "review-pack-remaining"
            ]
        );
    }

    #[test]
    fn pr_record_merge_precedence_keeps_stronger_provider_evidence() {
        let temp = tempdir().expect("tempdir");
        let checked_at = "2026-05-09T05:00:00Z".parse().unwrap();
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;

        let pr_view_source = temp.path().join("gh-pr-view.json");
        fs::write(
            &pr_view_source,
            serde_json::to_string_pretty(&json!({
                "number": 46,
                "url": "https://github.com/BjornMelin/dev-skills/pull/46",
                    "state": "OPEN",
                    "isDraft": false,
                    "mergeable": "MERGEABLE",
                    "mergeStateStatus": "CLEAN",
                    "reviewDecision": "APPROVED",
                    "headRefOid": "abc123",
                    "headRefName": "feature",
                    "baseRefName": "main",
                    "baseRefOid": "base123",
                    "statusCheckRollup": [{
                    "name": "lint",
                    "status": "COMPLETED",
                    "conclusion": "FAILURE"
                }]
            }))
            .expect("fixture json"),
        )
        .expect("write fixture");
        record_pr_snapshot(
            pr_record_args(
                capsule.clone(),
                pr_view_source,
                PrRecordSourceKind::GhPrView,
            ),
            checked_at,
        )
        .expect("record pr view");

        let reviews_source = temp.path().join("gh-reviews.json");
        fs::write(
            &reviews_source,
            serde_json::to_string_pretty(&json!([
                { "id": 1, "state": "COMMENTED", "submitted_at": "2026-05-09T06:00:00Z" }
            ]))
            .expect("fixture json"),
        )
        .expect("write fixture");
        let mut args = pr_record_args(
            capsule.clone(),
            reviews_source,
            PrRecordSourceKind::GhReviews,
        );
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, checked_at).expect("record reviews");
        assert_eq!(result.pr.review_decision.as_deref(), Some("approved"));

        let thread_source = temp.path().join("gh-review-threads.json");
        fs::write(
            &thread_source,
            serde_json::to_string_pretty(&json!({
                "reviewThreads": {
                    "nodes": [
                        { "id": "stale", "isResolved": false, "isOutdated": true }
                    ],
                    "pageInfo": { "hasNextPage": false }
                }
            }))
            .expect("fixture json"),
        )
        .expect("write fixture");
        let mut args = pr_record_args(
            capsule.clone(),
            thread_source,
            PrRecordSourceKind::GhReviewThreads,
        );
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        record_pr_snapshot(args, checked_at).expect("record threads");

        let comments_source = temp.path().join("gh-review-comments.json");
        fs::write(
            &comments_source,
            serde_json::to_string_pretty(&json!([
                { "id": 2, "position": 4, "original_position": 4 }
            ]))
            .expect("fixture json"),
        )
        .expect("write fixture");
        let mut args = pr_record_args(
            capsule.clone(),
            comments_source,
            PrRecordSourceKind::GhReviewComments,
        );
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, checked_at).expect("record comments");
        assert_eq!(result.pr.review_threads.outdated, 1);
        assert!(result.pr.review_threads.authoritative);

        let checks_source = temp.path().join("empty-gh-pr-checks.json");
        fs::write(&checks_source, "[]").expect("write fixture");
        let mut args = pr_record_args(capsule, checks_source, PrRecordSourceKind::GhPrChecks);
        args.repository = Some("BjornMelin/dev-skills".to_string());
        args.number = Some(46);
        let result = record_pr_snapshot(args, checked_at).expect("record empty checks");
        assert!(result.pr.checks.is_empty());
    }

    #[test]
    fn review_pack_remaining_text_parser_uses_review_count_not_exit_code() {
        assert_eq!(
            parse_review_pack_remaining("exit 0; unresolved_threads: 5").expect("parse count"),
            5
        );
        assert_eq!(
            parse_review_pack_remaining("0 unresolved threads").expect("parse prefix count"),
            0
        );
        assert!(
            parse_review_pack_remaining("exit 0; no review count").is_err(),
            "wrapper status without a review count must be rejected"
        );
    }

    #[test]
    fn pr_record_keeps_capsule_updated_at_monotonic() {
        let temp = tempdir().expect("tempdir");
        let mut args = init_args(temp.path().join("tasks"));
        args.created_at = "2026-05-09T10:00:00Z".parse().unwrap();
        let capsule = init_capsule(args).expect("init capsule").path;
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

        record_pr_snapshot(
            PrRecordArgs {
                capsule: capsule.clone(),
                source,
                source_kind: PrRecordSourceKind::Normalized,
                repository: None,
                number: None,
                retrieved_at: None,
                source_command: None,
                command: Some("fixture-pr-recorder".to_string()),
            },
            "2026-05-09T09:00:00Z".parse().unwrap(),
        )
        .expect("record backfilled pr");

        let status = capsule_status(&capsule).expect("status");
        assert_eq!(
            status.updated_at,
            "2026-05-09T10:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn pr_record_rejects_symlinked_contract_file_before_writing() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let pr_path = capsule.join("pr.json");
        let outside_path = temp.path().join("outside-pr.json");
        fs::write(&outside_path, "not json").expect("write invalid outside pr");
        fs::remove_file(&pr_path).expect("remove pr");
        std::os::unix::fs::symlink(&outside_path, &pr_path).expect("symlink pr");
        let outside_before = fs::read_to_string(&outside_path).expect("outside before");
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
                source_kind: PrRecordSourceKind::Normalized,
                repository: None,
                number: None,
                retrieved_at: None,
                source_command: None,
                command: Some("fixture-pr-recorder".to_string()),
            },
            "2026-05-09T09:00:00Z".parse().unwrap(),
        )
        .expect_err("symlinked pr rejected");

        assert!(
            error
                .to_string()
                .contains("symlinked capsule contract file")
        );
        assert_eq!(
            fs::read_to_string(&outside_path).expect("outside after"),
            outside_before
        );
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
    fn validate_accepts_legacy_policy_manifest_without_gate_metadata() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let mut policy: Value = read_json(&capsule.join("policy.json")).expect("policy json");
        let gate = policy["gates"][0].as_object_mut().expect("gate object");
        gate.remove("working_directory");
        gate.remove("required_tools");
        gate.remove("failure_interpretation");
        write_json(capsule.join("policy.json"), &policy).expect("write legacy policy");

        let validation = validate_capsule(&capsule).expect("validate");

        assert!(validation.valid, "{:?}", validation.errors);
    }

    #[test]
    fn validate_rejects_invalid_policy_manifest_semantics() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        fs::write(
            capsule.join("policy.json"),
            serde_json::to_string_pretty(&json!({
                "schema": POLICY_GATES_SCHEMA,
                "profile": "codex_dev",
                "generated_at": "2026-05-09T04:00:00Z",
                "gates": [{
                    "id": "bad gate",
                    "name": "",
                    "command": [],
                    "source": "",
                    "working_directory": "../outside",
                    "required_tools": [],
                    "required": true,
                    "network": false,
                    "secrets": false,
                    "failure_interpretation": ""
                }]
            }))
            .expect("policy json"),
        )
        .expect("write policy");

        let validation = validate_capsule(&capsule).expect("validate");

        assert!(!validation.valid);
        let joined = validation.errors.join("\n");
        assert!(joined.contains("policy.gates[0].id"), "{joined}");
        assert!(joined.contains("policy.gates[0].command"), "{joined}");
        assert!(
            joined.contains("policy.gates[0].required_tools"),
            "{joined}"
        );
        assert!(
            joined.contains("working_directory must stay within the repository"),
            "{joined}"
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
                source_kind: PrRecordSourceKind::Normalized,
                repository: None,
                number: None,
                retrieved_at: None,
                source_command: None,
                command: Some("fixture-pr-recorder".to_string()),
            },
            "2026-05-09T05:00:00Z".parse().unwrap(),
        )
        .expect_err("missing pr.json rejected");

        assert!(error.to_string().contains("missing required file: pr.json"));
        assert!(!capsule.join("pr.json").exists());
    }

    #[test]
    fn record_subagent_plan_accepts_common_batch_fixtures() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        let fixtures = [
            (
                "research",
                "research batch",
                vec!["openai_docs_researcher", "github_researcher"],
            ),
            (
                "review",
                "review batch",
                vec!["reviewer", "false_positive_validator"],
            ),
            (
                "implementation",
                "implementation batch",
                vec!["repo_explorer", "implementation_worker"],
            ),
            ("validation", "validation batch", vec!["test_runner"]),
        ];

        for (index, (batch_id, task, roles)) in fixtures.iter().enumerate() {
            let source = write_subspawn_plan_fixture(
                temp.path(),
                &format!("{batch_id}-plan.json"),
                task,
                roles,
            );
            let result = record_subagent_plan(RecordSubagentPlanArgs {
                capsule: capsule.clone(),
                batch_id: (*batch_id).to_string(),
                source,
                command: Some(format!("subspawn_plan.py plan --fixture {batch_id}")),
                recorded_at: format!("2026-05-09T05:0{index}:00Z").parse().unwrap(),
            })
            .expect("record subagent plan");

            assert_eq!(result.batch.id, *batch_id);
            assert_eq!(result.batch.status, "planned");
            assert_eq!(result.batch.agents.len(), roles.len());
            assert!(result.batch.prompts[0].prompt_hash.starts_with("sha256:"));
            assert_eq!(result.batch.prompts[0].prompt_hash.len(), 71);
        }

        let subagents: Subagents = read_json(&capsule.join("subagents.json")).expect("subagents");
        assert_eq!(subagents.batches.len(), fixtures.len());
        assert_eq!(
            subagents.batches[0].duplicate_roles_ignored["test_runner"].len(),
            2
        );
    }

    #[test]
    fn record_subagent_outcome_and_synthesis_append_evidence() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        let source = write_subspawn_plan_fixture(
            temp.path(),
            "review-plan.json",
            "review batch",
            &["reviewer"],
        );
        record_subagent_plan(RecordSubagentPlanArgs {
            capsule: capsule.clone(),
            batch_id: "review".to_string(),
            source,
            command: None,
            recorded_at: "2026-05-09T05:00:00Z".parse().unwrap(),
        })
        .expect("record plan");

        let outcome = record_subagent_outcome(RecordSubagentOutcomeArgs {
            capsule: capsule.clone(),
            batch_id: "review".to_string(),
            role: "reviewer".to_string(),
            status: SubagentOutcomeStatus::Completed,
            summary: "no blocking findings".to_string(),
            disposition: SubagentDisposition::Accepted,
            human_verified: true,
            source_ids: vec!["reviewer:1".to_string()],
            artifacts: vec!["review-notes.md".to_string()],
            recorded_at: "2026-05-09T05:10:00Z".parse().unwrap(),
        })
        .expect("record outcome");

        assert_eq!(outcome.agent.status, "completed");
        assert_eq!(outcome.agent.disposition.as_deref(), Some("accepted"));
        assert!(outcome.agent.human_verified);
        assert_eq!(outcome.batch.status, "completed");

        let synthesis = record_subagent_synthesis(RecordSubagentSynthesisArgs {
            capsule: capsule.clone(),
            batch_id: "review".to_string(),
            status: SubagentSynthesisStatus::Completed,
            summary: "review batch clean".to_string(),
            human_verified: true,
            source_ids: vec!["synthesis:review".to_string()],
            artifacts: vec!["review-summary.md".to_string()],
            recorded_at: "2026-05-09T05:20:00Z".parse().unwrap(),
        })
        .expect("record synthesis");

        assert_eq!(synthesis.synthesis.status, "completed");
        assert_eq!(synthesis.batch.status, "completed");
        assert_eq!(synthesis.evidence.total, 4);
        let evidence = fs::read_to_string(capsule.join("evidence.jsonl")).expect("evidence");
        assert!(evidence.contains("Subagent reviewer completed: no blocking findings"));
        assert!(evidence.contains("Subagent synthesis completed: review batch clean"));
        let evidence_records = evidence
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("evidence json line"))
            .collect::<Vec<_>>();
        let outcome_evidence = evidence_records
            .iter()
            .find(|record| {
                record["summary"]
                    .as_str()
                    .is_some_and(|summary| summary.contains("Subagent reviewer completed"))
            })
            .expect("outcome evidence");
        let source_ids = outcome_evidence["source_ids"]
            .as_array()
            .expect("source ids")
            .iter()
            .map(|value| value.as_str().expect("source id"))
            .collect::<Vec<_>>();
        assert!(source_ids.contains(&"subagents:review"));
        assert!(source_ids.contains(&"subagent:review:reviewer"));
        assert!(!source_ids.contains(&"subagent:reviewer"));
    }

    #[test]
    fn record_subagent_plan_rejects_ambiguous_prompt_rows() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        let source = temp.path().join("bad-plan.json");
        fs::write(
            &source,
            serde_json::to_string_pretty(&json!({
                "task": "review batch",
                "roles": [{ "name": "reviewer" }],
                "prompts": [
                    { "role": "reviewer", "prompt": "first prompt" },
                    { "role": "reviewer", "prompt": "second prompt" },
                    { "role": "security_reviewer", "prompt": "extra prompt" }
                ],
                "duplicate_roles_ignored": {
                    "test_runner": []
                }
            }))
            .expect("plan json"),
        )
        .expect("write plan");

        let error = record_subagent_plan(RecordSubagentPlanArgs {
            capsule: capsule.clone(),
            batch_id: "review".to_string(),
            source,
            command: None,
            recorded_at: "2026-05-09T05:00:00Z".parse().unwrap(),
        })
        .expect_err("ambiguous prompt rows rejected");

        let message = format!("{error:#}");
        assert!(
            message.contains("duplicate prompt for role reviewer"),
            "{message}"
        );
        assert!(
            message.contains("prompt role security_reviewer is not present in plan roles"),
            "{message}"
        );
        assert!(
            message.contains("duplicate_roles_ignored[test_runner] must not be empty"),
            "{message}"
        );
    }

    #[test]
    fn record_subagent_plan_validates_evidence_before_write() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        let source = write_subspawn_plan_fixture(
            temp.path(),
            "review-plan.json",
            "review batch",
            &["reviewer"],
        );

        let error = record_subagent_plan(RecordSubagentPlanArgs {
            capsule: capsule.clone(),
            batch_id: "review".to_string(),
            source,
            command: Some("subspawn_plan.py plan\nwith-control".to_string()),
            recorded_at: "2026-05-09T05:00:00Z".parse().unwrap(),
        })
        .expect_err("invalid evidence rejected before write");

        assert!(
            format!("{error:#}").contains("invalid subagent evidence record"),
            "{error:#}"
        );
        let subagents: Subagents = read_json(&capsule.join("subagents.json")).expect("subagents");
        assert!(subagents.batches.is_empty());
    }

    #[test]
    fn record_subagent_plan_rejects_unknown_json_fields() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;

        for (file_name, extra) in [
            ("unknown-root.json", json!({"unexpected_root": true})),
            (
                "unknown-role.json",
                json!({"roles": [{ "name": "reviewer", "unexpected_role": true }]}),
            ),
            (
                "unknown-prompt.json",
                json!({"prompts": [{ "role": "reviewer", "prompt": "Review.", "unexpected_prompt": true }]}),
            ),
        ] {
            let mut plan = json!({
                "task": "review batch",
                "mode": "read-only",
                "scope": "fixture scope",
                "wait_policy": "strict",
                "rendezvous_required": true,
                "roles": [{ "name": "reviewer" }],
                "prompts": [{ "role": "reviewer", "prompt": "Review." }],
                "registry_issues": [],
                "duplicate_roles_ignored": {}
            });
            merge_json_object(&mut plan, extra);
            let source = temp.path().join(file_name);
            fs::write(
                &source,
                serde_json::to_string_pretty(&plan).expect("plan json"),
            )
            .expect("write plan");

            let error = record_subagent_plan(RecordSubagentPlanArgs {
                capsule: capsule.clone(),
                batch_id: file_name.trim_end_matches(".json").to_string(),
                source,
                command: None,
                recorded_at: "2026-05-09T05:00:00Z".parse().unwrap(),
            })
            .expect_err("unknown field rejected");

            assert!(format!("{error:#}").contains("unknown field"), "{error:#}");
        }

        let subagents: Subagents = read_json(&capsule.join("subagents.json")).expect("subagents");
        assert!(subagents.batches.is_empty());
    }

    #[test]
    fn record_subagent_completed_synthesis_requires_verified_terminal_outcomes() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        let source = write_subspawn_plan_fixture(
            temp.path(),
            "review-plan.json",
            "review batch",
            &["reviewer"],
        );
        record_subagent_plan(RecordSubagentPlanArgs {
            capsule: capsule.clone(),
            batch_id: "review".to_string(),
            source,
            command: None,
            recorded_at: "2026-05-09T05:00:00Z".parse().unwrap(),
        })
        .expect("record plan");

        let error = record_subagent_synthesis(RecordSubagentSynthesisArgs {
            capsule,
            batch_id: "review".to_string(),
            status: SubagentSynthesisStatus::Completed,
            summary: "review batch clean".to_string(),
            human_verified: true,
            source_ids: vec!["synthesis:review".to_string()],
            artifacts: vec!["review-summary.md".to_string()],
            recorded_at: "2026-05-09T05:20:00Z".parse().unwrap(),
        })
        .expect_err("completed synthesis requires completed agents");

        assert!(
            format!("{error:#}").contains("incomplete roles: reviewer"),
            "{error:#}"
        );
    }

    #[test]
    fn record_subagent_outcome_rejects_pending_terminal_disposition() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        let source = write_subspawn_plan_fixture(
            temp.path(),
            "review-plan.json",
            "review batch",
            &["reviewer"],
        );
        record_subagent_plan(RecordSubagentPlanArgs {
            capsule: capsule.clone(),
            batch_id: "review".to_string(),
            source,
            command: None,
            recorded_at: "2026-05-09T05:00:00Z".parse().unwrap(),
        })
        .expect("record plan");

        let error = record_subagent_outcome(RecordSubagentOutcomeArgs {
            capsule: capsule.clone(),
            batch_id: "review".to_string(),
            role: "reviewer".to_string(),
            status: SubagentOutcomeStatus::Completed,
            summary: "not actually finalized".to_string(),
            disposition: SubagentDisposition::Pending,
            human_verified: true,
            source_ids: vec!["reviewer:1".to_string()],
            artifacts: vec!["review-notes.md".to_string()],
            recorded_at: "2026-05-09T05:10:00Z".parse().unwrap(),
        })
        .expect_err("pending disposition rejected for terminal outcomes");

        assert!(
            format!("{error:#}").contains("terminal subagent outcomes require a final disposition"),
            "{error:#}"
        );
        let subagents: Subagents = read_json(&capsule.join("subagents.json")).expect("subagents");
        assert_eq!(subagents.batches[0].agents[0].status, "planned");
    }

    #[test]
    fn record_subagent_blocked_outcome_marks_batch_blocked() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        let source = write_subspawn_plan_fixture(
            temp.path(),
            "review-plan.json",
            "review batch",
            &["reviewer"],
        );
        record_subagent_plan(RecordSubagentPlanArgs {
            capsule: capsule.clone(),
            batch_id: "review".to_string(),
            source,
            command: None,
            recorded_at: "2026-05-09T05:00:00Z".parse().unwrap(),
        })
        .expect("record plan");

        let outcome = record_subagent_outcome(RecordSubagentOutcomeArgs {
            capsule: capsule.clone(),
            batch_id: "review".to_string(),
            role: "reviewer".to_string(),
            status: SubagentOutcomeStatus::Blocked,
            summary: "waiting for required input".to_string(),
            disposition: SubagentDisposition::Pending,
            human_verified: true,
            source_ids: vec!["reviewer:1".to_string()],
            artifacts: vec!["review-notes.md".to_string()],
            recorded_at: "2026-05-09T05:10:00Z".parse().unwrap(),
        })
        .expect("record blocked outcome");

        assert_eq!(outcome.batch.status, "blocked");
        assert!(validate_capsule(&capsule).expect("validate").valid);

        let synthesis = record_subagent_synthesis(RecordSubagentSynthesisArgs {
            capsule,
            batch_id: "review".to_string(),
            status: SubagentSynthesisStatus::Partial,
            summary: "blocked agent still needs user input".to_string(),
            human_verified: true,
            source_ids: vec!["synthesis:review".to_string()],
            artifacts: vec!["review-summary.md".to_string()],
            recorded_at: "2026-05-09T05:20:00Z".parse().unwrap(),
        })
        .expect("record partial synthesis");

        assert_eq!(synthesis.batch.status, "blocked");
    }

    #[test]
    fn validate_capsule_rejects_invalid_subagents_semantics() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        fs::write(
            capsule.join("subagents.json"),
            serde_json::to_string_pretty(&json!({
                "schema": SUBAGENTS_SCHEMA,
                "batches": [{
                    "id": "review",
                    "status": "nonsense",
                    "duplicate_roles_ignored": {
                        "test_runner": [""]
                    },
                    "prompts": [{
                        "role": "Bad Role",
                        "prompt_id": "",
                        "prompt_hash": "fnv1a64:example"
                    }],
                    "agents": [{
                        "role": "reviewer",
                        "task": "review batch",
                        "status": "completed",
                        "summary": "done",
                        "prompt_id": "review:reviewer",
                        "prompt_hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                        "disposition": "accepted",
                        "human_verified": false
                    }]
                }]
            }))
            .expect("subagents json"),
        )
        .expect("write subagents");

        let validation = validate_capsule(&capsule).expect("validate capsule");

        assert!(!validation.valid);
        let joined = validation.errors.join("\n");
        assert!(joined.contains("status \"nonsense\""), "{joined}");
        assert!(
            joined.contains("prompt_hash must start with sha256:"),
            "{joined}"
        );
        assert!(
            joined.contains("terminal status requires a final human-verified disposition"),
            "{joined}"
        );
        assert!(joined.contains("prompt role Bad Role"), "{joined}");
    }

    #[test]
    fn validate_capsule_rejects_empty_completed_subagent_batch() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        fs::write(
            capsule.join("subagents.json"),
            serde_json::to_string_pretty(&json!({
                "schema": SUBAGENTS_SCHEMA,
                "batches": [{
                    "id": "review",
                    "status": "completed",
                    "prompts": [],
                    "agents": [],
                    "synthesis": {
                        "status": "completed",
                        "summary": "nothing reviewed",
                        "human_verified": true,
                        "source_ids": ["synthesis:review"],
                        "artifacts": ["review-summary.md"],
                        "updated_at": "2026-05-09T05:20:00Z"
                    }
                }]
            }))
            .expect("subagents json"),
        )
        .expect("write subagents");

        let validation = validate_capsule(&capsule).expect("validate capsule");

        assert!(!validation.valid);
        let joined = validation.errors.join("\n");
        assert!(joined.contains("prompts must not be empty"), "{joined}");
        assert!(joined.contains("agents must not be empty"), "{joined}");
    }

    #[cfg(unix)]
    #[test]
    fn write_json_refuses_symlink_targets() {
        let temp = tempdir().expect("tempdir");
        let target = temp.path().join("target.json");
        fs::write(&target, "{\"sentinel\":true}\n").expect("write target");
        let link = temp.path().join("link.json");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");

        let error = write_json(link, &json!({"changed": true})).expect_err("symlink refused");

        let message = format!("{error:#}");
        assert!(
            message.contains("failed to create") || message.contains("Too many levels"),
            "{message}"
        );
        let target_content = fs::read_to_string(target).expect("target content");
        assert!(target_content.contains("sentinel"));
        assert!(!target_content.contains("changed"));
    }

    #[test]
    fn record_subagent_outcome_requires_human_verified_disposition() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().to_path_buf()))
            .expect("init capsule")
            .path;
        let source = write_subspawn_plan_fixture(
            temp.path(),
            "review-plan.json",
            "review batch",
            &["reviewer"],
        );
        record_subagent_plan(RecordSubagentPlanArgs {
            capsule: capsule.clone(),
            batch_id: "review".to_string(),
            source,
            command: None,
            recorded_at: "2026-05-09T05:00:00Z".parse().unwrap(),
        })
        .expect("record plan");

        let error = record_subagent_outcome(RecordSubagentOutcomeArgs {
            capsule,
            batch_id: "review".to_string(),
            role: "reviewer".to_string(),
            status: SubagentOutcomeStatus::Completed,
            summary: "looks good".to_string(),
            disposition: SubagentDisposition::Accepted,
            human_verified: false,
            source_ids: Vec::new(),
            artifacts: Vec::new(),
            recorded_at: "2026-05-09T05:10:00Z".parse().unwrap(),
        })
        .expect_err("requires human verification");

        assert!(
            format!("{error:#}").contains("human_verified must be set"),
            "{error:#}"
        );
    }

    #[test]
    fn append_evidence_updates_ledger_and_status_summary() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;

        let result = append_evidence(AppendEvidenceArgs {
            capsule: capsule.clone(),
            record: EvidenceRecord {
                schema: EVIDENCE_SCHEMA.to_string(),
                kind: EvidenceKind::Decision,
                at: "2026-05-09T06:00:00Z".parse().unwrap(),
                summary: "Use one typed evidence append command".to_string(),
                command: None,
                exit_code: None,
                source_ids: vec!["issue:42".to_string()],
                actor: Some("codex".to_string()),
                tool: Some("codex-dev".to_string()),
                confidence: Some(95),
                residual_risk: Some("future PR normalizers still need fixtures".to_string()),
                artifacts: vec!["docs/reference/codex-dev-cli.md".to_string()],
            },
        })
        .expect("append evidence");

        assert_eq!(result.record.kind, EvidenceKind::Decision);
        assert_eq!(result.evidence.total, 2);
        assert!(
            result
                .evidence
                .by_kind
                .iter()
                .any(|kind| kind.kind == EvidenceKind::Decision
                    && kind.count == 1
                    && kind.latest_summary == "Use one typed evidence append command")
        );

        let status = capsule_status(&capsule).expect("status");
        assert_eq!(
            status.updated_at,
            "2026-05-09T06:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
        assert_eq!(status.evidence.total, 2);

        let rendered = render_capsule(&capsule).expect("render");
        assert!(rendered.markdown.contains("## Evidence"));
        assert!(
            rendered
                .markdown
                .contains("`decision`: 1 record(s); latest")
        );
    }

    #[test]
    fn append_evidence_rejects_invalid_records_before_writing() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let evidence_before = fs::read_to_string(capsule.join("evidence.jsonl")).unwrap();

        let error = append_evidence(AppendEvidenceArgs {
            capsule: capsule.clone(),
            record: EvidenceRecord {
                schema: EVIDENCE_SCHEMA.to_string(),
                kind: EvidenceKind::Ci,
                at: "2026-05-09T06:00:00Z".parse().unwrap(),
                summary: " ".to_string(),
                command: None,
                exit_code: Some(1),
                source_ids: vec!["".to_string()],
                actor: Some("codex".to_string()),
                tool: Some("codex-dev".to_string()),
                confidence: Some(90),
                residual_risk: None,
                artifacts: Vec::new(),
            },
        })
        .expect_err("invalid evidence rejected");

        let message = error.to_string();
        assert!(message.contains("summary must not be empty"));
        assert!(message.contains("exit_code requires command"));
        assert!(message.contains("source_ids[0] must not be empty"));
        assert_eq!(
            fs::read_to_string(capsule.join("evidence.jsonl")).unwrap(),
            evidence_before
        );
    }

    #[test]
    fn append_evidence_keeps_capsule_updated_at_monotonic() {
        let temp = tempdir().expect("tempdir");
        let mut args = init_args(temp.path().join("tasks"));
        args.created_at = "2026-05-09T10:00:00Z".parse().unwrap();
        let capsule = init_capsule(args).expect("init capsule").path;

        append_evidence(AppendEvidenceArgs {
            capsule: capsule.clone(),
            record: EvidenceRecord {
                schema: EVIDENCE_SCHEMA.to_string(),
                kind: EvidenceKind::Manual,
                at: "2026-05-09T09:00:00Z".parse().unwrap(),
                summary: "Backfilled manual note".to_string(),
                command: None,
                exit_code: None,
                source_ids: Vec::new(),
                actor: None,
                tool: None,
                confidence: None,
                residual_risk: None,
                artifacts: Vec::new(),
            },
        })
        .expect("append backfilled evidence");

        let status = capsule_status(&capsule).expect("status");
        assert_eq!(
            status.updated_at,
            "2026-05-09T10:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn append_evidence_rejects_symlinked_contract_file_before_writing() {
        let temp = tempdir().expect("tempdir");
        let capsule = init_capsule(init_args(temp.path().join("tasks")))
            .expect("init capsule")
            .path;
        let evidence_path = capsule.join("evidence.jsonl");
        let outside_path = temp.path().join("outside-evidence.jsonl");
        fs::write(&outside_path, "not jsonl").expect("write invalid outside evidence");
        fs::remove_file(&evidence_path).expect("remove evidence");
        std::os::unix::fs::symlink(&outside_path, &evidence_path).expect("symlink evidence");
        let outside_before = fs::read_to_string(&outside_path).expect("outside before");

        let error = append_evidence(AppendEvidenceArgs {
            capsule: capsule.clone(),
            record: EvidenceRecord {
                schema: EVIDENCE_SCHEMA.to_string(),
                kind: EvidenceKind::Manual,
                at: "2026-05-09T06:00:00Z".parse().unwrap(),
                summary: "Symlink write attempt".to_string(),
                command: None,
                exit_code: None,
                source_ids: Vec::new(),
                actor: None,
                tool: None,
                confidence: None,
                residual_risk: None,
                artifacts: Vec::new(),
            },
        })
        .expect_err("symlinked evidence rejected");

        assert!(
            error
                .to_string()
                .contains("symlinked capsule contract file")
        );
        assert_eq!(
            fs::read_to_string(&outside_path).expect("outside after"),
            outside_before
        );
    }

    #[test]
    fn command_rendering_preserves_argument_boundaries() {
        let command = vec![
            "fixture-command".to_string(),
            "-c".to_string(),
            "print('hello world')".to_string(),
        ];

        assert_eq!(
            render_command(&command),
            "fixture-command -c 'print('\\''hello world'\\'')'"
        );
    }
}
