use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand, ValueEnum};
use codex_dev_core::{
    COMMIT_PLAN_SCHEMA, COMMIT_VALIDATION_SCHEMA, CapsuleStatus, CommitPlan, CommitPlanGroup,
    CommitPlanSummary, CommitValidationReport, GitHubReviewComment, InitArgs,
    LOCAL_REVIEW_WORKLIST_SCHEMA, LocalReviewWorkItem, LocalReviewWorklist,
    LocalReviewWorklistSummary, PR_REVIEW_CLOSEOUT_SCHEMA, PR_REVIEW_WORKLIST_SCHEMA,
    PolicyProfile, PrAgentSeverity, PrAgentSourceStatus, PrReviewCloseoutReport,
    PrReviewCloseoutSummary, PrReviewCloseoutThread, PrReviewCluster, PrReviewSuggestion,
    PrReviewWorkItem, PrReviewWorklist, PrReviewWorklistSummary,
    github_review_threads_from_graphql, init_capsule, read_json, stable_text_hash, write_json,
};
use serde_json::{Value, json};

use crate::{
    CommandOutput, PrAgentArgs, RESOLVE_REVIEW_THREAD_MUTATION, graph_ql_thread_command,
    parse_github_repository, policy_manifest, run_hosted_command, run_pr_agent_state,
};

#[derive(Subcommand, Debug)]
pub(crate) enum PrReviewCommand {
    /// Capture unresolved hosted review work into a first-class worklist.
    Start(PrReviewStartArgs),
    /// Force a fresh hosted-state capture and worklist render.
    Refresh(PrReviewStartArgs),
    /// Query a captured PR review worklist.
    Query(PrReviewQueryArgs),
    /// Render a captured PR review worklist as Markdown.
    Render(PrReviewRenderArgs),
    /// Plan or apply exact GitHub suggestion fences.
    #[command(name = "apply-suggestions")]
    ApplySuggestions(PrReviewApplySuggestionsArgs),
    /// Plan or apply verified hosted thread resolution.
    Closeout(PrReviewCloseoutArgs),
}

#[derive(Args, Clone, Debug)]
pub(crate) struct PrReviewStartArgs {
    #[arg(long, value_name = "OWNER/REPO")]
    pub repo: Option<String>,
    #[arg(long, value_name = "PR_NUMBER")]
    pub number: Option<u64>,
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: Option<PathBuf>,
    #[arg(long, value_name = "SOURCE_DIR")]
    pub source_dir: Option<PathBuf>,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
    #[arg(long, value_name = "JSON")]
    pub out: Option<PathBuf>,
    #[arg(
        long,
        help = "Require a fresh live capture instead of relying on cached caller state"
    )]
    pub fresh: bool,
}

