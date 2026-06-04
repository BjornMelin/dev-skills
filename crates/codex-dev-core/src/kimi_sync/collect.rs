use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::config::paths_equivalent;
use super::{
    CodexConfig, FOCUSED_PLUGIN_NAMES, KimiSyncDiagnostic, KimiSyncDiagnosticSeverity,
    KimiSyncExcludedSkill, KimiSyncScope, KimiSyncSourceKind, PluginManifest, SkillCandidate,
    SkillRootSpec, USER_EXCLUDED_PLUGIN_SKILLS,
};

pub(super) fn collect_global_skills(
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

pub(super) fn collect_project_skills(
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

pub(super) fn collect_plugin_skills(
    codex_home: &Path,
    scope: KimiSyncScope,
    config: &CodexConfig,
    candidates: &mut Vec<SkillCandidate>,
    excluded: &mut Vec<KimiSyncExcludedSkill>,
    diagnostics: &mut Vec<KimiSyncDiagnostic>,
) {
    let mut plugin_entries = Vec::new();
    match scope {
        KimiSyncScope::Focused => {
            for focused_name in FOCUSED_PLUGIN_NAMES {
                let matching = config
                    .plugins
                    .values()
                    .filter(|state| state.name == *focused_name && state.enabled)
                    .collect::<Vec<_>>();
                if matching.is_empty() {
                    diagnostics.push(KimiSyncDiagnostic {
                        severity: KimiSyncDiagnosticSeverity::Warning,
                        code: "plugin_not_configured".to_string(),
                        message: format!("plugin {focused_name} is not configured in Codex"),
                        path: None,
                    });
                }
                plugin_entries.extend(matching);
            }
        }
        KimiSyncScope::AllEnabled => {
            plugin_entries.extend(config.plugins.values().filter(|state| state.enabled));
        }
        KimiSyncScope::GlobalOnly => return,
    }

    for plugin_state in plugin_entries {
        let plugin_name = &plugin_state.name;
        let Some((manifest_path, manifest)) =
            discover_plugin_manifest(codex_home, plugin_name, &plugin_state.source, diagnostics)
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
        let Some(skills_root) =
            safe_plugin_skills_root(plugin_root, &skills, plugin_name, diagnostics)
        else {
            continue;
        };
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

fn safe_plugin_skills_root(
    plugin_root: &Path,
    skills: &str,
    plugin_name: &str,
    diagnostics: &mut Vec<KimiSyncDiagnostic>,
) -> Option<PathBuf> {
    let skills_root = plugin_root.join(skills);
    let canonical_plugin_root = match fs::canonicalize(plugin_root) {
        Ok(path) => path,
        Err(error) => {
            diagnostics.push(KimiSyncDiagnostic {
                severity: KimiSyncDiagnosticSeverity::Warning,
                code: "plugin_root_canonicalize_error".to_string(),
                message: format!("failed to canonicalize plugin root for {plugin_name}: {error}"),
                path: Some(plugin_root.to_path_buf()),
            });
            return None;
        }
    };
    let canonical_skills_root = match fs::canonicalize(&skills_root) {
        Ok(path) => path,
        Err(error) => {
            diagnostics.push(KimiSyncDiagnostic {
                severity: KimiSyncDiagnosticSeverity::Warning,
                code: "plugin_skills_root_canonicalize_error".to_string(),
                message: format!(
                    "failed to canonicalize plugin skills root for {plugin_name}: {error}"
                ),
                path: Some(skills_root),
            });
            return None;
        }
    };
    if !canonical_skills_root.starts_with(&canonical_plugin_root) {
        diagnostics.push(KimiSyncDiagnostic {
            severity: KimiSyncDiagnosticSeverity::Warning,
            code: "plugin_skills_root_escape".to_string(),
            message: format!(
                "plugin {plugin_name} skills root escapes plugin root: {}",
                canonical_skills_root.display()
            ),
            path: Some(canonical_skills_root),
        });
        return None;
    }
    Some(canonical_skills_root)
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

pub(super) fn resolve_collisions(
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
