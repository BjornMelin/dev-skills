//! Audit a Claude Code configuration estate for the drift classes that
//! accumulate silently: overrides pointing at skills that no longer exist,
//! broken skill symlinks, guides that outgrew their budget, duplicate agent
//! names that the loader silently discards, and skill descriptions that push
//! the always-on listing past its budget.
//!
//! Every check here is something that went wrong in a real estate and was not
//! detected by anything. The tool exists so the next drift is loud.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use walkdir::WalkDir;

/// Anthropic's guidance: keep a CLAUDE.md under 200 lines.
const CLAUDE_MD_MAX_LINES: usize = 200;
/// Skill-authoring guidance: keep a SKILL.md body under 500 lines.
const SKILL_MAX_LINES: usize = 500;
/// A SKILL.md above this size is skipped entirely by the loader.
const SKILL_MAX_BYTES: u64 = 128 * 1024;
/// Frontmatter `description` hard cap.
const DESC_MAX_CHARS: usize = 1024;
/// Descriptions beyond this are listing-cost hogs (~50 tokens).
const DESC_TARGET_CHARS: usize = 280;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
enum Severity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize)]
struct Finding {
    id: &'static str,
    severity: Severity,
    subject: String,
    message: String,
    suggestion: &'static str,
}

#[derive(Debug, Serialize)]
struct Summary {
    total: usize,
    by_severity: BTreeMap<String, usize>,
    listing_tokens: usize,
    skills_listed: usize,
}

#[derive(Debug, Serialize)]
struct Report {
    tool: &'static str,
    version: &'static str,
    summary: Summary,
    findings: Vec<Finding>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    Markdown,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "claude-config-audit",
    about = "Audit a Claude Code configuration estate for drift",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Scan a Claude home (and optionally a project) for configuration drift.
    Scan {
        /// Claude home directory, e.g. ~/.claude
        #[arg(long, default_value = "~/.claude")]
        home: String,
        /// Optional project root containing a .claude directory.
        #[arg(long)]
        project: Option<String>,
        #[arg(long, value_enum, default_value_t = Format::Markdown)]
        format: Format,
        /// Exit non-zero only at or above this severity.
        #[arg(long, value_enum, default_value_t = Severity::Medium)]
        min_severity: Severity,
    },
    /// Print the full rule catalog.
    Doctor,
    /// Generate shell completions.
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

fn expand(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

/// Frontmatter fields this tool cares about.
struct Frontmatter {
    name: Option<String>,
    description: Option<String>,
    disable_model_invocation: bool,
}

fn parse_frontmatter(text: &str) -> Option<Frontmatter> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let block = &rest[..end];
    let mut name = None;
    let mut description = None;
    let mut disable = false;
    let mut lines = block.lines().peekable();
    while let Some(line) = lines.next() {
        if let Some(v) = line.strip_prefix("name:") {
            name = Some(v.trim().trim_matches(['"', '\'']).to_string());
        } else if let Some(v) = line.strip_prefix("disable-model-invocation:") {
            disable = v.trim() == "true";
        } else if let Some(v) = line.strip_prefix("description:") {
            let mut acc = v.trim().to_string();
            // Fold YAML block scalars and indented continuations.
            while let Some(next) = lines.peek() {
                if next.starts_with(' ') || next.starts_with('\t') || next.trim().is_empty() {
                    let n = lines.next().unwrap_or_default();
                    if !n.trim().is_empty() {
                        acc.push(' ');
                        acc.push_str(n.trim());
                    }
                } else {
                    break;
                }
            }
            description = Some(
                acc.trim_matches(['"', '\'', '|', '>', '-'])
                    .trim()
                    .to_string(),
            );
        }
    }
    Some(Frontmatter {
        name,
        description,
        disable_model_invocation: disable,
    })
}

/// Every `SKILL.md` reachable from a skills root, following symlinks.
fn skill_files(root: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let dir = entry.path();
        let skill = dir.join("SKILL.md");
        if skill.exists() {
            let name = entry.file_name().to_string_lossy().to_string();
            out.push((name, skill));
        }
    }
    out.sort();
    out
}