#[derive(Args, Clone, Debug)]
pub(crate) struct PrReviewQueryArgs {
    #[arg(long, value_name = "WORKLIST_JSON")]
    pub worklist: PathBuf,
    #[arg(long, value_name = "ITEM_ID")]
    pub item: Option<String>,
    #[arg(long, value_name = "THREAD_ID")]
    pub thread_id: Option<String>,
    #[arg(long, value_name = "PATH")]
    pub file: Option<String>,
    #[arg(long, value_name = "TEXT")]
    pub text: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub(crate) struct PrReviewRenderArgs {
    #[arg(long, value_name = "WORKLIST_JSON")]
    pub worklist: PathBuf,
}

#[derive(Args, Clone, Debug)]
pub(crate) struct PrReviewApplySuggestionsArgs {
    #[arg(long, value_name = "WORKLIST_JSON")]
    pub worklist: PathBuf,
    #[arg(long, value_name = "REPO_ROOT", default_value = ".")]
    pub repo_root: PathBuf,
    #[arg(long, value_name = "ITEM_ID")]
    pub item: Option<String>,
    #[arg(long, help = "Rewrite files for exact-hunk suggestion matches")]
    pub apply: bool,
}

#[derive(Args, Clone, Debug)]
pub(crate) struct PrReviewCloseoutArgs {
    #[arg(long, value_name = "OWNER/REPO")]
    pub repo: Option<String>,
    #[arg(long, value_name = "PR_NUMBER")]
    pub number: Option<u64>,
    #[arg(long, value_name = "CAPSULE_DIR")]
    pub capsule: Option<PathBuf>,
    #[arg(long, value_name = "WORKLIST_JSON")]
    pub worklist: Option<PathBuf>,
    #[arg(long = "thread-id", value_name = "THREAD_ID")]
    pub thread_ids: Vec<String>,
    #[arg(long, value_name = "HEAD_SHA")]
    pub expected_head_sha: Option<String>,
    #[arg(long = "commit", value_name = "COMMIT_SHA")]
    pub commit_shas: Vec<String>,
    #[arg(long = "validation-command", value_name = "COMMAND")]
    pub validation_commands: Vec<String>,
    #[arg(long, value_name = "SOURCE_DIR")]
    pub source_dir: Option<PathBuf>,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
    #[arg(long, help = "Resolve verified current hosted review threads")]
    pub apply: bool,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ReviewCommand {
    /// Ingest a local review notes file into a worklist.
    Ingest(LocalReviewIngestArgs),
    /// Render a local review worklist as Markdown.
    Render(LocalReviewRenderArgs),
    /// Query a local review worklist.
    Query(LocalReviewQueryArgs),
}

#[derive(Args, Clone, Debug)]
pub(crate) struct LocalReviewIngestArgs {
    #[arg(long, value_name = "FILE")]
    pub source: PathBuf,
    #[arg(long, value_enum, default_value_t = LocalReviewKind::Manual)]
    pub kind: LocalReviewKind,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
    #[arg(long, value_name = "JSON")]
    pub out: Option<PathBuf>,
}

#[derive(Args, Clone, Debug)]
pub(crate) struct LocalReviewRenderArgs {
    #[arg(long, value_name = "WORKLIST_JSON")]
    pub worklist: PathBuf,
}

#[derive(Args, Clone, Debug)]
pub(crate) struct LocalReviewQueryArgs {
    #[arg(long, value_name = "WORKLIST_JSON")]
    pub worklist: PathBuf,
    #[arg(long, value_name = "ITEM_ID")]
    pub item: Option<String>,
    #[arg(long, value_name = "PATH")]
    pub file: Option<String>,
    #[arg(long, value_name = "TEXT")]
    pub text: Option<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum LocalReviewKind {
    Codex,
    Zen,
    Manual,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CommitCommand {
    /// Plan scoped semantic conventional commit groups from the current diff.
    Plan(CommitPlanArgs),
    /// Validate one commit subject and optional staged-diff constraints.
    Validate(CommitValidateArgs),
}

#[derive(Args, Clone, Debug)]
pub(crate) struct CommitPlanArgs {
    #[arg(long, value_name = "REPO_ROOT", default_value = ".")]
    pub repo_root: PathBuf,
    #[arg(long, value_name = "WORKLIST_JSON")]
    pub worklist: Option<PathBuf>,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
}

#[derive(Args, Clone, Debug)]
pub(crate) struct CommitValidateArgs {
    #[arg(long, value_name = "SUBJECT")]
    pub subject: String,
    #[arg(long, value_name = "REPO_ROOT", default_value = ".")]
    pub repo_root: PathBuf,
    #[arg(long, value_name = "RFC3339")]
    pub checked_at: Option<DateTime<Utc>>,
    #[arg(long, help = "Fail unless the repository has staged changes")]
    pub require_staged: bool,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ReviewCommentFingerprint {
    id: Option<String>,
    author: Option<String>,
    path: Option<String>,
    line: Option<u64>,
    body_excerpt: String,
    body_hash: Option<String>,
}

pub(crate) fn handle_pr_review_command(command: PrReviewCommand) -> Result<CommandOutput> {
    match command {
        PrReviewCommand::Start(args) => handle_pr_review_start(args, "pr review start"),
        PrReviewCommand::Refresh(mut args) => {
            args.fresh = true;
            handle_pr_review_start(args, "pr review refresh")
        }
        PrReviewCommand::Query(args) => {
            let worklist: PrReviewWorklist = read_json(&args.worklist)?;
            let items = filter_pr_review_items(&worklist.items, &args)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            let result = json!({
                "schema": "codex-dev.pr-review-query.v1",
                "repository": worklist.repository,
                "number": worklist.number,
                "matched": items.len(),
                "items": items,
            });
            Ok(CommandOutput {
                ok: true,
                command: "pr review query",
                human: format!("matched {} PR review work item(s)", result["matched"]),
                result,
            })
        }
        PrReviewCommand::Render(args) => {
            let worklist: PrReviewWorklist = read_json(&args.worklist)?;
            let markdown = render_pr_review_worklist(&worklist);
            Ok(CommandOutput {
                ok: true,
                command: "pr review render",
                human: markdown.clone(),
                result: json!({
                    "schema": "codex-dev.pr-review-render.v1",
                    "markdown": markdown,
                }),
            })
        }
        PrReviewCommand::ApplySuggestions(args) => {
            let report = plan_or_apply_suggestions(args)?;
            Ok(CommandOutput {
                ok: report["ok"].as_bool().unwrap_or(false),
                command: "pr review apply-suggestions",
                human: format!(
                    "{} suggestion action(s) {}",
                    report["actions"].as_array().map_or(0, Vec::len),
                    if report["dryRun"].as_bool().unwrap_or(true) {
                        "planned"
                    } else {
                        "processed"
                    }
                ),
                result: report,
            })
        }
        PrReviewCommand::Closeout(args) => {
            let generated_at = args.checked_at.unwrap_or_else(Utc::now);
            let report = plan_or_apply_closeout(args, generated_at)?;
            Ok(CommandOutput {
                ok: report.ok,
                command: "pr review closeout",
                human: format!(
                    "closeout for {}#{}: {} planned, {} applied, {} skipped, {} blocked",
                    report.repository,
                    report.number,
                    report.summary.planned,
                    report.summary.applied,
                    report.summary.skipped,
                    report.summary.blocked
                ),
                result: serde_json::to_value(report)?,
            })
        }
    }
}

fn handle_pr_review_start(args: PrReviewStartArgs, command: &'static str) -> Result<CommandOutput> {
    if args.fresh && args.source_dir.is_some() {
        bail!(
            "--fresh cannot be combined with --source-dir; remove --source-dir to capture live hosted state"
        );
    }
    let checked_at = args.checked_at.unwrap_or_else(Utc::now);
    let (repo, number) = resolve_pr_identity(args.repo.as_deref(), args.number)?;
    let capsule = resolve_pr_review_capsule(args.capsule.as_deref(), &repo, number, checked_at)?;
    let state = run_pr_agent_state(
        PrAgentArgs {
            capsule: capsule.clone(),
            repo: repo.clone(),
            number,
            checked_at: Some(checked_at),
            source_dir: args.source_dir.clone(),
        },
        checked_at,
    )?;
    let mut worklist = build_pr_review_worklist(&state, command)?;
    if state
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == PrAgentSeverity::Error)
    {
        worklist
            .diagnostics
            .push("one or more hosted source captures failed".to_string());
    }
    let worklist_path = capsule.join("pr-review-worklist.json");
    write_json(worklist_path.clone(), &worklist)?;
    if let Some(out) = args.out.as_deref() {
        write_json(out.to_path_buf(), &worklist)?;
    }
    let result = pr_review_start_result(&worklist, &worklist_path, args.out.as_deref())?;
    let path_note = pr_review_start_path_note(args.out.as_deref());
    let worklist_path_display = display_canonical_path(&worklist_path);
    let human = if worklist.summary.fast_noop {
        format!(
            "captured {}#{} review worklist at {}: no actionable unresolved work{}",
            worklist.repository, worklist.number, worklist_path_display, path_note
        )
    } else {
        format!(
            "captured {}#{} review worklist at {} with {} actionable item(s) in {} cluster(s){}",
            worklist.repository,
            worklist.number,
            worklist_path_display,
            worklist.summary.actionable_items,
            worklist.summary.clusters,
            path_note
        )
    };
    Ok(CommandOutput {
        ok: worklist.diagnostics.is_empty(),
        command,
        human,
        result,
    })
}

fn pr_review_start_result(
    worklist: &PrReviewWorklist,
    worklist_path: &Path,
    out_path: Option<&Path>,
) -> Result<Value> {
    let mut result = serde_json::to_value(worklist)?;
    if let Some(map) = result.as_object_mut() {
        map.insert(
            "worklist_path".to_string(),
            json!(display_canonical_path(worklist_path)),
        );
        if let Some(out_path) = out_path {
            map.insert(
                "out_path".to_string(),
                json!(display_canonical_path(out_path)),
            );
        }
    }
    Ok(result)
}

fn pr_review_start_path_note(out_path: Option<&Path>) -> String {
    out_path.map_or_else(String::new, |out_path| {
        format!(" and wrote copy to {}", display_canonical_path(out_path))
    })
}

fn display_canonical_path(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

pub(crate) fn handle_review_command(command: ReviewCommand) -> Result<CommandOutput> {
    match command {
        ReviewCommand::Ingest(args) => {
            let checked_at = args.checked_at.unwrap_or_else(Utc::now);
            let worklist = ingest_local_review(&args.source, args.kind, checked_at)?;
            if let Some(out) = args.out {
                write_json(out, &worklist)?;
            }
            Ok(CommandOutput {
                ok: true,
                command: "review ingest",
                human: format!(
                    "ingested {} local review item(s) from {}",
                    worklist.summary.items, worklist.source
                ),
                result: serde_json::to_value(worklist)?,
            })
        }
        ReviewCommand::Render(args) => {
            let worklist: LocalReviewWorklist = read_json(&args.worklist)?;
            let markdown = render_local_review_worklist(&worklist);
            Ok(CommandOutput {
                ok: true,
                command: "review render",
                human: markdown.clone(),
                result: json!({
                    "schema": "codex-dev.review-render.v1",
                    "markdown": markdown,
                }),
            })
        }
        ReviewCommand::Query(args) => {
            let worklist: LocalReviewWorklist = read_json(&args.worklist)?;
            let items = worklist
                .items
                .iter()
                .filter(|item| local_review_item_matches(item, &args))
                .cloned()
                .collect::<Vec<_>>();
            Ok(CommandOutput {
                ok: true,
                command: "review query",
                human: format!("matched {} local review item(s)", items.len()),
                result: json!({
                    "schema": "codex-dev.review-query.v1",
                    "matched": items.len(),
                    "items": items,
                }),
            })
        }
    }
}

pub(crate) fn handle_commit_command(command: CommitCommand) -> Result<CommandOutput> {
    match command {
        CommitCommand::Plan(args) => {
            let checked_at = args.checked_at.unwrap_or_else(Utc::now);
            let plan = build_commit_plan(args, checked_at)?;
            Ok(CommandOutput {
                ok: plan.ok,
                command: "commit plan",
                human: format!(
                    "planned {} semantic commit group(s) for {} changed file(s)",
                    plan.summary.groups, plan.summary.changed_files
                ),
                result: serde_json::to_value(plan)?,
            })
        }
        CommitCommand::Validate(args) => {
            let checked_at = args.checked_at.unwrap_or_else(Utc::now);
            let report = validate_commit_subject_report(&args, checked_at)?;
            Ok(CommandOutput {
                ok: report.ok,
                command: "commit validate",
                human: if report.ok {
                    format!("valid commit subject: {}", report.subject)
                } else {
                    format!(
                        "invalid commit subject: {}",
                        report.errors.first().cloned().unwrap_or_default()
                    )
                },
                result: serde_json::to_value(report)?,
            })
        }
    }
}

fn resolve_pr_identity(repo: Option<&str>, number: Option<u64>) -> Result<(String, u64)> {
    let repo = match repo {
        Some(repo) => repo.to_string(),
        None => infer_repo_from_git_or_gh()?,
    };
    parse_github_repository(&repo)?;
    let number = match number {
        Some(number) => number,
        None => infer_pr_number_from_gh()?,
    };
    Ok((repo, number))
}

fn resolve_pr_review_capsule(
    explicit: Option<&Path>,
    repo: &str,
    number: u64,
    checked_at: DateTime<Utc>,
) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }
    let id = format!("pr-review-{}-{}", safe_id(repo), number);
    let root = PathBuf::from(".codex/tasks");
    let path = root.join(&id);
    if path.join("capsule.json").is_file() {
        return Ok(path);
    }
    init_capsule(InitArgs {
        title: format!("PR review remediation for {repo}#{number}"),
        objective: "Capture, fix, verify, commit, push, and close hosted PR review threads"
            .to_string(),
        branch: current_git_branch().unwrap_or_else(|| "unknown".to_string()),
        base_branch: "main".to_string(),
        issues: Vec::new(),
        pull_requests: vec![number],
        root,
        slug: Some(id.clone()),
        id: Some(id),
        status: CapsuleStatus::Active,
        created_at: checked_at,
        policy_manifest: policy_manifest(PolicyProfile::CodexDev, checked_at),
        force: false,
    })?;
    Ok(path)
}

fn infer_repo_from_git_or_gh() -> Result<String> {
    if let Ok(output) = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()
        && output.status.success()
        && let Ok(remote) = String::from_utf8(output.stdout)
        && let Some(repo) = parse_github_remote(remote.trim())
    {
        return Ok(repo);
    }
    let output = Command::new("gh")
        .args([
            "repo",
            "view",
            "--json",
            "nameWithOwner",
            "--jq",
            ".nameWithOwner",
        ])
        .output()
        .context("failed to infer repository with git remote or gh repo view")?;
    if !output.status.success() {
        bail!("failed to infer repository; pass --repo OWNER/REPO");
    }
    let repo = String::from_utf8(output.stdout)
        .context("gh repo view returned non-UTF8 output")?
        .trim()
        .to_string();
    parse_github_repository(&repo)?;
    Ok(repo)
}

fn infer_pr_number_from_gh() -> Result<u64> {
    let output = Command::new("gh")
        .args(["pr", "view", "--json", "number", "--jq", ".number"])
        .output()
        .context("failed to infer PR number with gh pr view")?;
    if !output.status.success() {
        bail!("failed to infer PR number; pass --number");
    }
    String::from_utf8(output.stdout)
        .context("gh pr view returned non-UTF8 output")?
        .trim()
        .parse::<u64>()
        .context("gh pr view did not return a numeric PR number")
}

fn parse_github_remote(remote: &str) -> Option<String> {
    let value = remote.trim_end_matches(".git");
    if let Some(rest) = value.strip_prefix("git@github.com:") {
        return Some(rest.to_string());
    }
    if let Some(rest) = value.strip_prefix("https://github.com/") {
        return Some(rest.to_string());
    }
    if let Some(rest) = value.strip_prefix("ssh://git@github.com/") {
        return Some(rest.to_string());
    }
    None
}

fn current_git_branch() -> Option<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn safe_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn build_pr_review_worklist(
    state: &codex_dev_core::PrAgentStateReport,
    source: &str,
) -> Result<PrReviewWorklist> {
    let mut diagnostics = Vec::new();
    if !state.pr.review_threads.authoritative {
        diagnostics.push(
            "review-thread state was not authoritative; refresh hosted review threads before making worklist or closeout decisions"
                .to_string(),
        );
    }
    let thread_source = state
        .sources
        .iter()
        .find(|source| source.id == "gh-review-threads");
    let mut threads = Vec::new();
    if let Some(source) = thread_source {
        if source.status != PrAgentSourceStatus::Captured {
            diagnostics.push(format!(
                "review-thread source was not captured; gh-review-threads status was {:?}",
                source.status
            ));
        } else {
            let path = PathBuf::from(&source.path);
            let value = read_json::<Value>(&path).with_context(|| {
                format!("failed to read review thread source {}", path.display())
            })?;
            let parsed_threads = github_review_threads_from_graphql(&value)?;
            if !parsed_threads.is_complete() {
                diagnostics.push(
                "review-thread state was not authoritative; reviewThreads pagination was incomplete"
                    .to_string(),
            );
            }
            threads = parsed_threads.threads;
        }
    } else {
        diagnostics.push("review-thread source was not captured".to_string());
    }

    let mut items = Vec::new();
    for thread in threads {
        if thread.is_resolved {
            continue;
        }
        if !thread.comments_complete() {
            diagnostics.push(format!(
                "review-thread state was not authoritative; review-thread {} comments were incomplete: captured {} of {} comment(s)",
                thread.id,
                thread.comments.len(),
                thread.comment_count()
            ));
        }
        let status = if thread.is_outdated {
            "outdated"
        } else {
            "actionable"
        };
        let comments = if thread.comments.is_empty() {
            vec![GitHubReviewComment {
                id: None,
                author: None,
                path: None,
                line: None,
                start_line: None,
                body: None,
                diff_hunk: None,
            }]
        } else {
            thread.comments
        };
        for (comment_index, comment) in comments.into_iter().enumerate() {
            let item_id = format!("item-{:03}", items.len() + 1);
            let body = comment
                .body
                .as_deref()
                .unwrap_or("Unresolved GitHub review thread");
            let suggestions = extract_suggestions(
                body,
                comment.diff_hunk.as_deref(),
                comment.line,
                comment.start_line,
            );
            let provider = provider_from_author(comment.author.as_deref());
            let mut hints = vec![
                "Verify this finding against current code before editing.".to_string(),
                "Group any fix with related files into a semantic commit, not by reviewer comment."
                    .to_string(),
            ];
            if provider == "coderabbit" {
                hints.push("CodeRabbit item: preserve concrete prompt/suggestion details, but do not assume stale generated guidance is still valid.".to_string());
            }
            items.push(PrReviewWorkItem {
                id: item_id,
                thread_id: thread.id.clone(),
                comment_id: comment.id.or_else(|| {
                    (comment_index > 0).then(|| format!("{}:{comment_index}", thread.id))
                }),
                provider,
                author: comment.author,
                path: comment.path,
                line: comment.line,
                severity: severity_from_body(body).to_string(),
                action: action_from_body(body, !suggestions.is_empty()).to_string(),
                status: status.to_string(),
                body_excerpt: excerpt(body, 420),
                body_hash: Some(stable_text_hash(body)),
                suggestions,
                hints,
            });
        }
    }
    items.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.line.cmp(&right.line))
            .then(left.thread_id.cmp(&right.thread_id))
    });
    for (index, item) in items.iter_mut().enumerate() {
        item.id = format!("item-{:03}", index + 1);
    }
    let clusters = build_clusters(&items);
    let actionable_items = items
        .iter()
        .filter(|item| item.status == "actionable")
        .count() as u64;
    let suggestion_items = items
        .iter()
        .filter(|item| !item.suggestions.is_empty())
        .count() as u64;
    Ok(PrReviewWorklist {
        schema: PR_REVIEW_WORKLIST_SCHEMA.to_string(),
        repository: state.repository.clone(),
        number: state.number,
        checked_at: state.checked_at,
        head_sha: state.pr.head_sha.clone(),
        source: source.to_string(),
        summary: PrReviewWorklistSummary {
            unresolved_threads: state.pr.review_threads.unresolved,
            actionable_items,
            suggestion_items,
            clusters: clusters.len() as u64,
            fast_noop: state.pr.review_threads.unresolved == 0 && actionable_items == 0,
        },
        items,
        clusters,
        diagnostics,
    })
}

