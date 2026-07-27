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

use anyhow::{Context, Result, bail};
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
        /// Authoring source of truth for skills, e.g. a repo's `skills/`
        /// directory. Skills present in both it and the live install are
        /// compared; a difference means an edit was committed but never
        /// deployed, or deployed but never committed.
        #[arg(long)]
        mirror: Option<String>,
        /// Additional per-skill symlink farm belonging to another agent, e.g.
        /// ~/.cursor/skills. Repeatable. Checked for dangling links and for two
        /// names pointing at the same skill.
        #[arg(long = "farm")]
        farms: Vec<String>,
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

/// Parse the frontmatter block as real YAML.
///
/// A hand-rolled line scanner accepts blocks the actual loader and the
/// repository validator reject, for example valid `name:`/`description:` lines
/// sitting alongside malformed YAML such as `metadata: [`. Parsing the whole
/// block means a file that would fail validation fails here too.
fn parse_frontmatter(text: &str) -> Option<Frontmatter> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let block = &rest[..end];
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(block).ok()?;
    let map = value.as_mapping()?;
    let get_str = |key: &str| {
        map.get(serde_yaml_ng::Value::String(key.to_string()))
            .and_then(|v| v.as_str())
            .map(str::to_string)
    };
    Some(Frontmatter {
        name: get_str("name"),
        description: get_str("description"),
        disable_model_invocation: map
            .get(serde_yaml_ng::Value::String(
                "disable-model-invocation".to_string(),
            ))
            .and_then(serde_yaml_ng::Value::as_bool)
            .unwrap_or(false),
    })
}

/// Whether a matched phrase is preceded by a negation or sits inside a quote,
/// e.g. "do not show your reasoning" or "avoid `explain your reasoning`".
fn phrase_is_negated(haystack: &str, at: usize) -> bool {
    const NEGATORS: &[&str] = &[
        "do not",
        "don't",
        "never",
        "avoid",
        "without",
        "no ",
        "not ",
        "must not",
        "should not",
        "refrain from",
        "instead of",
        "rather than",
        "prevent",
    ];
    let start = at.saturating_sub(60);
    let window = &haystack[start..at];
    NEGATORS.iter().any(|n| window.contains(n))
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

/// Generated output that lives beside a skill without being part of it.
///
/// Reporting a stale `.pyc` as drift would be noise, and noise is how a rule
/// teaches people to ignore it.
fn is_generated_noise(relative: &Path) -> bool {
    relative.components().any(|component| {
        matches!(
            component.as_os_str().to_string_lossy().as_ref(),
            "__pycache__" | "node_modules" | ".DS_Store" | ".pytest_cache" | ".ruff_cache"
        )
    })
}

/// Every meaningful file under `root`, as paths relative to it.
fn relative_files(root: &Path) -> BTreeSet<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            entry
                .path()
                .strip_prefix(root)
                .ok()
                .map(std::path::Path::to_path_buf)
        })
        .filter(|relative| !is_generated_noise(relative))
        .collect()
}

/// The first difference between two skill directories, described for a reader.
///
/// Returns `None` when the trees hold the same files with the same bytes.
/// Reporting only the first difference keeps the finding readable; the point is
/// to say *that* a skill drifted, not to render a diff.
fn first_tree_difference(left: &Path, right: &Path) -> Option<String> {
    let left_files = relative_files(left);
    let right_files = relative_files(right);

    if let Some(missing) = left_files.difference(&right_files).next() {
        return Some(format!("{} is not installed", missing.display()));
    }
    if let Some(extra) = right_files.difference(&left_files).next() {
        return Some(format!("{} exists only in the install", extra.display()));
    }
    for name in &left_files {
        let (Ok(a), Ok(b)) = (fs::read(left.join(name)), fs::read(right.join(name))) else {
            continue;
        };
        if a != b {
            return Some(format!("{} differs", name.display()));
        }
    }
    None
}

