use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::path::Component;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const CAPSULE_SCHEMA: &str = "codex-dev.task-capsule.v1";
pub const EVIDENCE_SCHEMA: &str = "codex-dev.evidence.v1";
pub const VERIFICATION_SCHEMA: &str = "codex-dev.verification.v1";
pub const SUBAGENTS_SCHEMA: &str = "codex-dev.subagents.v1";
pub const PR_SCHEMA: &str = "codex-dev.pr.v1";
pub const PR_CONTROL_PLAN_SCHEMA: &str = "codex-dev.pr-control-plan.v1";
pub const OUTPUT_SCHEMA: &str = "codex-dev.output.v1";
pub const POLICY_GATES_SCHEMA: &str = "codex-dev.policy-gates.v1";

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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppendEvidenceResult {
    pub capsule: PathBuf,
    pub evidence_path: PathBuf,
    pub record: EvidenceRecord,
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

fn ensure_regular_contract_files(capsule_path: &Path) -> Result<()> {
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
    let records = read_evidence_records(&capsule_path.join("evidence.jsonl"))?;
    Ok(summarize_evidence(&records))
}

fn summarize_evidence(records: &[EvidenceRecord]) -> EvidenceSummary {
    let mut by_kind: BTreeMap<EvidenceKind, EvidenceKindSummary> = BTreeMap::new();
    for record in records {
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
    }

    EvidenceSummary {
        total: records.len() as u64,
        by_kind: by_kind.into_values().collect(),
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
    let records = read_evidence_records(path)?;
    let mut errors = Vec::new();
    for (index, record) in records.iter().enumerate() {
        for error in validate_evidence_record(record) {
            errors.push(format!("evidence.jsonl line {} {error}", index + 1));
        }
    }
    Ok(errors)
}

fn read_evidence_records(path: &Path) -> Result<Vec<EvidenceRecord>> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: EvidenceRecord = serde_json::from_str(&line)
            .with_context(|| format!("line {} is not valid evidence JSON", index + 1))?;
        records.push(record);
    }
    Ok(records)
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
    let mut file =
        File::create(&path).with_context(|| format!("failed to create {}", path.display()))?;
    serde_json::to_writer_pretty(&mut file, value)?;
    writeln!(file)?;
    Ok(())
}

pub fn append_jsonl<T: Serialize>(path: PathBuf, value: &T) -> Result<()> {
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