fn provider_from_author(author: Option<&str>) -> String {
    let lower = author.unwrap_or("").to_ascii_lowercase();
    if lower.contains("coderabbit") {
        "coderabbit".to_string()
    } else if lower.contains("copilot") {
        "github-copilot".to_string()
    } else {
        "github-review".to_string()
    }
}

fn severity_from_body(body: &str) -> &'static str {
    let lower = body.to_ascii_lowercase();
    if lower.contains("security") || lower.contains("critical") || lower.contains("data loss") {
        "high"
    } else if lower.contains("bug")
        || lower.contains("incorrect")
        || lower.contains("failing")
        || lower.contains("panic")
    {
        "medium"
    } else {
        "low"
    }
}

fn action_from_body(body: &str, has_suggestion: bool) -> &'static str {
    let lower = body.to_ascii_lowercase();
    if has_suggestion {
        "apply-suggestion-or-verify"
    } else if lower.contains("test") || lower.contains("coverage") {
        "add-or-update-tests"
    } else if lower.contains("doc") {
        "update-documentation"
    } else {
        "verify-and-fix-current-code"
    }
}

fn extract_suggestions(
    body: &str,
    diff_hunk: Option<&str>,
    line: Option<u64>,
    start_line: Option<u64>,
) -> Vec<PrReviewSuggestion> {
    let mut suggestions = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("```suggestion") {
        let after_start = &rest[start + "```suggestion".len()..];
        let after_header = after_start
            .find('\n')
            .map(|index| &after_start[index + 1..])
            .unwrap_or("");
        let Some(end) = closing_fence_offset(after_header) else {
            break;
        };
        let payload = &after_header[..end];
        let replacement = payload
            .strip_suffix("\r\n")
            .or_else(|| payload.strip_suffix('\n'))
            .unwrap_or(payload)
            .to_string();
        let original = original_from_diff_hunk(diff_hunk, line, start_line);
        let apply_mode = if original.is_some() {
            "exact-hunk"
        } else {
            "manual"
        };
        suggestions.push(PrReviewSuggestion {
            id: format!("suggestion-{:03}", suggestions.len() + 1),
            replacement,
            original,
            apply_mode: apply_mode.to_string(),
        });
        rest = &after_header[end + 3..];
    }
    suggestions
}

fn closing_fence_offset(markdown: &str) -> Option<usize> {
    let mut offset = 0;
    for line in markdown.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "```" {
            return Some(offset);
        }
        offset += line.len();
    }
    (markdown.trim_end_matches(['\r', '\n']) == "```").then_some(0)
}

fn original_from_diff_hunk(
    diff_hunk: Option<&str>,
    end_line: Option<u64>,
    start_line: Option<u64>,
) -> Option<String> {
    let hunk = diff_hunk?;
    if let Some(end_line) = end_line {
        let start_line = start_line.unwrap_or(end_line).min(end_line);
        let current = current_diff_lines_for_range(hunk, start_line, end_line);
        if !current.is_empty() {
            return Some(current.join("\n"));
        }
    }
    let added = hunk
        .lines()
        .filter_map(|line| line.strip_prefix('+').filter(|_| !line.starts_with("+++")))
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if !added.is_empty() {
        return Some(added.join("\n"));
    }
    hunk.lines()
        .rev()
        .find_map(|line| {
            if line.starts_with("---") || line.starts_with("+++") || line.starts_with("@@") {
                None
            } else {
                line.strip_prefix(' ').map(str::to_string)
            }
        })
        .filter(|line| !line.trim().is_empty())
}

