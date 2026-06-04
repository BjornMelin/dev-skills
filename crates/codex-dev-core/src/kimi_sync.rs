use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::write_json;

pub const KIMI_SYNC_SCHEMA: &str = "codex-dev.kimi-sync.v1";

const FOCUSED_PLUGIN_NAMES: &[&str] = &[
    "clerk-skills",
    "expo",
    "native-motion",
    "vercel",
    "web-motion",
];

const USER_EXCLUDED_PLUGIN_SKILLS: &[(&str, &str, &str)] = &[(
    "vercel",
    "shadcn",
    "excluded because the global official shadcn skill is preferred",
)];

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum KimiSyncScope {
    Focused,
    AllEnabled,
    GlobalOnly,
}

impl KimiSyncScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Focused => "focused",
            Self::AllEnabled => "all-enabled",
            Self::GlobalOnly => "global-only",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KimiSyncArgs {
    pub apply: bool,
    pub scope: KimiSyncScope,
    pub codex_home: Option<PathBuf>,
    pub agents_home: Option<PathBuf>,
    pub kimi_home: Option<PathBuf>,
    pub project_root: Option<PathBuf>,
    pub checked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KimiSyncReport {
    pub schema: String,
    pub checked_at: DateTime<Utc>,
    pub dry_run: bool,
    pub scope: KimiSyncScope,
    pub ok: bool,
    pub codex_home: PathBuf,
    pub agents_home: PathBuf,
    pub kimi_home: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_root: Option<PathBuf>,
    pub sync_root: PathBuf,
    pub mirror_root: PathBuf,
    pub skills_root: PathBuf,
    pub included: Vec<KimiSyncSkill>,
    pub excluded: Vec<KimiSyncExcludedSkill>,
    pub diagnostics: Vec<KimiSyncDiagnostic>,
    pub summary: KimiSyncSummary,
    pub launch_command: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KimiSyncSummary {
    pub included: usize,
    pub excluded: usize,
    pub diagnostics: usize,
    pub errors: usize,
    pub warnings: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KimiSyncSkill {
    pub name: String,
    pub source_kind: KimiSyncSourceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    pub source_path: PathBuf,
    pub mirror_path: PathBuf,
    pub priority: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KimiSyncSourceKind {
    GlobalSkill,
    ProjectSkill,
    PluginSkill,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KimiSyncExcludedSkill {
    pub name: String,
    pub source_kind: KimiSyncSourceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    pub source_path: PathBuf,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KimiSyncDiagnostic {
    pub severity: KimiSyncDiagnosticSeverity,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KimiSyncDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Clone, Debug)]
struct SkillCandidate {
    name: String,
    source_kind: KimiSyncSourceKind,
    plugin: Option<String>,
    source_path: PathBuf,
    priority: u8,
}

struct SkillRootSpec<'a> {
    root: &'a Path,
    source_kind: KimiSyncSourceKind,
    plugin: Option<&'a str>,
    priority: u8,
}

#[derive(Clone, Debug)]
struct SkillConfigRule {
    name: Option<String>,
    path: Option<PathBuf>,
    enabled: bool,
}

#[derive(Debug)]
struct CodexConfig {
    rules: Vec<SkillConfigRule>,
    plugins: BTreeMap<String, PluginState>,
}

#[derive(Debug)]
struct PluginState {
    source: String,
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct PluginManifest {
    name: String,
    #[serde(default)]
    skills: Option<String>,
}

pub fn kimi_sync(args: KimiSyncArgs) -> Result<KimiSyncReport> {
    let checked_at = args.checked_at.unwrap_or_else(Utc::now);
    let codex_home = default_home_path(args.codex_home, ".codex")?;
    let agents_home = default_home_path(args.agents_home, ".agents")?;
    let kimi_home = default_home_path(args.kimi_home, ".kimi-code")?;
    let sync_root = kimi_home.join("codex-sync");
    let config = read_codex_config(&codex_home.join("config.toml"))?;
    let project_root = resolve_project_root(args.project_root)?;
    let mirror_root = sync_root
        .join("effective")
        .join(project_hash(project_root.as_deref()));
    let skills_root = mirror_root.join("skills");

    let mut diagnostics = Vec::new();
    let mut excluded = Vec::new();
    let mut candidates = Vec::new();

    collect_global_skills(
        &agents_home.join("skills"),
        &config,
        &mut candidates,
        &mut excluded,
        &mut diagnostics,
    );
    collect_global_skills(
        &codex_home.join("skills"),
        &config,
        &mut candidates,
        &mut excluded,
        &mut diagnostics,
    );
    if let Some(project_root) = &project_root {
        collect_project_skills(
            &project_root.join(".agents").join("skills"),
            &config,
            &mut candidates,
            &mut excluded,
            &mut diagnostics,
        );
        collect_project_skills(
            &project_root.join(".codex").join("skills"),
            &config,
            &mut candidates,
            &mut excluded,
            &mut diagnostics,
        );
        collect_project_skills(
            &project_root.join(".kimi-code").join("skills"),
            &config,
            &mut candidates,
            &mut excluded,
            &mut diagnostics,
        );
    }
    if args.scope != KimiSyncScope::GlobalOnly {
        collect_plugin_skills(
            &codex_home,
            args.scope,
            &config,
            &mut candidates,
            &mut excluded,
            &mut diagnostics,
        );
    }

    let candidates = resolve_collisions(candidates, &mut excluded);
    let mut included = candidates
        .into_iter()
        .map(|candidate| KimiSyncSkill {
            mirror_path: skills_root.join(&candidate.name),
            name: candidate.name,
            source_kind: candidate.source_kind,
            plugin: candidate.plugin,
            source_path: candidate.source_path,
            priority: candidate.priority,
        })
        .collect::<Vec<_>>();
    included.sort_by(|left, right| left.name.cmp(&right.name));
    excluded.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.source_path.cmp(&right.source_path))
    });

    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == KimiSyncDiagnosticSeverity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == KimiSyncDiagnosticSeverity::Warning)
        .count();
    let ok = errors == 0;
    let launch_command = vec![
        "kimi".to_string(),
        "--skills-dir".to_string(),
        skills_root.display().to_string(),
    ];
    let summary = KimiSyncSummary {
        included: included.len(),
        excluded: excluded.len(),
        diagnostics: diagnostics.len(),
        errors,
        warnings,
    };

    let report = KimiSyncReport {
        schema: KIMI_SYNC_SCHEMA.to_string(),
        checked_at,
        dry_run: !args.apply,
        scope: args.scope,
        ok,
        codex_home,
        agents_home,
        kimi_home,
        project_root,
        sync_root,
        mirror_root,
        skills_root,
        included,
        excluded,
        diagnostics,
        summary,
        launch_command,
    };

    if args.apply {
        if !report.ok {
            bail!("refusing to apply Kimi sync with error diagnostics");
        }
        write_kimi_mirror(&report)?;
    }

    Ok(report)
}

fn default_home_path(path: Option<PathBuf>, child: &str) -> Result<PathBuf> {
    match path {
        Some(path) => Ok(path),
        None => {
            let home = env::var_os("HOME").ok_or_else(|| anyhow::anyhow!("HOME is not set"))?;
            Ok(PathBuf::from(home).join(child))
        }
    }
}

fn read_codex_config(path: &Path) -> Result<CodexConfig> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read Codex config {}", path.display()))?;
    let value = toml::Value::Table(
        toml::from_str::<toml::Table>(&text)
            .with_context(|| format!("failed to parse Codex config {}", path.display()))?,
    );
    let mut rules = Vec::new();
    if let Some(items) = value
        .get("skills")
        .and_then(|skills| skills.get("config"))
        .and_then(toml::Value::as_array)
    {
        for item in items {
            let Some(table) = item.as_table() else {
                continue;
            };
            rules.push(SkillConfigRule {
                name: table
                    .get("name")
                    .and_then(toml::Value::as_str)
                    .map(ToString::to_string),
                path: table
                    .get("path")
                    .and_then(toml::Value::as_str)
                    .map(|path| expand_home_path(Path::new(path))),
                enabled: table
                    .get("enabled")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(true),
            });
        }
    }

    let mut plugins = BTreeMap::new();
    if let Some(table) = value.get("plugins").and_then(toml::Value::as_table) {
        for (key, value) in table {
            let Some((name, source)) = key.split_once('@') else {
                continue;
            };
            let enabled = value
                .get("enabled")
                .and_then(toml::Value::as_bool)
                .unwrap_or(true);
            plugins.insert(
                name.to_string(),
                PluginState {
                    source: source.to_string(),
                    enabled,
                },
            );
        }
    }

    Ok(CodexConfig { rules, plugins })
}