fn read_overrides(settings: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Ok(text) = fs::read_to_string(settings) else {
        return out;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return out;
    };
    if let Some(map) = value.get("skillOverrides").and_then(|v| v.as_object()) {
        for (k, v) in map {
            if let Some(s) = v.as_str() {
                out.insert(k.clone(), s.to_string());
            }
        }
    }
    out
}

#[allow(clippy::too_many_lines)]
fn scan(home: &Path, project: Option<&Path>) -> Result<(Vec<Finding>, usize, usize)> {
    let mut findings = Vec::new();

    // ---- skill roots -------------------------------------------------------
    let home_skills = home.join("skills");
    let mut roots = vec![home_skills.clone()];
    if let Some(p) = project {
        roots.push(p.join(".claude").join("skills"));
    }

    // A broken symlink in a skills root is invisible to the loader.
    for root in &roots {
        if !root.exists() {
            continue;
        }
        for entry in fs::read_dir(root).with_context(|| format!("read {}", root.display()))? {
            let entry = entry?;
            let path = entry.path();
            if path.is_symlink() && fs::metadata(&path).is_err() {
                let target = fs::read_link(&path).unwrap_or_default();
                findings.push(Finding {
                    id: "links.broken-skill-symlink",
                    severity: Severity::High,
                    subject: path.display().to_string(),
                    message: format!("symlink target does not resolve: {}", target.display()),
                    suggestion: "Repoint or remove the link; a broken link is silently skipped.",
                });
            }
        }
    }

    // ---- skills ------------------------------------------------------------
    let mut listed: BTreeMap<String, (usize, bool)> = BTreeMap::new();
    let mut known: BTreeSet<String> = BTreeSet::new();

    for root in &roots {
        for (name, file) in skill_files(root) {
            known.insert(name.clone());
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            let bytes = fs::metadata(&file).map(|m| m.len()).unwrap_or(0);
            let lines = text.lines().count();

            if bytes > SKILL_MAX_BYTES {
                findings.push(Finding {
                    id: "skill.oversized-skipped",
                    severity: Severity::High,
                    subject: name.clone(),
                    message: format!(
                        "SKILL.md is {bytes} bytes, above the {SKILL_MAX_BYTES}-byte loader limit"
                    ),
                    suggestion: "Split into references/; the loader skips this file entirely.",
                });
            } else if lines > SKILL_MAX_LINES {
                findings.push(Finding {
                    id: "skill.body-too-long",
                    severity: Severity::Low,
                    subject: name.clone(),
                    message: format!(
                        "SKILL.md body is {lines} lines, above the {SKILL_MAX_LINES}-line guidance"
                    ),
                    suggestion: "Move detail into references/ and keep the entrypoint a router.",
                });
            }

            let Some(fm) = parse_frontmatter(&text) else {
                findings.push(Finding {
                    id: "skill.missing-frontmatter",
                    severity: Severity::High,
                    subject: name.clone(),
                    message: "no parseable YAML frontmatter".to_string(),
                    suggestion: "Add a --- delimited block with name and description.",
                });
                continue;
            };

            if let Some(decl) = &fm.name
                && decl != &name
            {
                findings.push(Finding {
                    id: "skill.name-mismatch",
                    severity: Severity::Medium,
                    subject: name.clone(),
                    message: format!("frontmatter name is `{decl}` but the directory is `{name}`"),
                    suggestion: "Identity comes from frontmatter; align them to avoid confusion.",
                });
            }

            // Instructions that ask the model to expose its reasoning trigger
            // Fable 5's reasoning_extraction refusal and fall back to Opus 4.8.
            let lowered = text.to_lowercase();
            for probe in [
                "show your reasoning",
                "explain your reasoning",
                "reproduce its reasoning",
                "transcribe your thinking",
                "echo your reasoning",
            ] {
                if lowered.contains(probe) {
                    findings.push(Finding {
                        id: "model.reasoning-extraction-risk",
                        severity: Severity::Medium,
                        subject: name.clone(),
                        message: format!("body contains \"{probe}\""),
                        suggestion: "Fable 5 refuses reasoning extraction and falls back to Opus 4.8. Ask for conclusions, not reasoning.",
                    });
                    break;
                }
            }

            if fm.disable_model_invocation {
                continue; // excluded from the listing; costs nothing
            }
            let desc = fm.description.unwrap_or_default();
            if desc.len() > DESC_MAX_CHARS {
                findings.push(Finding {
                    id: "skill.description-over-cap",
                    severity: Severity::High,
                    subject: name.clone(),
                    message: format!(
                        "description is {} chars, above the {DESC_MAX_CHARS} cap",
                        desc.len()
                    ),
                    suggestion: "Trim; the frontmatter cap is enforced by the loader.",
                });
            } else if desc.len() > DESC_TARGET_CHARS {
                findings.push(Finding {
                    id: "skill.description-listing-hog",
                    severity: Severity::Low,
                    subject: name.clone(),
                    message: format!("description is {} chars (~{} tokens), above the ~{DESC_TARGET_CHARS}-char target", desc.len(), desc.len() / 4),
                    suggestion: "Lead with the key use case and keep concrete trigger phrases.",
                });
            }
            listed.insert(name.clone(), (name.len() + desc.len() + 20, true));
        }
    }

    // Plugin marketplaces contribute skills too. They are not audited for size
    // (they are upstream), but their names must be known so overrides that
    // target them are not misreported as stale.
    let marketplaces = home.join("plugins").join("marketplaces");
    if marketplaces.exists() {
        for entry in WalkDir::new(&marketplaces)
            .max_depth(6)
            .into_iter()
            .flatten()
        {
            if entry.file_name() != "SKILL.md" {
                continue;
            }
            if let Some(dir) = entry.path().parent()
                && let Some(name) = dir.file_name()
            {
                known.insert(name.to_string_lossy().to_string());
            }
        }
    }

    // ---- overrides ---------------------------------------------------------
    let mut settings_files = vec![home.join("settings.json"), home.join("settings.local.json")];
    if let Some(p) = project {
        settings_files.push(p.join(".claude").join("settings.json"));
        settings_files.push(p.join(".claude").join("settings.local.json"));
    }
    for settings in &settings_files {
        for (skill, mode) in read_overrides(settings) {
            // Plugin-namespaced entries (`plugin:skill`) are not resolvable here.
            if skill.contains(':') || known.contains(&skill) {
                continue;
            }
            findings.push(Finding {
                id: "overrides.stale",
                severity: Severity::Medium,
                subject: format!("{skill} = {mode}"),
                message: format!(
                    "override in {} targets a skill that does not exist",
                    settings.display()
                ),
                suggestion: "Delete the entry; it silently does nothing and hides real intent.",
            });
        }
    }

    // ---- agents ------------------------------------------------------------
    let mut agent_names: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut agent_dirs = vec![home.join("agents")];
    if let Some(p) = project {
        agent_dirs.push(p.join(".claude").join("agents"));
    }
    for dir in &agent_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(dir)?.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "md") {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let Some(fm) = parse_frontmatter(&text) else {
                continue;
            };
            let Some(name) = fm.name else { continue };
            agent_names
                .entry(name)
                .or_default()
                .push(path.display().to_string());
            if let Some(d) = fm.description
                && d.len() > DESC_MAX_CHARS
            {
                findings.push(Finding {
                    id: "agent.description-bloat",
                    severity: Severity::Low,
                    subject: path.display().to_string(),
                    message: format!(
                        "agent description is {} chars (~{} tokens), always in the system prompt",
                        d.len(),
                        d.len() / 4
                    ),
                    suggestion: "Strip <example> blocks; state when to delegate in two sentences.",
                });
            }
        }
    }
    for (name, paths) in agent_names {
        if paths.len() > 1 {
            findings.push(Finding {
                id: "agent.duplicate-name",
                severity: Severity::High,
                subject: name,
                message: format!("declared by {} files: {}", paths.len(), paths.join(", ")),
                suggestion: "The loader keeps one and silently discards the rest. Rename or remove.",
            });
        }
    }

    // ---- guides ------------------------------------------------------------
    let mut guides = vec![home.join("CLAUDE.md")];
    if let Some(p) = project {
        for entry in WalkDir::new(p)
            .max_depth(3)
            .into_iter()
            .filter_entry(|e| {
                let n = e.file_name().to_string_lossy();
                n != "node_modules" && n != ".git" && n != "target"
            })
            .flatten()
        {
            if entry.file_name() == "CLAUDE.md" {
                guides.push(entry.path().to_path_buf());
            }
        }
    }
    for guide in guides {
        let Ok(text) = fs::read_to_string(&guide) else {
            continue;
        };
        let lines = text.lines().count();
        if lines > CLAUDE_MD_MAX_LINES {
            findings.push(Finding {
                id: "guide.over-line-budget",
                severity: Severity::Low,
                subject: guide.display().to_string(),
                message: format!("{lines} lines, above the {CLAUDE_MD_MAX_LINES}-line guidance"),
                suggestion: "Apply the derivability test, then move task-shaped detail into a skill or nested guide.",
            });
        }
    }

    let listing_tokens: usize = listed.values().map(|(cost, _)| cost / 4).sum();
    Ok((findings, listing_tokens, listed.len()))
}