fn current_diff_lines_for_range(hunk: &str, start_line: u64, end_line: u64) -> Vec<String> {
    let mut current_line = None;
    let mut current = Vec::new();
    for line in hunk.lines() {
        if line.starts_with("@@") {
            current_line = parse_current_hunk_start(line);
            continue;
        }
        if line.starts_with("---") || line.starts_with("+++") {
            continue;
        }
        let Some(line_number) = current_line else {
            continue;
        };
        if let Some(text) = line.strip_prefix('+').or_else(|| line.strip_prefix(' ')) {
            if (start_line..=end_line).contains(&line_number) {
                current.push(text.to_string());
            }
            current_line = line_number.checked_add(1);
        } else if line.starts_with('-') {
            continue;
        }
    }
    current
}

fn parse_current_hunk_start(header: &str) -> Option<u64> {
    let after_plus = header.split_once('+')?.1;
    let digits = after_plus
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse().ok()
}

fn excerpt(value: &str, max_chars: usize) -> String {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= max_chars {
        compact
    } else {
        let mut truncated = compact.chars().take(max_chars).collect::<String>();
        truncated.push_str("...");
        truncated
    }
}

fn build_clusters(items: &[PrReviewWorkItem]) -> Vec<PrReviewCluster> {
    let mut by_prefix = BTreeMap::<String, Vec<String>>::new();
    for item in items {
        let prefix = item
            .path
            .as_deref()
            .map(path_prefix)
            .unwrap_or_else(|| "unknown".to_string());
        by_prefix.entry(prefix).or_default().push(item.id.clone());
    }
    by_prefix
        .into_iter()
        .enumerate()
        .map(|(index, (path_prefix, item_ids))| {
            let id = format!("cluster-{:03}", index + 1);
            PrReviewCluster {
                id: id.clone(),
                path_prefix: path_prefix.clone(),
                item_ids: item_ids.clone(),
                subagent_prompt: format!(
                    "Review and plan fixes for PR review items {} under `{}`. Verify each item against current code and return only still-valid fixes grouped by semantic commit intent.",
                    item_ids.join(", "),
                    path_prefix
                ),
            }
        })
        .collect()
}

fn path_prefix(path: &str) -> String {
    let mut parts = path.split('/').filter(|part| !part.is_empty());
    match (parts.next(), parts.next()) {
        (Some("skills"), Some(skill)) => format!("skills/{skill}"),
        (Some("crates"), Some(crate_name)) => format!("crates/{crate_name}"),
        (Some("docs"), _) => "docs".to_string(),
        (Some(first), _) => first.to_string(),
        _ => "unknown".to_string(),
    }
}

fn filter_pr_review_items<'a>(
    items: &'a [PrReviewWorkItem],
    args: &PrReviewQueryArgs,
) -> Vec<&'a PrReviewWorkItem> {
    items
        .iter()
        .filter(|item| {
            args.item.as_deref().is_none_or(|id| item.id == id)
                && args
                    .thread_id
                    .as_deref()
                    .is_none_or(|thread_id| item.thread_id == thread_id)
                && args
                    .file
                    .as_deref()
                    .is_none_or(|file| item.path.as_deref().is_some_and(|path| path.contains(file)))
                && args.text.as_deref().is_none_or(|text| {
                    item.body_excerpt
                        .to_ascii_lowercase()
                        .contains(&text.to_ascii_lowercase())
                })
        })
        .collect()
}

fn render_pr_review_worklist(worklist: &PrReviewWorklist) -> String {
    let mut markdown = format!(
        "# PR Review Worklist: {}#{}\n\n- Head: {}\n- Actionable items: {}\n- Suggestion items: {}\n- Fast no-op: {}\n\n",
        worklist.repository,
        worklist.number,
        worklist.head_sha.as_deref().unwrap_or("unknown"),
        worklist.summary.actionable_items,
        worklist.summary.suggestion_items,
        worklist.summary.fast_noop
    );
    for cluster in &worklist.clusters {
        markdown.push_str(&format!("## {} ({})\n\n", cluster.id, cluster.path_prefix));
        for item_id in &cluster.item_ids {
            if let Some(item) = worklist.items.iter().find(|item| &item.id == item_id) {
                markdown.push_str(&format!(
                    "- {} `{}` {}:{} [{}] {}\n",
                    item.id,
                    item.thread_id,
                    item.path.as_deref().unwrap_or("<no-path>"),
                    item.line
                        .map(|line| line.to_string())
                        .unwrap_or_else(|| "?".to_string()),
                    item.severity,
                    item.body_excerpt
                ));
            }
        }
        markdown.push('\n');
    }
    markdown
}

fn plan_or_apply_suggestions(args: PrReviewApplySuggestionsArgs) -> Result<Value> {
    let worklist: PrReviewWorklist = read_json(&args.worklist)?;
    let repo_root = fs::canonicalize(&args.repo_root).with_context(|| {
        format!(
            "failed to canonicalize repo root {}",
            args.repo_root.display()
        )
    })?;
    let mut actions = Vec::new();
    let mut pending = Vec::new();
    let mut ok = true;
    for (item_index, item) in worklist
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| suggestion_item_selected(item, args.item.as_deref()))
    {
        for (suggestion_index, suggestion) in item.suggestions.iter().enumerate() {
            let Some(path) = &item.path else {
                ok = false;
                actions.push(suggestion_action(
                    item,
                    suggestion,
                    "blocked",
                    "suggestion has no file path",
                ));
                continue;
            };
            let file_path = match resolve_suggestion_file_path(&repo_root, path) {
                Ok(file_path) => file_path,
                Err(reason) => {
                    ok = false;
                    actions.push(suggestion_action(item, suggestion, "blocked", &reason));
                    continue;
                }
            };
            let Some(line) = item.line else {
                ok = false;
                actions.push(suggestion_action(
                    item,
                    suggestion,
                    "blocked",
                    "suggestion has no line number",
                ));
                continue;
            };
            let Some(original) = &suggestion.original else {
                ok = false;
                actions.push(suggestion_action(
                    item,
                    suggestion,
                    "blocked",
                    "suggestion has no exact original hunk",
                ));
                continue;
            };
            let original_lines = exact_hunk_lines(original);
            let Some(start_index) = suggestion_start_index(line, original_lines.len()) else {
                ok = false;
                actions.push(suggestion_action(
                    item,
                    suggestion,
                    "blocked",
                    "line number cannot cover the original hunk",
                ));
                continue;
            };
            pending.push(PendingSuggestion {
                item_index,
                suggestion_index,
                file_path,
                start_index,
                original_lines,
            });
        }
    }
    if args.apply {
        pending.sort_by(|left, right| {
            left.file_path
                .cmp(&right.file_path)
                .then_with(|| right.start_index.cmp(&left.start_index))
                .then_with(|| left.item_index.cmp(&right.item_index))
                .then_with(|| left.suggestion_index.cmp(&right.suggestion_index))
        });
    }
    for pending in pending {
        let item = &worklist.items[pending.item_index];
        let suggestion = &item.suggestions[pending.suggestion_index];
        let contents = fs::read_to_string(&pending.file_path)
            .with_context(|| format!("failed to read {}", pending.file_path.display()))?;
        let mut lines = contents.lines().map(str::to_string).collect::<Vec<_>>();
        let end_index = pending.start_index + pending.original_lines.len();
        if end_index > lines.len() {
            ok = false;
            actions.push(suggestion_action(
                item,
                suggestion,
                "blocked",
                "suggestion hunk is outside the current file",
            ));
            continue;
        }
        if !suggestion_hunk_matches(
            &lines[pending.start_index..end_index],
            &pending.original_lines,
        ) {
            ok = false;
            actions.push(suggestion_action(
                item,
                suggestion,
                "blocked",
                "current hunk no longer matches suggestion hunk",
            ));
            continue;
        }
        if args.apply {
            let replacement = replacement_lines(&suggestion.replacement);
            lines.splice(pending.start_index..end_index, replacement);
            let newline = newline_sequence(&contents);
            let mut updated = lines.join(newline);
            if contents.ends_with('\n') {
                updated.push_str(newline);
            }
            fs::write(&pending.file_path, updated)
                .with_context(|| format!("failed to write {}", pending.file_path.display()))?;
            actions.push(suggestion_action(
                item,
                suggestion,
                "applied",
                "exact hunk matched and file was updated",
            ));
        } else {
            actions.push(suggestion_action(
                item,
                suggestion,
                "planned",
                "exact hunk matches and can be applied with --apply",
            ));
        }
    }
    Ok(json!({
        "schema": "codex-dev.pr-review-suggestion-apply.v1",
        "dryRun": !args.apply,
        "ok": ok,
        "actions": actions,
    }))
}