/// Plugins the user has explicitly switched off, as `plugin@marketplace` keys.
///
/// A skill supplied only by a disabled plugin is not loadable, so an override
/// naming it is just as inert as one naming a deleted skill. Counting those
/// names as "known" is why an earlier version reported such overrides as fine.
/// `settings_files` must be ordered lowest precedence first. A later file wins,
/// so a project that re-enables a plugin the user disabled globally is reported
/// as enabled. Accumulating every `false` instead would mark a plugin inert
/// while it is actually active for the project.
fn disabled_plugins(settings_files: &[PathBuf]) -> BTreeSet<String> {
    let mut effective: BTreeMap<String, bool> = BTreeMap::new();
    for settings in settings_files {
        let Ok(text) = fs::read_to_string(settings) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        if let Some(map) = value.get("enabledPlugins").and_then(|v| v.as_object()) {
            for (key, enabled) in map {
                if let Some(enabled) = enabled.as_bool() {
                    effective.insert(key.clone(), enabled);
                }
            }
        }
    }
    effective
        .into_iter()
        .filter_map(|(key, enabled)| (!enabled).then_some(key))
        .collect()
}

/// Map a marketplace SKILL.md path back to the `plugin@marketplace` key that
/// gates it, when the layout makes that derivable.
///
/// Recognised layout: `<marketplaces>/<market>/plugins/<plugin>/**/SKILL.md`.
/// Skills that sit directly under `<marketplaces>/<market>/skills/**` are not
/// plugin-gated in this way and return `None`.
fn plugin_key_for(marketplaces: &Path, skill: &Path) -> Option<String> {
    let rel = skill.strip_prefix(marketplaces).ok()?;
    let mut parts = rel.components().map(|c| c.as_os_str().to_string_lossy());
    let market = parts.next()?.to_string();
    if parts.next()? != "plugins" {
        return None;
    }
    let plugin = parts.next()?.to_string();
    Some(format!("{plugin}@{market}"))
}