fn resolve_project_root(project_root: Option<PathBuf>) -> Result<Option<PathBuf>> {
    let root = match project_root {
        Some(path) => path,
        None => {
            let cwd = env::current_dir().context("failed to read current directory")?;
            find_git_root(&cwd).unwrap_or(cwd)
        }
    };
    match fs::canonicalize(&root) {
        Ok(path) => Ok(Some(path)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to inspect {}", root.display())),
    }
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    for path in start.ancestors() {
        if path.join(".git").exists() {
            return Some(path.to_path_buf());
        }
    }
    None
}

fn project_hash(project_root: Option<&Path>) -> String {
    let Some(project_root) = project_root else {
        return "global".to_string();
    };
    let mut hasher = Sha256::new();
    hasher.update(project_root.display().to_string().as_bytes());
    let hash = hasher.finalize();
    format!("{:x}", hash)[..16].to_string()
}

fn collect_global_skills(
    root: &Path,
    config: &CodexConfig,
    candidates: &mut Vec<SkillCandidate>,
    excluded: &mut Vec<KimiSyncExcludedSkill>,
    diagnostics: &mut Vec<KimiSyncDiagnostic>,
) {
    collect_skill_root(
        SkillRootSpec {
            root,
            source_kind: KimiSyncSourceKind::GlobalSkill,
            plugin: None,
            priority: 20,
        },
        config,
        candidates,
        excluded,
        diagnostics,
    );
}

fn collect_project_skills(
    root: &Path,
    config: &CodexConfig,
    candidates: &mut Vec<SkillCandidate>,
    excluded: &mut Vec<KimiSyncExcludedSkill>,
    diagnostics: &mut Vec<KimiSyncDiagnostic>,
) {
    collect_skill_root(
        SkillRootSpec {
            root,
            source_kind: KimiSyncSourceKind::ProjectSkill,
            plugin: None,
            priority: 30,
        },
        config,
        candidates,
        excluded,
        diagnostics,
    );
}

fn collect_skill_root(
    spec: SkillRootSpec<'_>,
    config: &CodexConfig,
    candidates: &mut Vec<SkillCandidate>,
    excluded: &mut Vec<KimiSyncExcludedSkill>,
    diagnostics: &mut Vec<KimiSyncDiagnostic>,
) {
    let Some(entries) = read_skill_directories(spec.root, diagnostics) else {
        return;
    };
    for source_path in entries {
        let name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        add_candidate(
            SkillCandidate {
                name,
                source_kind: spec.source_kind.clone(),
                plugin: spec.plugin.map(ToString::to_string),
                source_path,
                priority: spec.priority,
            },
            config,
            candidates,
            excluded,
        );
    }
}

fn collect_plugin_skills(
    codex_home: &Path,
    scope: KimiSyncScope,
    config: &CodexConfig,
    candidates: &mut Vec<SkillCandidate>,
    excluded: &mut Vec<KimiSyncExcludedSkill>,
    diagnostics: &mut Vec<KimiSyncDiagnostic>,
) {
    let plugin_names = match scope {
        KimiSyncScope::Focused => FOCUSED_PLUGIN_NAMES
            .iter()
            .map(|name| (*name).to_string())
            .collect::<BTreeSet<_>>(),
        KimiSyncScope::AllEnabled => config
            .plugins
            .iter()
            .filter_map(|(name, state)| state.enabled.then_some(name.clone()))
            .collect(),
        KimiSyncScope::GlobalOnly => return,
    };

    for plugin_name in plugin_names {
        let Some(plugin_state) = config.plugins.get(&plugin_name) else {
            diagnostics.push(KimiSyncDiagnostic {
                severity: KimiSyncDiagnosticSeverity::Warning,
                code: "plugin_not_configured".to_string(),
                message: format!("plugin {plugin_name} is not configured in Codex"),
                path: None,
            });
            continue;
        };
        if !plugin_state.enabled {
            continue;
        }
        let Some((manifest_path, manifest)) =
            discover_plugin_manifest(codex_home, &plugin_name, &plugin_state.source, diagnostics)
        else {
            continue;
        };
        let Some(skills) = manifest.skills else {
            diagnostics.push(KimiSyncDiagnostic {
                severity: KimiSyncDiagnosticSeverity::Warning,
                code: "plugin_missing_skills".to_string(),
                message: format!("plugin {plugin_name} does not expose a skills directory"),
                path: Some(manifest_path),
            });
            continue;
        };
        let plugin_root = manifest_path
            .parent()
            .and_then(Path::parent)
            .unwrap_or(codex_home);
        let skills_root = plugin_root.join(skills);
        let Some(entries) = read_skill_directories_recursive(&skills_root, diagnostics) else {
            continue;
        };
        for source_path in entries {
            let name = source_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();
            if let Some(reason) = forced_exclusion_reason(&manifest.name, &name) {
                excluded.push(KimiSyncExcludedSkill {
                    name,
                    source_kind: KimiSyncSourceKind::PluginSkill,
                    plugin: Some(manifest.name.clone()),
                    source_path,
                    reason,
                });
                continue;
            }
            add_candidate(
                SkillCandidate {
                    name,
                    source_kind: KimiSyncSourceKind::PluginSkill,
                    plugin: Some(manifest.name.clone()),
                    source_path,
                    priority: 10,
                },
                config,
                candidates,
                excluded,
            );
        }
    }
}

fn discover_plugin_manifest(
    codex_home: &Path,
    plugin_name: &str,
    source: &str,
    diagnostics: &mut Vec<KimiSyncDiagnostic>,
) -> Option<(PathBuf, PluginManifest)> {
    let cache_root = codex_home.join("plugins").join("cache").join(source);
    let mut candidates = Vec::new();
    collect_plugin_manifest_candidates(&cache_root, plugin_name, &mut candidates);
    candidates.sort();
    candidates.reverse();

    for path in candidates {
        match read_plugin_manifest(&path) {
            Ok(manifest) if manifest.name == plugin_name => return Some((path, manifest)),
            Ok(_) => continue,
            Err(error) => diagnostics.push(KimiSyncDiagnostic {
                severity: KimiSyncDiagnosticSeverity::Warning,
                code: "plugin_manifest_read_error".to_string(),
                message: format!(
                    "failed to read plugin manifest {}: {error:#}",
                    path.display()
                ),
                path: Some(path),
            }),
        }
    }

    diagnostics.push(KimiSyncDiagnostic {
        severity: KimiSyncDiagnosticSeverity::Warning,
        code: "plugin_manifest_missing".to_string(),
        message: format!("failed to find enabled Codex plugin {plugin_name}@{source}"),
        path: Some(cache_root),
    });
    None
}

fn collect_plugin_manifest_candidates(
    root: &Path,
    plugin_name: &str,
    candidates: &mut Vec<PathBuf>,
) {
    let direct = root.join(plugin_name);
    collect_manifest_paths(&direct, 3, candidates);
    collect_manifest_paths(root, 4, candidates);
}

fn collect_manifest_paths(root: &Path, depth: usize, candidates: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let manifest = path.join(".codex-plugin").join("plugin.json");
        if manifest.is_file() {
            candidates.push(manifest);
            continue;
        }
        if path.is_dir() {
            collect_manifest_paths(&path, depth - 1, candidates);
        }
    }
}