fn suggestion_item_selected(item: &PrReviewWorkItem, selected: Option<&str>) -> bool {
    match selected {
        Some(selected) => item.id == selected,
        None => item.status == "actionable",
    }
}

#[derive(Clone, Debug)]
struct PendingSuggestion {
    item_index: usize,
    suggestion_index: usize,
    file_path: PathBuf,
    start_index: usize,
    original_lines: Vec<String>,
}

fn resolve_suggestion_file_path(
    repo_root: &Path,
    path: &str,
) -> std::result::Result<PathBuf, String> {
    if path.trim().is_empty() {
        return Err("suggestion path is empty".to_string());
    }
    let relative = Path::new(path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err("suggestion path must be relative and stay inside repo root".to_string());
    }
    let file_path = repo_root.join(relative);
    let canonical = fs::canonicalize(&file_path)
        .map_err(|error| format!("failed to resolve suggestion path inside repo root: {error}"))?;
    if !canonical.starts_with(repo_root) {
        return Err("suggestion path resolves outside repo root".to_string());
    }
    Ok(canonical)
}

fn suggestion_start_index(end_line: u64, original_line_count: usize) -> Option<usize> {
    if original_line_count == 0 {
        return None;
    }
    let end_line = usize::try_from(end_line).ok()?;
    end_line.checked_sub(original_line_count)
}

fn suggestion_hunk_matches(current: &[String], original: &[String]) -> bool {
    current == original
}

fn exact_hunk_lines(value: &str) -> Vec<String> {
    value.split('\n').map(str::to_string).collect()
}

fn replacement_lines(replacement: &str) -> Vec<String> {
    if replacement.is_empty() {
        Vec::new()
    } else {
        exact_hunk_lines(replacement)
    }
}

fn newline_sequence(contents: &str) -> &'static str {
    if contents.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn suggestion_action(
    item: &PrReviewWorkItem,
    suggestion: &PrReviewSuggestion,
    status: &str,
    reason: &str,
) -> Value {
    json!({
        "itemId": item.id,
        "threadId": item.thread_id,
        "suggestionId": suggestion.id,
        "path": item.path,
        "line": item.line,
        "status": status,
        "reason": reason,
    })
}

fn plan_or_apply_closeout(
    args: PrReviewCloseoutArgs,
    generated_at: DateTime<Utc>,
) -> Result<PrReviewCloseoutReport> {
    if args.apply && args.source_dir.is_some() {
        bail!(
            "--source-dir is only allowed for dry-run closeout planning; --apply must capture live state"
        );
    }
    let worklist = match &args.worklist {
        Some(path) => Some(read_json::<PrReviewWorklist>(path)?),
        None => None,
    };
    if args.apply && (args.repo.is_none() || args.number.is_none()) {
        bail!("closeout --apply requires explicit --repo and --number");
    }
    let repo = args
        .repo
        .clone()
        .or_else(|| {
            worklist
                .as_ref()
                .map(|worklist| worklist.repository.clone())
        })
        .ok_or_else(|| anyhow::anyhow!("closeout requires --repo or --worklist"))?;
    let number = args
        .number
        .or_else(|| worklist.as_ref().map(|worklist| worklist.number))
        .ok_or_else(|| anyhow::anyhow!("closeout requires --number or --worklist"))?;
    parse_github_repository(&repo)?;
    let expected_head_sha = args
        .expected_head_sha
        .clone()
        .filter(|sha| !sha.trim().is_empty());
    let validation_commands = args
        .validation_commands
        .iter()
        .filter(|command| !command.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    let target_threads = closeout_targets(&args, worklist.as_ref())?;
    let early_diagnostics = closeout_precondition_diagnostics(
        args.apply,
        expected_head_sha.as_deref(),
        None,
        !validation_commands.is_empty(),
    );
    if args.apply
        && early_diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("requires --expected-head-sha")
                || diagnostic.contains("requires at least one --validation-command")
        })
    {
        return Ok(closeout_blocked_report(CloseoutBlockedReportArgs {
            repo: &repo,
            number,
            generated_at,
            apply: args.apply,
            expected_head_sha,
            current_head_sha: None,
            targets: target_threads,
            validation_command: validation_commands.first().cloned(),
            diagnostics: early_diagnostics,
        }));
    }
    let capsule = if args.apply || args.source_dir.is_some() {
        Some(resolve_pr_review_capsule(
            args.capsule.as_deref(),
            &repo,
            number,
            generated_at,
        )?)
    } else {
        args.capsule.clone()
    };
    let current_state = if args.apply || args.source_dir.is_some() {
        Some(run_pr_agent_state(
            PrAgentArgs {
                capsule: capsule.expect("capsule resolved"),
                repo: repo.clone(),
                number,
                checked_at: Some(generated_at),
                source_dir: args.source_dir.clone(),
            },
            generated_at,
        )?)
    } else {
        None
    };
    let current_head_sha = current_state
        .as_ref()
        .and_then(|state| state.pr.head_sha.clone());
    let fresh_threads_checked = current_state.is_some();
    let mut diagnostics = closeout_precondition_diagnostics(
        args.apply,
        expected_head_sha.as_deref(),
        current_head_sha.as_deref(),
        !validation_commands.is_empty(),
    );
    let current_unresolved = if let Some(state) = current_state.as_ref() {
        match current_unresolved_threads(state) {
            Ok(threads) => threads,
            Err(error) => {
                if args.apply {
                    diagnostics.push(error.to_string());
                    BTreeMap::new()
                } else {
                    target_threads
                        .iter()
                        .map(|target| {
                            (
                                target.thread_id.clone(),
                                CurrentThreadComments {
                                    fingerprints: BTreeSet::new(),
                                    count: 0,
                                },
                            )
                        })
                        .collect()
                }
            }
        }
    } else {
        target_threads
            .iter()
            .map(|target| {
                (
                    target.thread_id.clone(),
                    CurrentThreadComments {
                        fingerprints: BTreeSet::new(),
                        count: 0,
                    },
                )
            })
            .collect()
    };
    let mut threads = Vec::new();
    for (index, target) in target_threads.into_iter().enumerate() {
        let command = graph_ql_thread_command(&target.thread_id, RESOLVE_REVIEW_THREAD_MUTATION);
        let commit_sha = args
            .commit_shas
            .get(index)
            .cloned()
            .or_else(|| args.commit_shas.first().cloned())
            .or_else(|| expected_head_sha.clone());
        let validation_command = validation_commands
            .get(index)
            .cloned()
            .or_else(|| validation_commands.first().cloned());
        if !diagnostics.is_empty() {
            threads.push(PrReviewCloseoutThread {
                thread_id: target.thread_id,
                work_item_id: target.work_item_id,
                status: "blocked".to_string(),
                reason: "fresh PR state did not satisfy closeout preconditions".to_string(),
                commit_sha,
                validation_command,
                command,
            });
        } else if let Some(current_comments) = current_unresolved.get(&target.thread_id) {
            if fresh_threads_checked
                && (target
                    .expected_comments
                    .as_ref()
                    .is_some_and(|expected_comments| {
                        !review_comment_fingerprints_match(
                            expected_comments,
                            &current_comments.fingerprints,
                        )
                    })
                    || target
                        .expected_comment_count
                        .is_some_and(|expected_count| expected_count != current_comments.count))
            {
                threads.push(PrReviewCloseoutThread {
                    thread_id: target.thread_id,
                    work_item_id: target.work_item_id,
                    status: "blocked".to_string(),
                    reason: "thread comments changed since worklist capture".to_string(),
                    commit_sha,
                    validation_command,
                    command,
                });
            } else if args.apply {
                let output = run_hosted_command(&command)?;
                let applied = output.exit_code == Some(0);
                threads.push(PrReviewCloseoutThread {
                    thread_id: target.thread_id,
                    work_item_id: target.work_item_id,
                    status: if applied { "applied" } else { "blocked" }.to_string(),
                    reason: if applied {
                        "resolveReviewThread completed".to_string()
                    } else {
                        output
                            .stderr
                            .unwrap_or_else(|| "resolveReviewThread failed".to_string())
                    },
                    commit_sha,
                    validation_command,
                    command,
                });
            } else {
                threads.push(PrReviewCloseoutThread {
                    thread_id: target.thread_id,
                    work_item_id: target.work_item_id,
                    status: "planned".to_string(),
                    reason: "thread will resolve after --apply revalidates fresh PR state"
                        .to_string(),
                    commit_sha,
                    validation_command,
                    command,
                });
            }
        } else {
            threads.push(PrReviewCloseoutThread {
                thread_id: target.thread_id,
                work_item_id: target.work_item_id,
                status: "skipped".to_string(),
                reason: "thread is not currently unresolved".to_string(),
                commit_sha,
                validation_command,
                command,
            });
        }
    }
    let summary = PrReviewCloseoutSummary {
        planned: threads
            .iter()
            .filter(|thread| thread.status == "planned")
            .count() as u64,
        applied: threads
            .iter()
            .filter(|thread| thread.status == "applied")
            .count() as u64,
        skipped: threads
            .iter()
            .filter(|thread| thread.status == "skipped")
            .count() as u64,
        blocked: threads
            .iter()
            .filter(|thread| thread.status == "blocked")
            .count() as u64,
    };
    let ok = summary.blocked == 0 && diagnostics.is_empty();
    Ok(PrReviewCloseoutReport {
        schema: PR_REVIEW_CLOSEOUT_SCHEMA.to_string(),
        repository: repo,
        number,
        generated_at,
        dry_run: !args.apply,
        apply_requested: args.apply,
        expected_head_sha,
        current_head_sha,
        ok,
        summary,
        threads,
        diagnostics,
    })
}

