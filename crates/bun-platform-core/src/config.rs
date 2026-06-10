use crate::types::Severity;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

const DEFAULT_EXCLUDE_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    ".next",
    ".turbo",
    "dist",
    "build",
    "coverage",
    "out",
    "opensrc",
    "target",
];

#[derive(Clone, Debug, Default)]
pub struct CliOverrides {
    pub baseline_path: Option<PathBuf>,
    pub include_paths: Vec<PathBuf>,
    pub exclude_dirs: Vec<String>,
    pub adapters: Vec<String>,
    pub max_files: Option<usize>,
    pub max_bytes: Option<u64>,
    pub write_cache: bool,
}

#[derive(Clone, Debug)]
pub struct AuditConfig {
    pub disabled_rules: Vec<String>,
    pub severity_overrides: HashMap<String, Severity>,
    pub exclude_dirs: Vec<String>,
    pub baseline_keys: Vec<String>,
    pub adapters: Vec<String>,
    pub include_paths: Vec<PathBuf>,
    pub max_files: usize,
    pub max_bytes: u64,
    pub validation_commands: Vec<String>,
    pub write_cache: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    #[serde(alias = "disabledRules")]
    disabled_rules: Option<Vec<String>>,
    #[serde(alias = "severityOverrides")]
    severity_overrides: Option<HashMap<String, Severity>>,
    #[serde(alias = "excludeDirs")]
    exclude_dirs: Option<Vec<String>>,
    baseline: Option<serde_json::Value>,
    adapters: Option<Vec<String>>,
    #[serde(alias = "includePaths")]
    include_paths: Option<Vec<String>>,
    #[serde(alias = "maxFiles")]
    max_files: Option<usize>,
    #[serde(alias = "maxBytes")]
    max_bytes: Option<u64>,
    #[serde(alias = "validationCommands")]
    validation_commands: Option<Vec<String>>,
    #[serde(alias = "writeCache")]
    write_cache: Option<bool>,
}

pub fn load_audit_config(
    root: &Path,
    config_path: Option<&Path>,
    overrides: &CliOverrides,
) -> Result<AuditConfig> {
    let resolved_config_path = config_path
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("bun-platform.config.json"));
    let loaded = if resolved_config_path.exists() {
        let text = fs::read_to_string(&resolved_config_path)
            .with_context(|| format!("failed to read {}", resolved_config_path.display()))?;
        Some(
            serde_json::from_str::<FileConfig>(&text)
                .with_context(|| format!("failed to parse {}", resolved_config_path.display()))?,
        )
    } else if config_path.is_some() {
        anyhow::bail!(
            "config file does not exist: {}",
            resolved_config_path.display()
        );
    } else {
        None
    };

    let mut exclude_dirs = DEFAULT_EXCLUDE_DIRS
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    if let Some(extra) = loaded.as_ref().and_then(|cfg| cfg.exclude_dirs.clone()) {
        for value in extra {
            if !exclude_dirs.contains(&value) {
                exclude_dirs.push(value);
            }
        }
    }
    for value in &overrides.exclude_dirs {
        if !exclude_dirs.contains(value) {
            exclude_dirs.push(value.clone());
        }
    }

    let mut include_paths = loaded
        .as_ref()
        .and_then(|cfg| cfg.include_paths.clone())
        .unwrap_or_default()
        .into_iter()
        .map(|value| root.join(value))
        .collect::<Vec<_>>();
    for value in &overrides.include_paths {
        let resolved = if value.is_absolute() {
            value.clone()
        } else {
            root.join(value)
        };
        if !include_paths.contains(&resolved) {
            include_paths.push(resolved);
        }
    }

    let adapters = if !overrides.adapters.is_empty() {
        overrides.adapters.clone()
    } else if let Some(values) = loaded.as_ref().and_then(|cfg| cfg.adapters.clone()) {
        values
    } else {
        vec!["auto".to_string()]
    };

    let baseline_keys = read_baseline_keys(root, config_path, overrides, loaded.as_ref())?;

    Ok(AuditConfig {
        disabled_rules: loaded
            .as_ref()
            .and_then(|cfg| cfg.disabled_rules.clone())
            .unwrap_or_default(),
        severity_overrides: loaded
            .as_ref()
            .and_then(|cfg| cfg.severity_overrides.clone())
            .unwrap_or_default(),
        exclude_dirs,
        baseline_keys,
        adapters,
        include_paths,
        max_files: overrides
            .max_files
            .or_else(|| loaded.as_ref().and_then(|cfg| cfg.max_files))
            .unwrap_or(usize::MAX),
        max_bytes: overrides
            .max_bytes
            .or_else(|| loaded.as_ref().and_then(|cfg| cfg.max_bytes))
            .unwrap_or(u64::MAX),
        validation_commands: loaded
            .as_ref()
            .and_then(|cfg| cfg.validation_commands.clone())
            .unwrap_or_default(),
        write_cache: overrides.write_cache
            || loaded
                .as_ref()
                .and_then(|cfg| cfg.write_cache)
                .unwrap_or(false),
    })
}

fn read_baseline_keys(
    root: &Path,
    config_path: Option<&Path>,
    overrides: &CliOverrides,
    loaded: Option<&FileConfig>,
) -> Result<Vec<String>> {
    if let Some(path) = overrides.baseline_path.as_ref() {
        return load_baseline_file(path);
    }

    let Some(loaded) = loaded else {
        return Ok(Vec::new());
    };
    let Some(baseline) = loaded.baseline.as_ref() else {
        return Ok(Vec::new());
    };

    match baseline {
        serde_json::Value::Array(values) => Ok(values
            .iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect()),
        serde_json::Value::String(value) => {
            let base = config_path.and_then(Path::parent).unwrap_or(root);
            load_baseline_file(&base.join(value))
        }
        serde_json::Value::Object(map) => {
            let values = map
                .get("suppressionKeys")
                .and_then(|value| value.as_array())
                .context("baseline object must contain `suppressionKeys` as an array")?;
            Ok(values
                .iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect())
        }
        _ => anyhow::bail!(
            "baseline must be an array of suppression keys, a path string, or an object with suppressionKeys"
        ),
    }
}

fn load_baseline_file(path: &Path) -> Result<Vec<String>> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let json = serde_json::from_str::<serde_json::Value>(&text)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    match json {
        serde_json::Value::Array(values) => Ok(values
            .iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect()),
        serde_json::Value::Object(map) => {
            let values = map
                .get("suppressionKeys")
                .and_then(|value| value.as_array())
                .context("baseline file object must contain `suppressionKeys` as an array")?;
            Ok(values
                .iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect())
        }
        _ => anyhow::bail!(
            "baseline file must be an array of suppression keys or an object with suppressionKeys"
        ),
    }
}