#[allow(clippy::too_many_lines)]
fn scan(
    home: &Path,
    project: Option<&Path>,
    mirror: Option<&Path>,
    farms: &[PathBuf],
) -> Result<(Vec<Finding>, usize, usize)> {
    let mut findings = Vec::new();

    // A root the caller named explicitly must exist. Skipping a mistyped or
    // unmounted --mirror/--farm would let the scan print "No drift detected"
    // having audited neither, which is the failure this tool exists to end.
    if let Some(mirror) = mirror
        && !mirror.is_dir()
    {
        bail!("--mirror path is not a directory: {}", mirror.display());
    }
    for farm in farms {
        if !farm.is_dir() {
            bail!("--farm path is not a directory: {}", farm.display());
        }
    }

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
                // Only affirmative requests count. Guidance that forbids
                // extraction ("do not show your reasoning; return conclusions")
                // is the opposite of the risk and must not be reported.
                if let Some(at) = lowered.find(probe)
                    && !phrase_is_negated(&lowered, at)
                {
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

            let desc = fm.description.unwrap_or_default();
            // Count characters, not UTF-8 bytes. The cap is a character limit and
            // the repository validator counts code points, so measuring bytes
            // would flag a valid accented description as over-cap.
            let desc_chars = desc.chars().count();

            // The hard cap applies regardless of `disable-model-invocation`: that
            // flag removes a skill from the automatic listing, it does not exempt
            // its frontmatter from validation.
            if desc_chars > DESC_MAX_CHARS {
                findings.push(Finding {
                    id: "skill.description-over-cap",
                    severity: Severity::High,
                    subject: name.clone(),
                    message: format!(
                        "description is {desc_chars} chars, above the {DESC_MAX_CHARS} cap"
                    ),
                    suggestion: "Trim; the frontmatter cap is enforced by the loader.",
                });
            }

            if fm.disable_model_invocation {
                // Excluded from the automatic listing, so it has no listing cost
                // and the listing-hog rule does not apply.
                continue;
            }
            if desc_chars <= DESC_MAX_CHARS && desc_chars > DESC_TARGET_CHARS {
                findings.push(Finding {
                    id: "skill.description-listing-hog",
                    severity: Severity::Low,
                    subject: name.clone(),
                    message: format!("description is {desc_chars} chars (~{} tokens), above the ~{DESC_TARGET_CHARS}-char target", desc_chars / 4),
                    suggestion: "Lead with the key use case and keep concrete trigger phrases.",
                });
            }
            listed.insert(name.clone(), (name.chars().count() + desc_chars + 20, true));
        }
    }

    let mut settings_files = vec![home.join("settings.json"), home.join("settings.local.json")];
    if let Some(p) = project {
        settings_files.push(p.join(".claude").join("settings.json"));
        settings_files.push(p.join(".claude").join("settings.local.json"));
    }
    let disabled = disabled_plugins(&settings_files);

    // Plugin marketplaces contribute skills too. They are not audited for size
    // (they are upstream), but their names must be known so overrides that
    // target them are not misreported as stale.
    //
    // Skills reachable only through a DISABLED plugin are tracked separately:
    // they are not loadable, so an override naming one is inert, and reporting
    // it as fine hides exactly the drift this tool exists to surface.
    let marketplaces = home.join("plugins").join("marketplaces");
    let mut disabled_only: BTreeMap<String, String> = BTreeMap::new();
    if marketplaces.exists() {
        for entry in WalkDir::new(&marketplaces)
            .max_depth(6)
            .into_iter()
            .flatten()
        {
            if entry.file_name() != "SKILL.md" {
                continue;
            }
            let Some(dir) = entry.path().parent() else {
                continue;
            };
            let Some(name) = dir.file_name() else {
                continue;
            };
            let name = name.to_string_lossy().to_string();
            match plugin_key_for(&marketplaces, entry.path()) {
                Some(key) if disabled.contains(&key) => {
                    disabled_only.entry(name).or_insert(key);
                }
                _ => {
                    disabled_only.remove(&name);
                    known.insert(name);
                }
            }
        }
    }

    // ---- overrides ---------------------------------------------------------
    for settings in &settings_files {
        for (skill, mode) in read_overrides(settings) {
            // Plugin-namespaced entries (`plugin:skill`) are not resolvable here.
            if skill.contains(':') || known.contains(&skill) {
                continue;
            }
            if let Some(key) = disabled_only.get(&skill) {
                findings.push(Finding {
                    id: "overrides.targets-disabled-plugin",
                    severity: Severity::Medium,
                    subject: format!("{skill} = {mode}"),
                    message: format!(
                        "override in {} targets a skill supplied only by `{key}`, which is disabled",
                        settings.display()
                    ),
                    suggestion:
                        "Enable the plugin or drop the override; as written it has no effect.",
                });
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

    // ---- mirror drift ------------------------------------------------------
    // The authoring repo and the live install are separate trees with no
    // automatic sync. A skill edited in one and not the other looks shipped
    // while the running estate never changed.
    if let Some(mirror) = mirror {
        let live: BTreeMap<String, PathBuf> = roots.iter().flat_map(|r| skill_files(r)).collect();
        for (name, src) in skill_files(mirror) {
            let Some(dst) = live.get(&name) else {
                continue; // authored but deliberately not installed
            };
            // Compare the whole skill directory, not just SKILL.md. A skill's
            // references/, scripts/ and assets/ change its behaviour just as
            // much as its entrypoint, and an entrypoint-only check missed a
            // real 51-line reference drift in this very estate.
            let (Some(src_dir), Some(dst_dir)) = (src.parent(), dst.parent()) else {
                continue;
            };
            if let Some(detail) = first_tree_difference(src_dir, dst_dir) {
                findings.push(Finding {
                    id: "mirror.drift",
                    severity: Severity::Medium,
                    subject: name,
                    message: format!("{} differs from the installed copy: {detail}", src_dir.display()),
                    suggestion:
                        "Decide which side is authoritative and sync it; committing does not deploy.",
                });
            }
        }
    }

    // ---- sibling agent farms ----------------------------------------------
    // Other agents (Cursor, Factory, Codex) curate the same shared skill
    // library through their own symlink farms. A dangling link there is
    // invisible in exactly the same way it is here.
    for farm in farms {
        if !farm.exists() {
            continue;
        }
        let mut targets: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
        for entry in fs::read_dir(farm)
            .with_context(|| format!("read {}", farm.display()))?
            .flatten()
        {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if !path.is_symlink() {
                continue;
            }
            if fs::metadata(&path).is_err() {
                let target = fs::read_link(&path).unwrap_or_default();
                findings.push(Finding {
                    id: "farm.broken-symlink",
                    severity: Severity::Medium,
                    subject: path.display().to_string(),
                    message: format!("symlink target does not resolve: {}", target.display()),
                    suggestion: "Repoint or remove it; the owning agent skips it in silence.",
                });
                continue;
            }
            if let Ok(real) = fs::canonicalize(&path) {
                targets.entry(real).or_default().push(name);
            }
        }
        for (target, mut names) in targets {
            if names.len() > 1 {
                names.sort();
                findings.push(Finding {
                    id: "farm.duplicate-target",
                    severity: Severity::Low,
                    subject: format!("{} -> {}", farm.display(), target.display()),
                    message: format!("{} names resolve to one skill: {}", names.len(), names.join(", ")),
                    suggestion: "Keep the canonical name; an alias shadows it under a stale identity.",
                });
            }
        }
    }

    // ---- agents ------------------------------------------------------------
    // Keyed by (scope, name). A project agent intentionally shadowing a global
    // one is a supported configuration, so only duplicates WITHIN a scope are a
    // defect: those are the ones the loader silently discards.
    let mut agent_names: BTreeMap<(&str, String), Vec<String>> = BTreeMap::new();
    let mut agent_dirs = vec![("user", home.join("agents"))];
    if let Some(p) = project {
        agent_dirs.push(("project", p.join(".claude").join("agents")));
    }
    for (scope, dir) in &agent_dirs {
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
                .entry((scope, name))
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
    for ((scope, name), paths) in agent_names {
        if paths.len() > 1 {
            findings.push(Finding {
                id: "agent.duplicate-name",
                severity: Severity::High,
                subject: format!("{name} ({scope} scope)"),
                message: format!("declared by {} files: {}", paths.len(), paths.join(", ")),
                suggestion: "The loader keeps one and silently discards the rest. Rename or remove.",
            });
        }
    }

    // ---- guides ------------------------------------------------------------
    let mut guides = vec![home.join("CLAUDE.md")];
    if let Some(p) = project {
        // No depth cap: a guide at packages/app/src/CLAUDE.md is real and would
        // otherwise pass an audit while over budget. Pruning below keeps the
        // walk cheap.
        for entry in WalkDir::new(p)
            .into_iter()
            .filter_entry(|e| {
                let n = e.file_name().to_string_lossy();
                n != "node_modules"
                    && n != ".git"
                    && n != "target"
                    && n != "dist"
                    && n != "build"
                    && n != ".next"
                    && n != ".turbo"
                    && n != "worktrees"
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
            mirror,
            farms,
            format,
            min_severity,
        } => {
            let home = expand(&home);
            let project = project.map(|p| expand(&p));
            let mirror = mirror.map(|m| expand(&m));
            let farms: Vec<PathBuf> = farms.iter().map(|f| expand(f)).collect();
            let (mut findings, listing_tokens, skills_listed) =
                match scan(&home, project.as_deref(), mirror.as_deref(), &farms) {
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
        "overrides.targets-disabled-plugin",
        "An override targets a skill supplied only by a disabled plugin, so it has no effect.",
    ),
    (
        "mirror.drift",
        "A skill differs between its authoring source and the installed copy; committing does not deploy.",
    ),
    (
        "farm.broken-symlink",
        "A dangling link in another agent's skill farm (--farm), silently skipped by that agent.",
    ),
    (
        "farm.duplicate-target",
        "Two names in one farm resolve to the same skill, shadowing the canonical identity.",
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