struct CloseoutBlockedReportArgs<'a> {
    repo: &'a str,
    number: u64,
    generated_at: DateTime<Utc>,
    apply: bool,
    expected_head_sha: Option<String>,
    current_head_sha: Option<String>,
    targets: Vec<CloseoutTarget>,
    validation_command: Option<String>,
    diagnostics: Vec<String>,
}

fn closeout_blocked_report(args: CloseoutBlockedReportArgs<'_>) -> PrReviewCloseoutReport {
    let threads = args
        .targets
        .into_iter()
        .map(|target| PrReviewCloseoutThread {
            thread_id: target.thread_id,
            work_item_id: target.work_item_id,
            status: "blocked".to_string(),
            reason: "closeout apply preconditions were not satisfied".to_string(),
            commit_sha: None,
            validation_command: args.validation_command.clone(),
            command: Vec::new(),
        })
        .collect::<Vec<_>>();

    PrReviewCloseoutReport {
        schema: PR_REVIEW_CLOSEOUT_SCHEMA.to_string(),
        repository: args.repo.to_string(),
        number: args.number,
        generated_at: args.generated_at,
        dry_run: !args.apply,
        apply_requested: args.apply,
        expected_head_sha: args.expected_head_sha,
        current_head_sha: args.current_head_sha,
        ok: false,
        summary: PrReviewCloseoutSummary {
            planned: 0,
            applied: 0,
            skipped: 0,
            blocked: threads.len() as u64,
        },
        threads,
        diagnostics: args.diagnostics,
    }
}

fn closeout_precondition_diagnostics(
    apply: bool,
    expected_head_sha: Option<&str>,
    current_head_sha: Option<&str>,
    has_validation_command: bool,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if apply && expected_head_sha.is_none() {
        diagnostics.push("closeout apply requires --expected-head-sha".to_string());
    }
    if apply && !has_validation_command {
        diagnostics.push(
            "closeout apply requires at least one --validation-command with passed-gate evidence"
                .to_string(),
        );
    }
    if let Some(expected) = expected_head_sha {
        match current_head_sha {
            Some(current) if current == expected => {}
            Some(current) => diagnostics.push(format!(
                "fresh PR head {current} does not match expected fixed head {expected}"
            )),
            None if apply => diagnostics.push(format!(
                "fresh PR head was unavailable; expected fixed head {expected}"
            )),
            None => {}
        }
    }
    diagnostics
}

#[derive(Clone, Debug)]
struct CloseoutTarget {
    thread_id: String,
    work_item_id: String,
    expected_comments: Option<BTreeSet<ReviewCommentFingerprint>>,
    expected_comment_count: Option<u64>,
}

fn closeout_targets(
    args: &PrReviewCloseoutArgs,
    worklist: Option<&PrReviewWorklist>,
) -> Result<Vec<CloseoutTarget>> {
    let Some(worklist) = worklist else {
        if args.thread_ids.is_empty() {
            bail!("closeout requires --thread-id when --worklist is omitted");
        }
        return Ok(args
            .thread_ids
            .iter()
            .enumerate()
            .map(|(index, thread_id)| CloseoutTarget {
                thread_id: thread_id.clone(),
                work_item_id: format!("manual-{:03}", index + 1),
                expected_comments: None,
                expected_comment_count: None,
            })
            .collect());
    };

    let selected_thread_ids = (!args.thread_ids.is_empty())
        .then(|| args.thread_ids.iter().cloned().collect::<BTreeSet<_>>());
    let mut targets = BTreeMap::<String, CloseoutTarget>::new();
    for item in worklist
        .items
        .iter()
        .filter(|item| item.status == "actionable")
    {
        if let Some(selected_thread_ids) = &selected_thread_ids
            && !selected_thread_ids.contains(&item.thread_id)
        {
            continue;
        }
        let target = targets
            .entry(item.thread_id.clone())
            .or_insert_with(|| CloseoutTarget {
                thread_id: item.thread_id.clone(),
                work_item_id: item.id.clone(),
                expected_comments: Some(BTreeSet::new()),
                expected_comment_count: Some(0),
            });
        target.work_item_id = item.id.clone();
        if let Some(fingerprint) = work_item_comment_fingerprint(item)
            && let Some(expected_comments) = &mut target.expected_comments
        {
            expected_comments.insert(fingerprint);
            target.expected_comment_count = target
                .expected_comment_count
                .map(|count| count.saturating_add(1));
        }
    }
    if let Some(selected_thread_ids) = selected_thread_ids {
        let missing = selected_thread_ids
            .iter()
            .filter(|thread_id| !targets.contains_key(thread_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            bail!(
                "--thread-id values were not found in actionable worklist items: {}",
                missing.join(", ")
            );
        }
    }
    Ok(targets.into_values().collect())
}

struct CurrentThreadComments {
    fingerprints: BTreeSet<ReviewCommentFingerprint>,
    count: u64,
}

fn current_unresolved_threads(
    state: &codex_dev_core::PrAgentStateReport,
) -> Result<BTreeMap<String, CurrentThreadComments>> {
    let Some(source) = state
        .sources
        .iter()
        .find(|source| source.id == "gh-review-threads")
    else {
        bail!("fresh review-thread state was not captured");
    };
    if source.status != PrAgentSourceStatus::Captured {
        bail!(
            "fresh review-thread state capture failed for gh-review-threads with status {:?}",
            source.status
        );
    }
    if !state.pr.review_threads.authoritative {
        bail!("fresh review-thread state is not authoritative");
    }
    let path = PathBuf::from(&source.path);
    let value = read_json::<Value>(&path).with_context(|| {
        format!(
            "failed to read fresh review-thread state {}",
            path.display()
        )
    })?;
    let parsed_threads = github_review_threads_from_graphql(&value)?;
    if !parsed_threads.is_complete() {
        bail!("fresh review-thread state is not authoritative");
    }
    let threads = parsed_threads.threads;
    let incomplete_threads = threads
        .iter()
        .filter(|thread| !thread.is_resolved && !thread.is_outdated && !thread.comments_complete())
        .map(|thread| thread.id.as_str())
        .collect::<Vec<_>>();
    if !incomplete_threads.is_empty() {
        bail!(
            "fresh review-thread comments were incomplete for {}",
            incomplete_threads.join(", ")
        );
    }
    Ok(threads
        .into_iter()
        .filter(|thread| !thread.is_resolved && !thread.is_outdated)
        .map(|thread| {
            let count = thread.comment_count();
            let fingerprints = thread
                .comments
                .iter()
                .filter_map(review_comment_fingerprint)
                .collect();
            (
                thread.id,
                CurrentThreadComments {
                    fingerprints,
                    count,
                },
            )
        })
        .collect())
}

fn work_item_comment_fingerprint(item: &PrReviewWorkItem) -> Option<ReviewCommentFingerprint> {
    if item.comment_id.is_none()
        && item.author.is_none()
        && item.path.is_none()
        && item.line.is_none()
        && item.body_excerpt == "Unresolved GitHub review thread"
    {
        return None;
    }
    Some(ReviewCommentFingerprint {
        id: item.comment_id.clone(),
        author: item.author.clone(),
        path: item.path.clone(),
        line: item.line,
        body_excerpt: item.body_excerpt.clone(),
        body_hash: item.body_hash.clone(),
    })
}

fn review_comment_fingerprint(comment: &GitHubReviewComment) -> Option<ReviewCommentFingerprint> {
    let body = comment
        .body
        .as_deref()
        .unwrap_or("Unresolved GitHub review thread");
    let body_excerpt = excerpt(body, 420);
    if comment.id.is_none()
        && comment.author.is_none()
        && comment.path.is_none()
        && comment.line.is_none()
        && body_excerpt == "Unresolved GitHub review thread"
    {
        return None;
    }
    Some(ReviewCommentFingerprint {
        id: comment.id.clone(),
        author: comment.author.clone(),
        path: comment.path.clone(),
        line: comment.line,
        body_excerpt,
        body_hash: Some(stable_text_hash(body)),
    })
}

fn review_comment_fingerprints_match(
    expected: &BTreeSet<ReviewCommentFingerprint>,
    current: &BTreeSet<ReviewCommentFingerprint>,
) -> bool {
    expected.iter().all(|expected_comment| {
        current.iter().any(|current_comment| {
            review_comment_fingerprint_matches(expected_comment, current_comment)
        })
    })
}

fn review_comment_fingerprint_matches(
    expected: &ReviewCommentFingerprint,
    current: &ReviewCommentFingerprint,
) -> bool {
    expected.id == current.id
        && expected.author == current.author
        && expected.path == current.path
        && expected.line == current.line
        && expected.body_excerpt == current.body_excerpt
        && expected
            .body_hash
            .as_ref()
            .is_none_or(|expected_hash| current.body_hash.as_ref() == Some(expected_hash))
}

fn ingest_local_review(
    source: &Path,
    kind: LocalReviewKind,
    checked_at: DateTime<Utc>,
) -> Result<LocalReviewWorklist> {
    let text = fs::read_to_string(source)
        .with_context(|| format!("failed to read {}", source.display()))?;
    let mut items = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.eq_ignore_ascii_case("findings")
            || trimmed.eq_ignore_ascii_case("summary")
        {
            continue;
        }
        let actionable = looks_like_review_item(trimmed);
        if actionable {
            items.push(LocalReviewWorkItem {
                id: format!("local-{:03}", items.len() + 1),
                source_line: (index + 1) as u64,
                status: "actionable".to_string(),
                body_excerpt: excerpt(trimmed.trim_start_matches(['-', '*', ' ']), 420),
                path: extract_path_hint(trimmed),
            });
        }
    }
    Ok(LocalReviewWorklist {
        schema: LOCAL_REVIEW_WORKLIST_SCHEMA.to_string(),
        source: source.display().to_string(),
        kind: match kind {
            LocalReviewKind::Codex => "codex",
            LocalReviewKind::Zen => "zen",
            LocalReviewKind::Manual => "manual",
        }
        .to_string(),
        checked_at,
        summary: LocalReviewWorklistSummary {
            items: items.len() as u64,
            actionable_items: items.len() as u64,
        },
        items,
    })
}