fn render_markdown(report: &Report) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# claude-config-audit\n\n{} finding(s). Listing: ~{} tokens across {} skills.\n\n",
        report.summary.total, report.summary.listing_tokens, report.summary.skills_listed
    ));
    if report.findings.is_empty() {
        out.push_str("No drift detected.\n");
        return out;
    }
    for f in &report.findings {
        out.push_str(&format!(
            "- **{:?}** `{}` — {}\n  - {}\n  - {}\n",
            f.severity, f.id, f.subject, f.message, f.suggestion
        ));
    }
    out
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "claude-config-audit",
                &mut std::io::stdout(),
            );
            ExitCode::SUCCESS
        }
        Command::Doctor => {
            for (id, what) in RULES {
                println!("{id}\n    {what}");
            }
            ExitCode::SUCCESS
        }
        Command::Scan {
            home,
            project,
            format,
            min_severity,
        } => {
            let home = expand(&home);
            let project = project.map(|p| expand(&p));
            let (mut findings, listing_tokens, skills_listed) =
                match scan(&home, project.as_deref()) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("error: {e:#}");
                        return ExitCode::from(1);
                    }
                };
            findings.sort_by(|a, b| b.severity.cmp(&a.severity).then(a.id.cmp(b.id)));
            let mut by_severity = BTreeMap::new();
            for f in &findings {
                *by_severity
                    .entry(format!("{:?}", f.severity).to_lowercase())
                    .or_insert(0) += 1;
            }
            let report = Report {
                tool: "claude-config-audit",
                version: env!("CARGO_PKG_VERSION"),
                summary: Summary {
                    total: findings.len(),
                    by_severity,
                    listing_tokens,
                    skills_listed,
                },
                findings,
            };
            match format {
                Format::Json => println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_default()
                ),
                Format::Markdown => print!("{}", render_markdown(&report)),
            }
            if report.findings.iter().any(|f| f.severity >= min_severity) {
                ExitCode::from(2)
            } else {
                ExitCode::SUCCESS
            }
        }
    }
}

