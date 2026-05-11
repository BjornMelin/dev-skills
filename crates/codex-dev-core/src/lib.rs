use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
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
use sha2::{Digest, Sha256};

pub const CAPSULE_SCHEMA: &str = "codex-dev.task-capsule.v1";
pub const EVIDENCE_SCHEMA: &str = "codex-dev.evidence.v1";
pub const VERIFICATION_SCHEMA: &str = "codex-dev.verification.v1";
pub const SUBAGENTS_SCHEMA: &str = "codex-dev.subagents.v1";
pub const PR_SCHEMA: &str = "codex-dev.pr.v1";
pub const PR_CONTROL_PLAN_SCHEMA: &str = "codex-dev.pr-control-plan.v1";
pub const OUTPUT_SCHEMA: &str = "codex-dev.output.v1";
pub const POLICY_GATES_SCHEMA: &str = "codex-dev.policy-gates.v1";

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrRecordArgs {
    pub capsule: PathBuf,
    pub source: PathBuf,
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
                "PR snapshot recorded for {}; {} unresolved review thread(s); {} check(s)",
                render_pr_label(&pr),
                pr.review_threads.unresolved,
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
            checks: Vec::new(),
            review_threads: ReviewThreadSummary {
                unresolved: 0,
                last_checked_at: created_at,
            },
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
        match validate_subagents(&path.join("subagents.json")) {
            Ok(file_errors) => errors.extend(file_errors),
            Err(error) => errors.push(format!("invalid subagents.json: {error:#}")),
        }
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

pub fn render_pr_status(pr: &PrEvidence) -> String {
    format!(
        "{} {}: {} unresolved review thread(s), {} check(s)",
        render_pr_label(pr),
        pr.state,
        pr.review_threads.unresolved,
        pr.checks.len()
    )
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

fn stable_prompt_hash(prompt: &str) -> String {
    let digest = Sha256::digest(prompt.as_bytes());
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
    let mut source_ids = vec![format!("subagents:{batch_id}"), format!("subagent:{role}")];
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
        }
    }
}

impl FromStr for PolicyProfile {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "codex_dev" => Ok(Self::CodexDev),
            _ => Err(format!(
                "invalid policy profile {value:?}; expected codex_dev"
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
                required: true,
                network: false,
                secrets: false,
            }],
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

        let pr: PrEvidence = read_json(&capsule.join("pr.json")).expect("pr json");
        assert_eq!(pr.number, Some(25));

        let capsule_state: Capsule = read_json(&capsule.join("capsule.json")).expect("capsule");
        assert_eq!(capsule_state.pull_requests, vec![25]);

        let evidence = fs::read_to_string(capsule.join("evidence.jsonl")).expect("evidence");
        assert!(evidence.contains("PR snapshot recorded"));
        assert!(evidence.contains("fixture-pr-recorder --source pr-snapshot.json"));
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
