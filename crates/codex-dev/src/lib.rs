use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Component;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const CAPSULE_SCHEMA: &str = "codex-dev.task-capsule.v1";
pub const EVIDENCE_SCHEMA: &str = "codex-dev.evidence.v1";
pub const VERIFICATION_SCHEMA: &str = "codex-dev.verification.v1";
pub const SUBAGENTS_SCHEMA: &str = "codex-dev.subagents.v1";
pub const PR_SCHEMA: &str = "codex-dev.pr.v1";
pub const OUTPUT_SCHEMA: &str = "codex-dev.output.v1";

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

    write_markdown(
        path.join("plan.md"),
        &format!("# Plan\n\n{}\n", capsule.objective),
    )?;
    write_markdown(path.join("decisions.md"), "# Decisions\n\n")?;
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
    "retrospective.md",
];

pub fn validate_capsule(path: &Path) -> Result<ValidationResult> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
}