fn looks_like_review_item(line: &str) -> bool {
    line.starts_with('-')
        || line.starts_with('*')
        || line.contains(".rs:")
        || line.contains(".ts:")
        || line.contains(".tsx:")
        || line.to_ascii_lowercase().contains("fix")
}

fn extract_path_hint(line: &str) -> Option<String> {
    line.split_whitespace()
        .filter_map(normalize_path_hint_token)
        .find(|path| path.contains('/'))
}

fn normalize_path_hint_token(token: &str) -> Option<String> {
    let token = token.trim_matches(|ch: char| matches!(ch, '`' | '\'' | '"' | ',' | ';'));
    let path_end = token
        .char_indices()
        .filter_map(|(index, ch)| {
            if ch != '.' {
                return None;
            }
            let end = token[index + 1..]
                .char_indices()
                .take_while(|(_, ch)| ch.is_ascii_alphanumeric() || *ch == '_')
                .take(5)
                .map(|(offset, ch)| index + 1 + offset + ch.len_utf8())
                .last()?;
            (end > index + 1).then_some(end)
        })
        .max()?;
    Some(token[..path_end].to_string())
}

fn render_local_review_worklist(worklist: &LocalReviewWorklist) -> String {
    let mut markdown = format!(
        "# Local Review Worklist\n\n- Source: `{}`\n- Kind: {}\n- Items: {}\n\n",
        worklist.source, worklist.kind, worklist.summary.items
    );
    for item in &worklist.items {
        markdown.push_str(&format!(
            "- {} line {} {}{}\n",
            item.id,
            item.source_line,
            item.path
                .as_deref()
                .map(|path| format!("`{path}` "))
                .unwrap_or_default(),
            item.body_excerpt
        ));
    }
    markdown
}

fn local_review_item_matches(item: &LocalReviewWorkItem, args: &LocalReviewQueryArgs) -> bool {
    args.item.as_deref().is_none_or(|id| item.id == id)
        && args
            .file
            .as_deref()
            .is_none_or(|file| item.path.as_deref().is_some_and(|path| path.contains(file)))
        && args.text.as_deref().is_none_or(|text| {
            item.body_excerpt
                .to_ascii_lowercase()
                .contains(&text.to_ascii_lowercase())
        })
}

fn build_commit_plan(args: CommitPlanArgs, checked_at: DateTime<Utc>) -> Result<CommitPlan> {
    let repo_root = args.repo_root;
    let files = git_status_files(&repo_root)?;
    let staged_files = files.iter().filter(|file| file.staged).count() as u64;
    let work_items = match args.worklist {
        Some(path) => work_items_by_file(&read_json::<PrReviewWorklist>(&path)?),
        None => BTreeMap::new(),
    };
    let mut grouped = BTreeMap::<String, Vec<String>>::new();
    for file in &files {
        grouped
            .entry(scope_for_file(&file.path))
            .or_default()
            .push(file.path.clone());
    }
    let mut groups = Vec::new();
    for (index, (scope, mut paths)) in grouped.into_iter().enumerate() {
        paths.sort();
        paths.dedup();
        let commit_type = commit_type_for_files(&paths).to_string();
        let subject = subject_for_group(&commit_type, &scope, &paths);
        let semver_impact = semver_impact_for_type(&commit_type, false).to_string();
        let source_work_items = paths
            .iter()
            .flat_map(|path| work_items.get(path).into_iter().flatten().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        groups.push(CommitPlanGroup {
            id: format!("commit-{:03}", index + 1),
            commit_type,
            scope,
            subject,
            semver_impact,
            files: paths.clone(),
            source_work_items,
            validation_commands: validation_commands_for_files(&paths),
        });
    }
    let diagnostics = if files.is_empty() {
        vec!["no changed files found".to_string()]
    } else {
        Vec::new()
    };
    Ok(CommitPlan {
        schema: COMMIT_PLAN_SCHEMA.to_string(),
        checked_at,
        repo_root: repo_root.display().to_string(),
        ok: diagnostics.is_empty(),
        summary: CommitPlanSummary {
            changed_files: files.len() as u64,
            staged_files,
            groups: groups.len() as u64,
        },
        groups,
        diagnostics,
    })
}

#[derive(Clone, Debug)]
struct GitStatusFile {
    path: String,
    staged: bool,
}

fn git_status_files(repo_root: &Path) -> Result<Vec<GitStatusFile>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["status", "--porcelain=v1", "-z", "--untracked-files=all"])
        .output()
        .with_context(|| format!("failed to run git status in {}", repo_root.display()))?;
    if !output.status.success() {
        bail!("git status failed in {}", repo_root.display());
    }
    let mut files = Vec::new();
    let mut entries = output.stdout.split(|byte| *byte == b'\0');
    while let Some(entry) = entries.next() {
        if entry.is_empty() {
            continue;
        }
        let record =
            std::str::from_utf8(entry).context("git status returned non-UTF8 path data")?;
        if record.len() < 4 {
            continue;
        }
        let xy = &record[..2];
        let path = record[3..].to_string();
        if xy
            .as_bytes()
            .iter()
            .any(|status| matches!(status, b'R' | b'C'))
        {
            let _old_path = entries.next();
        }
        files.push(GitStatusFile {
            path,
            staged: !xy.starts_with(' ') && !xy.starts_with("??"),
        });
    }
    Ok(files)
}

fn work_items_by_file(worklist: &PrReviewWorklist) -> BTreeMap<String, Vec<String>> {
    let mut by_file = BTreeMap::<String, Vec<String>>::new();
    for item in &worklist.items {
        if let Some(path) = &item.path {
            by_file
                .entry(path.clone())
                .or_default()
                .push(item.id.clone());
        }
    }
    by_file
}

fn scope_for_file(path: &str) -> String {
    if let Some(skill) = path
        .strip_prefix("skills/")
        .and_then(|rest| rest.split('/').next())
    {
        return skill.to_string();
    }
    if let Some(crate_name) = path
        .strip_prefix("crates/")
        .and_then(|rest| rest.split('/').next())
    {
        return crate_name.to_string();
    }
    if let Some(plugin) = path
        .strip_prefix("plugins/")
        .and_then(|rest| rest.split('/').next())
    {
        return plugin.to_string();
    }
    if path.starts_with("docs/") {
        return "docs".to_string();
    }
    if path == "README.md" || path == "AGENTS.md" {
        return "docs".to_string();
    }
    let mut parts = path.split('/');
    let first = parts.next().unwrap_or("repo");
    if parts.next().is_none() || first.starts_with('.') {
        return "repo".to_string();
    }
    normalized_commit_scope(first)
}