fn read_plugin_manifest(path: &Path) -> Result<PluginManifest> {
    let file =
        fs::File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    serde_json::from_reader(file).with_context(|| format!("failed to parse {}", path.display()))
}

fn read_skill_directories(
    root: &Path,
    diagnostics: &mut Vec<KimiSyncDiagnostic>,
) -> Option<Vec<PathBuf>> {
    if !root.exists() {
        return None;
    }
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) => {
            diagnostics.push(KimiSyncDiagnostic {
                severity: KimiSyncDiagnosticSeverity::Warning,
                code: "skill_root_read_error".to_string(),
                message: format!("failed to read skill root {}: {error}", root.display()),
                path: Some(root.to_path_buf()),
            });
            return None;
        }
    };
    let mut skills = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()) == Some("dist") {
            continue;
        }
        if path.join("SKILL.md").is_file() {
            skills.push(path);
        }
    }
    skills.sort();
    Some(skills)
}

fn read_skill_directories_recursive(
    root: &Path,
    diagnostics: &mut Vec<KimiSyncDiagnostic>,
) -> Option<Vec<PathBuf>> {
    if !root.exists() {
        diagnostics.push(KimiSyncDiagnostic {
            severity: KimiSyncDiagnosticSeverity::Warning,
            code: "skill_root_missing".to_string(),
            message: format!("skill root does not exist: {}", root.display()),
            path: Some(root.to_path_buf()),
        });
        return None;
    }
    let mut skills = Vec::new();
    collect_skill_directories_recursive(root, 0, &mut skills, diagnostics);
    skills.sort();
    Some(skills)
}