const RULES: &[(&str, &str)] = &[
    (
        "links.broken-skill-symlink",
        "A symlink in a skills root does not resolve; the loader skips it silently.",
    ),
    (
        "skill.oversized-skipped",
        "SKILL.md exceeds 128KB and is skipped entirely by the loader.",
    ),
    (
        "skill.body-too-long",
        "SKILL.md body exceeds 500 lines; move detail into references/.",
    ),
    (
        "skill.missing-frontmatter",
        "No parseable YAML frontmatter.",
    ),
    (
        "skill.name-mismatch",
        "Frontmatter name differs from the directory name.",
    ),
    (
        "skill.description-over-cap",
        "Description exceeds the 1024-char frontmatter cap.",
    ),
    (
        "skill.description-listing-hog",
        "Description far above the ~280-char (~50 token) target; it is paid on every request.",
    ),
    (
        "model.reasoning-extraction-risk",
        "Body asks the model to expose reasoning; Fable 5 refuses and falls back to Opus 4.8.",
    ),
    (
        "overrides.stale",
        "A skillOverrides entry targets a skill that no longer exists.",
    ),
    (
        "agent.duplicate-name",
        "Two agent files declare the same name; the loader silently discards one.",
    ),
    (
        "agent.description-bloat",
        "Agent description is oversized and sits in the system prompt on every request.",
    ),
    (
        "guide.over-line-budget",
        "A CLAUDE.md exceeds the 200-line guidance.",
    ),
];