fn normalized_commit_scope(segment: &str) -> String {
    let mut scope = String::new();
    let mut last_was_dash = false;
    for ch in segment.chars() {
        let next = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '-'
        };
        if next == '-' {
            if !scope.is_empty() && !last_was_dash {
                scope.push(next);
            }
            last_was_dash = true;
        } else {
            scope.push(next);
            last_was_dash = false;
        }
    }
    while scope.ends_with('-') {
        scope.pop();
    }
    if scope.is_empty() {
        "repo".to_string()
    } else {
        scope
    }
}

fn commit_type_for_files(paths: &[String]) -> &'static str {
    if paths
        .iter()
        .all(|path| path.ends_with(".md") || path == "README.md" || path == "AGENTS.md")
    {
        "docs"
    } else if paths
        .iter()
        .any(|path| path.ends_with("Cargo.toml") || path.ends_with("Cargo.lock"))
    {
        "build"
    } else if paths.iter().any(|path| {
        path.contains("src/") || path.starts_with("skills/") || path.starts_with("plugins/")
    }) {
        "feat"
    } else if paths
        .iter()
        .any(|path| path.contains("/tests/") || path.ends_with("_test.rs"))
    {
        "test"
    } else {
        "chore"
    }
}

fn subject_for_group(commit_type: &str, scope: &str, paths: &[String]) -> String {
    let behavior = if paths.iter().any(|path| path.contains("gh-pr-review-fix")) {
        "hard-cut PR review remediation workflow"
    } else if paths.iter().any(|path| path.contains("review")) {
        "add review remediation workflow"
    } else if paths.iter().any(|path| path.contains("commit")) {
        "add semantic commit validation"
    } else if commit_type == "docs" {
        "document canonical workflow"
    } else {
        "update development workflow"
    };
    format!("{commit_type}({scope}): {behavior}")
}

fn validation_commands_for_files(paths: &[String]) -> Vec<String> {
    let mut commands = BTreeSet::new();
    if paths
        .iter()
        .any(|path| path.starts_with("crates/") && path.ends_with(".rs"))
    {
        commands.insert("cargo fmt --all --check".to_string());
        commands.insert("cargo clippy --all-targets -- -D warnings".to_string());
    }
    if paths
        .iter()
        .any(|path| path.starts_with("crates/codex-dev"))
    {
        commands.insert("cargo test -p codex-dev".to_string());
    }
    if paths
        .iter()
        .any(|path| path.starts_with("crates/codex-dev-core"))
    {
        commands.insert("cargo test -p codex-dev-core".to_string());
    }
    if paths.iter().any(|path| is_skill_path(path)) {
        for skill_path in changed_skill_roots(paths) {
            commands.insert(format!(
                "python3 tools/skill/quick_validate.py {skill_path}"
            ));
        }
    }
    for script_path in changed_motion_plugin_scripts(paths) {
        commands.insert(format!("node --check {script_path}"));
    }
    if paths
        .iter()
        .any(|path| path.starts_with("plugins/native-motion/scripts/"))
    {
        commands
            .insert("node plugins/native-motion/scripts/validate-atomic-skills.mjs".to_string());
    }
    for python_path in changed_eval_python_files(paths) {
        commands.insert(format!("python3 -m py_compile {python_path}"));
    }
    if paths.iter().any(|path| path.ends_with(".md")) {
        commands.insert("python3 tools/docs/check_links.py docs README.md AGENTS.md".to_string());
    }
    commands.insert("git diff --check".to_string());
    commands.into_iter().collect()
}

fn changed_skill_roots(paths: &[String]) -> BTreeSet<String> {
    paths
        .iter()
        .filter_map(|path| {
            let parts = path.split('/').collect::<Vec<_>>();
            if parts.first() == Some(&"skills") && parts.len() >= 2 {
                Some(format!("skills/{}", parts[1]))
            } else if parts.first() == Some(&"plugins")
                && parts.get(2) == Some(&"skills")
                && parts.len() >= 4
            {
                Some(format!("plugins/{}/skills/{}", parts[1], parts[3]))
            } else {
                None
            }
        })
        .collect()
}

fn is_skill_path(path: &str) -> bool {
    path.starts_with("skills/") || (path.starts_with("plugins/") && path.contains("/skills/"))
}

fn changed_motion_plugin_scripts(paths: &[String]) -> BTreeSet<String> {
    paths
        .iter()
        .filter(|path| {
            path.starts_with("plugins/native-motion/scripts/")
                && matches!(
                    Path::new(path)
                        .extension()
                        .and_then(|extension| extension.to_str()),
                    Some("cjs" | "js" | "mjs")
                )
        })
        .cloned()
        .collect()
}

fn changed_eval_python_files(paths: &[String]) -> BTreeSet<String> {
    paths
        .iter()
        .filter(|path| path.starts_with("tools/eval/") && path.ends_with(".py"))
        .cloned()
        .collect()
}

fn validate_commit_subject_report(
    args: &CommitValidateArgs,
    checked_at: DateTime<Utc>,
) -> Result<CommitValidationReport> {
    let (commit_type, scope, description, breaking) = parse_conventional_subject(&args.subject);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    match (&commit_type, &scope, &description) {
        (Some(commit_type), Some(scope), Some(description)) => {
            if !allowed_commit_type(commit_type) {
                errors.push(format!(
                    "unsupported conventional commit type `{commit_type}`"
                ));
            }
            if !valid_scope(scope) {
                errors.push("scope must be a lowercase codebase owner noun using letters, digits, or hyphen".to_string());
            }
            if process_scope(scope) {
                errors.push(format!(
                    "scope `{scope}` describes the review process, not code ownership"
                ));
            }
            validate_subject_description(description, &mut errors, &mut warnings);
        }
        _ => errors.push("subject must match <type>(<scope>): <description>".to_string()),
    }
    if args.require_staged
        && git_status_files(&args.repo_root)?
            .iter()
            .all(|file| !file.staged)
    {
        errors.push("--require-staged was set but no staged files were found".to_string());
    }
    let semver_impact = commit_type
        .as_deref()
        .map(|commit_type| semver_impact_for_type(commit_type, breaking).to_string());
    Ok(CommitValidationReport {
        schema: COMMIT_VALIDATION_SCHEMA.to_string(),
        checked_at,
        ok: errors.is_empty(),
        subject: args.subject.clone(),
        commit_type,
        scope,
        semver_impact,
        errors,
        warnings,
    })
}

fn parse_conventional_subject(
    subject: &str,
) -> (Option<String>, Option<String>, Option<String>, bool) {
    let Some((prefix, description)) = subject.split_once(": ") else {
        return (None, None, None, false);
    };
    let breaking = prefix.ends_with('!');
    let prefix = prefix.trim_end_matches('!');
    let Some((commit_type, rest)) = prefix.split_once('(') else {
        return (
            Some(prefix.to_string()),
            None,
            Some(description.to_string()),
            breaking,
        );
    };
    let Some(scope) = rest.strip_suffix(')') else {
        return (
            Some(commit_type.to_string()),
            None,
            Some(description.to_string()),
            breaking,
        );
    };
    (
        Some(commit_type.to_string()),
        Some(scope.to_string()),
        Some(description.to_string()),
        breaking,
    )
}

fn allowed_commit_type(commit_type: &str) -> bool {
    matches!(
        commit_type,
        "feat" | "fix" | "perf" | "refactor" | "docs" | "test" | "chore" | "ci" | "build"
    )
}

fn valid_scope(scope: &str) -> bool {
    !scope.is_empty()
        && scope
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn process_scope(scope: &str) -> bool {
    matches!(
        scope,
        "review" | "feedback" | "comments" | "pr-review" | "codex"
    )
}

fn validate_subject_description(
    description: &str,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    if description.trim().is_empty() {
        errors.push("description must not be empty".to_string());
        return;
    }
    if description.chars().count() > 72 {
        warnings.push("description is longer than 72 characters".to_string());
    }
    let lower = description.to_ascii_lowercase();
    let denied = [
        "address pr review",
        "address review",
        "review feedback",
        "pr review comments",
        "review comments",
        "generated client review feedback",
        "messaging review feedback",
        "fix feedback",
        "handle feedback",
        "apply review",
        "follow-up",
        "misc",
        "wip",
    ];
    if let Some(phrase) = denied.iter().find(|phrase| lower.contains(**phrase)) {
        errors.push(format!(
            "description uses process wording `{phrase}`; describe the code/docs/test behavior instead"
        ));
    }
}

fn semver_impact_for_type(commit_type: &str, breaking: bool) -> &'static str {
    if breaking {
        "major"
    } else {
        match commit_type {
            "feat" => "minor",
            "fix" | "perf" => "patch",
            _ => "none",
        }
    }
}