fn collect_skill_directories_recursive(
    root: &Path,
    depth: usize,
    skills: &mut Vec<PathBuf>,
    diagnostics: &mut Vec<KimiSyncDiagnostic>,
) {
    if depth > 3 {
        return;
    }
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) => {
            diagnostics.push(KimiSyncDiagnostic {
                severity: KimiSyncDiagnosticSeverity::Warning,
                code: "skill_root_read_error".to_string(),
                message: format!("failed to read skill root {}: {error}", root.display()),
                path: Some(root.to_path_buf()),
            });
            return;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.join("SKILL.md").is_file() {
            skills.push(path);
        } else if path.is_dir() {
            collect_skill_directories_recursive(&path, depth + 1, skills, diagnostics);
        }
    }
}

fn forced_exclusion_reason(plugin: &str, name: &str) -> Option<String> {
    USER_EXCLUDED_PLUGIN_SKILLS
        .iter()
        .find(|(excluded_plugin, excluded_name, _)| {
            *excluded_plugin == plugin && *excluded_name == name
        })
        .map(|(_, _, reason)| (*reason).to_string())
}

fn add_candidate(
    candidate: SkillCandidate,
    config: &CodexConfig,
    candidates: &mut Vec<SkillCandidate>,
    excluded: &mut Vec<KimiSyncExcludedSkill>,
) {
    if skill_enabled(config, &candidate) {
        candidates.push(candidate);
    } else {
        excluded.push(KimiSyncExcludedSkill {
            name: candidate.name,
            source_kind: candidate.source_kind,
            plugin: candidate.plugin,
            source_path: candidate.source_path,
            reason: "disabled by Codex skills.config".to_string(),
        });
    }
}

