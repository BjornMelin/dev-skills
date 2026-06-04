mod collect;
mod config;
mod mirror;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use collect::{
    collect_global_skills, collect_plugin_skills, collect_project_skills, resolve_collisions,
};
use config::{default_home_path, project_hash, read_codex_config, resolve_project_root};
use mirror::write_kimi_mirror;

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

#[derive(Debug, Default)]
struct CodexConfig {
    rules: Vec<SkillConfigRule>,
    plugins: BTreeMap<String, PluginState>,
}

#[derive(Debug)]
struct PluginState {
    name: String,
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
    if args.scope != KimiSyncScope::GlobalOnly
        && let Some(project_root) = &project_root
    {
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
