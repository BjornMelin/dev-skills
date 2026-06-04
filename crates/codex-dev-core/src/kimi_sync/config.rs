use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use super::{CodexConfig, PluginState, SkillConfigRule};

pub(super) fn default_home_path(path: Option<PathBuf>, child: &str) -> Result<PathBuf> {
    match path {
        Some(path) => Ok(path),
        None => {
            let home = env::var_os("HOME").ok_or_else(|| anyhow::anyhow!("HOME is not set"))?;
            Ok(PathBuf::from(home).join(child))
        }
    }
}

pub(super) fn read_codex_config(path: &Path) -> Result<CodexConfig> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(CodexConfig::default());
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read Codex config {}", path.display()));
        }
    };
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
                key.to_string(),
                PluginState {
                    name: name.to_string(),
                    source: source.to_string(),
                    enabled,
                },
            );
        }
    }

    Ok(CodexConfig { rules, plugins })
}

pub(super) fn resolve_project_root(project_root: Option<PathBuf>) -> Result<Option<PathBuf>> {
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

pub(super) fn project_hash(project_root: Option<&Path>) -> String {
    let Some(project_root) = project_root else {
        return "global".to_string();
    };
    let mut hasher = Sha256::new();
    hasher.update(project_root.display().to_string().as_bytes());
    let hash = hasher.finalize();
    format!("{:x}", hash)[..16].to_string()
}

pub(super) fn paths_equivalent(left: &Path, right: &Path) -> bool {
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