fn skill_enabled(config: &CodexConfig, candidate: &SkillCandidate) -> bool {
    let namespaced = candidate
        .plugin
        .as_ref()
        .map(|plugin| format!("{plugin}:{}", candidate.name));
    let skill_md = candidate.source_path.join("SKILL.md");
    let mut enabled = true;
    for rule in &config.rules {
        let mut matches = false;
        if let Some(rule_name) = &rule.name {
            matches |= rule_name == &candidate.name;
        }
        if let (Some(rule_name), Some(namespaced)) = (&rule.name, &namespaced) {
            matches |= rule_name == namespaced;
        }
        if let Some(rule_path) = &rule.path {
            matches |= paths_equivalent(rule_path, &candidate.source_path);
            matches |= paths_equivalent(rule_path, &skill_md);
        }
        if matches {
            enabled = rule.enabled;
        }
    }
    enabled
}

fn resolve_collisions(
    candidates: Vec<SkillCandidate>,
    excluded: &mut Vec<KimiSyncExcludedSkill>,
) -> Vec<SkillCandidate> {
    let mut by_name: BTreeMap<String, SkillCandidate> = BTreeMap::new();
    for candidate in candidates {
        match by_name.get(&candidate.name) {
            Some(existing) if existing.priority >= candidate.priority => {
                excluded.push(KimiSyncExcludedSkill {
                    name: candidate.name,
                    source_kind: candidate.source_kind,
                    plugin: candidate.plugin,
                    source_path: candidate.source_path,
                    reason: format!(
                        "shadowed by higher-priority {} skill",
                        describe_source_kind(&existing.source_kind)
                    ),
                });
            }
            Some(existing) => {
                excluded.push(KimiSyncExcludedSkill {
                    name: existing.name.clone(),
                    source_kind: existing.source_kind.clone(),
                    plugin: existing.plugin.clone(),
                    source_path: existing.source_path.clone(),
                    reason: format!(
                        "shadowed by higher-priority {} skill",
                        describe_source_kind(&candidate.source_kind)
                    ),
                });
                by_name.insert(candidate.name.clone(), candidate);
            }
            None => {
                by_name.insert(candidate.name.clone(), candidate);
            }
        }
    }
    by_name.into_values().collect()
}

fn describe_source_kind(source_kind: &KimiSyncSourceKind) -> &'static str {
    match source_kind {
        KimiSyncSourceKind::GlobalSkill => "global",
        KimiSyncSourceKind::ProjectSkill => "project",
        KimiSyncSourceKind::PluginSkill => "plugin",
    }
}

fn paths_equivalent(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    let left = expand_home_path(left);
    let right = expand_home_path(right);
    left == right
        || fs::canonicalize(&left)
            .ok()
            .zip(fs::canonicalize(&right).ok())
            .is_some_and(|(left, right)| left == right)
}

fn expand_home_path(path: &Path) -> PathBuf {
    let Some(path_str) = path.to_str() else {
        return path.to_path_buf();
    };
    if path_str == "~" {
        return env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    }
    if let Some(rest) = path_str.strip_prefix("~/")
        && let Some(home) = env::var_os("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    path.to_path_buf()
}

fn write_kimi_mirror(report: &KimiSyncReport) -> Result<()> {
    assert_safe_sync_path(&report.sync_root, &report.mirror_root)?;
    let tmp_root = report.sync_root.join("tmp").join(format!(
        "{}-{}",
        project_hash(report.project_root.as_deref()),
        process_id()
    ));
    if tmp_root.exists() {
        fs::remove_dir_all(&tmp_root)
            .with_context(|| format!("failed to remove {}", tmp_root.display()))?;
    }
    let tmp_skills = tmp_root.join("skills");
    fs::create_dir_all(&tmp_skills)
        .with_context(|| format!("failed to create {}", tmp_skills.display()))?;
    for skill in &report.included {
        let link_path = tmp_skills.join(&skill.name);
        create_dir_symlink(&skill.source_path, &link_path)?;
    }
    write_json(tmp_root.join("manifest.json"), report)?;

    if report.mirror_root.exists() {
        fs::remove_dir_all(&report.mirror_root)
            .with_context(|| format!("failed to remove {}", report.mirror_root.display()))?;
    }
    if let Some(parent) = report.mirror_root.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::rename(&tmp_root, &report.mirror_root).with_context(|| {
        format!(
            "failed to move {} into {}",
            tmp_root.display(),
            report.mirror_root.display()
        )
    })?;
    Ok(())
}

fn assert_safe_sync_path(sync_root: &Path, path: &Path) -> Result<()> {
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        bail!("refusing unsafe generated mirror path: {}", path.display());
    }
    if !path.starts_with(sync_root) {
        bail!(
            "refusing to write outside Kimi sync root: {} is not under {}",
            path.display(),
            sync_root.display()
        );
    }
    Ok(())
}

fn process_id() -> u32 {
    std::process::id()
}

#[cfg(unix)]
fn create_dir_symlink(source: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(source, link).with_context(|| {
        format!(
            "failed to symlink {} -> {}",
            link.display(),
            source.display()
        )
    })
}

#[cfg(windows)]
fn create_dir_symlink(source: &Path, link: &Path) -> Result<()> {
    std::os::windows::fs::symlink_dir(source, link).with_context(|| {
        format!(
            "failed to symlink {} -> {}",
            link.display(),
            source.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn path_rule_after_name_rule_disables_skill() {
        let temp = tempdir().expect("tempdir");
        let codex = temp.path().join(".codex");
        let agents = temp.path().join(".agents");
        let kimi = temp.path().join(".kimi-code");
        let plugin = codex.join("plugins/cache/clerk/clerk-skills/1.0.0/skills/core/clerk-setup");
        write_skill(&plugin, "clerk-setup");
        write_text(
            &codex.join("plugins/cache/clerk/clerk-skills/1.0.0/.codex-plugin/plugin.json"),
            r#"{"name":"clerk-skills","skills":"./skills/"}"#,
        );
        write_text(
            &codex.join("config.toml"),
            &format!(
                r#"
[[skills.config]]
name = "clerk-skills:clerk-setup"
enabled = true

[[skills.config]]
path = "{}"
enabled = false

[plugins."clerk-skills@clerk"]
enabled = true
"#,
                plugin.join("SKILL.md").display()
            ),
        );

        let report = kimi_sync(KimiSyncArgs {
            apply: false,
            scope: KimiSyncScope::Focused,
            codex_home: Some(codex),
            agents_home: Some(agents),
            kimi_home: Some(kimi),
            project_root: None,
            checked_at: None,
        })
        .expect("sync");

        assert!(
            !report
                .included
                .iter()
                .any(|skill| skill.name == "clerk-setup")
        );
        assert!(
            report
                .excluded
                .iter()
                .any(|skill| skill.name == "clerk-setup")
        );
    }

    #[test]
    fn plain_name_rule_disables_global_skill() {
        let temp = tempdir().expect("tempdir");
        let codex = temp.path().join(".codex");
        let agents = temp.path().join(".agents");
        let kimi = temp.path().join(".kimi-code");
        write_skill(&agents.join("skills/disabled-global"), "disabled-global");
        write_skill(&agents.join("skills/enabled-global"), "enabled-global");
        write_text(
            &codex.join("config.toml"),
            r#"[[skills.config]]
name = "disabled-global"
enabled = false
"#,
        );

        let report = kimi_sync(KimiSyncArgs {
            apply: false,
            scope: KimiSyncScope::GlobalOnly,
            codex_home: Some(codex),
            agents_home: Some(agents),
            kimi_home: Some(kimi),
            project_root: None,
            checked_at: None,
        })
        .expect("sync");

        assert!(
            report
                .included
                .iter()
                .any(|skill| skill.name == "enabled-global")
        );
        assert!(
            !report
                .included
                .iter()
                .any(|skill| skill.name == "disabled-global")
        );
        assert!(report.excluded.iter().any(|skill| {
            skill.name == "disabled-global" && skill.reason == "disabled by Codex skills.config"
        }));
    }

    #[test]
    fn forced_vercel_shadcn_exclusion_preserves_global_shadcn() {
        let temp = tempdir().expect("tempdir");
        let codex = temp.path().join(".codex");
        let agents = temp.path().join(".agents");
        let kimi = temp.path().join(".kimi-code");
        write_skill(&agents.join("skills/shadcn"), "shadcn");
        write_skill(
            &codex.join("plugins/cache/openai-curated/vercel/2abb1c44/skills/shadcn"),
            "shadcn",
        );
        write_text(
            &codex.join("plugins/cache/openai-curated/vercel/2abb1c44/.codex-plugin/plugin.json"),
            r#"{"name":"vercel","skills":"./skills/"}"#,
        );
        write_text(
            &codex.join("config.toml"),
            r#"[plugins."vercel@openai-curated"]
enabled = true
"#,
        );

        let report = kimi_sync(KimiSyncArgs {
            apply: false,
            scope: KimiSyncScope::Focused,
            codex_home: Some(codex),
            agents_home: Some(agents),
            kimi_home: Some(kimi),
            project_root: None,
            checked_at: None,
        })
        .expect("sync");

        let included = report
            .included
            .iter()
            .find(|skill| skill.name == "shadcn")
            .expect("global shadcn included");
        assert_eq!(included.source_kind, KimiSyncSourceKind::GlobalSkill);
        assert!(
            report
                .excluded
                .iter()
                .any(|skill| skill.plugin.as_deref() == Some("vercel") && skill.name == "shadcn")
        );
    }

    #[test]
    fn apply_writes_symlink_mirror_only_under_kimi_sync_root() {
        let temp = tempdir().expect("tempdir");
        let codex = temp.path().join(".codex");
        let agents = temp.path().join(".agents");
        let kimi = temp.path().join(".kimi-code");
        write_skill(&agents.join("skills/deep-researcher"), "deep-researcher");
        write_text(&codex.join("config.toml"), "");

        let report = kimi_sync(KimiSyncArgs {
            apply: true,
            scope: KimiSyncScope::GlobalOnly,
            codex_home: Some(codex),
            agents_home: Some(agents),
            kimi_home: Some(kimi),
            project_root: None,
            checked_at: None,
        })
        .expect("sync");

        assert!(report.skills_root.join("deep-researcher").exists());
        assert!(report.mirror_root.join("manifest.json").is_file());
    }

    fn write_skill(path: &Path, name: &str) {
        write_text(
            &path.join("SKILL.md"),
            &format!("---\nname: {name}\ndescription: Test skill.\n---\n\n# {name}\n"),
        );
    }

    fn write_text(path: &Path, text: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, text).expect("write text");
    }
}
