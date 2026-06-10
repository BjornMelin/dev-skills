use crate::{
    config::AuditConfig,
    state::PlatformPaths,
    types::{FixKind, PlannedFix, VERIFIED_BUN_VERSION},
};
use anyhow::{Context, Result};
use regex::Regex;
use serde_json::{Map, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

static NPX_COMMAND_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bnpx\b").expect("valid regex"));

pub fn plan_safe_fixes(root: &Path, config: &AuditConfig) -> Result<Vec<PlannedFix>> {
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", root.display()))?;
    let package_json_path = root.join("package.json");
    if !package_json_path.is_file() {
        return Ok(Vec::new());
    }

    let before = fs::read_to_string(&package_json_path)
        .with_context(|| format!("failed to read {}", package_json_path.display()))?;
    let mut json = serde_json::from_str::<Value>(&before)
        .with_context(|| format!("failed to parse {}", package_json_path.display()))?;
    let original = json.clone();
    let mut rule_ids = Vec::new();
    let mut descriptions = Vec::new();
    let policy = FixPolicy {
        package_json_path: &package_json_path,
        config,
    };

    if let Value::Object(root_map) = &mut json {
        let bun_first = is_bun_first_repo(&root, root_map);
        normalize_package_manager(
            root_map,
            bun_first,
            &policy,
            &mut rule_ids,
            &mut descriptions,
        );
        normalize_scripts(
            &root,
            root_map,
            bun_first,
            &policy,
            &mut rule_ids,
            &mut descriptions,
        );
    }

    if json == original || rule_ids.is_empty() {
        return Ok(Vec::new());
    }
    let after = serde_json::to_string_pretty(&json)? + "\n";
    Ok(vec![PlannedFix {
        rule_id: rule_ids[0].clone(),
        rule_ids,
        kind: FixKind::Safe,
        file: package_json_path.display().to_string(),
        description: format!("Update package.json to {}.", descriptions.join("; ")),
        before: Some(before),
        after: Some(after),
    }])
}

pub fn apply_safe_fixes(
    root: &Path,
    config: &AuditConfig,
    paths: &PlatformPaths,
) -> Result<Vec<PlannedFix>> {
    let fixes = plan_safe_fixes(root, config)?;
    if fixes.is_empty() {
        return Ok(fixes);
    }
    paths.ensure()?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let rollback_path = paths
        .rollback_dir()
        .join(format!("rollback-{timestamp}.json"));
    fs::write(
        &rollback_path,
        serde_json::to_vec_pretty(&serde_json::json!({
          "root": root.display().to_string(),
          "createdAtEpochSeconds": timestamp,
          "fixes": fixes,
        }))?,
    )?;
    for fix in &fixes {
        if let Some(after) = &fix.after {
            fs::write(PathBuf::from(&fix.file), after)
                .with_context(|| format!("failed to write {}", fix.file))?;
        }
    }
    Ok(fixes)
}

struct FixPolicy<'a> {
    package_json_path: &'a Path,
    config: &'a AuditConfig,
}

impl FixPolicy<'_> {
    fn allows(&self, rule_id: &str) -> bool {
        if self
            .config
            .disabled_rules
            .iter()
            .any(|value| value == rule_id)
        {
            return false;
        }
        let suppression_key = format!(
            "{rule_id}:{}",
            self.package_json_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("package.json")
        );
        !self
            .config
            .baseline_keys
            .iter()
            .any(|value| value == &suppression_key)
    }
}

fn normalize_package_manager(
    root_map: &mut Map<String, Value>,
    bun_first: bool,
    policy: &FixPolicy<'_>,
    rule_ids: &mut Vec<String>,
    descriptions: &mut Vec<String>,
) {
    const RULE_ID: &str = "pm-package-manager-field";
    let package_manager = root_map
        .get("packageManager")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if package_manager.is_empty() && bun_first && policy.allows(RULE_ID) {
        root_map.insert(
            "packageManager".to_string(),
            Value::String(format!("bun@{VERIFIED_BUN_VERSION}")),
        );
        rule_ids.push(RULE_ID.to_string());
        descriptions.push(format!("add packageManager bun@{VERIFIED_BUN_VERSION}"));
    }
}

fn normalize_scripts(
    root: &Path,
    root_map: &mut Map<String, Value>,
    bun_first: bool,
    policy: &FixPolicy<'_>,
    rule_ids: &mut Vec<String>,
    descriptions: &mut Vec<String>,
) {
    let vercel_bun_enabled = has_vercel_bun_runtime(root);
    let Some(scripts) = root_map.get_mut("scripts").and_then(Value::as_object_mut) else {
        return;
    };
    let mut rewrote_npx = false;

    const NPX_RULE_ID: &str = "pm-bunx-vs-npx";
    if bun_first && policy.allows(NPX_RULE_ID) {
        for value in scripts.values_mut() {
            if let Some(command) = value.as_str()
                && NPX_COMMAND_RE.is_match(command)
            {
                *value = Value::String(NPX_COMMAND_RE.replace_all(command, "bunx").into_owned());
                rewrote_npx = true;
            }
        }
    }
    if rewrote_npx {
        rule_ids.push(NPX_RULE_ID.to_string());
        descriptions.push("rewrite npx invocations to bunx".to_string());
    }
    const VERCEL_RULE_ID: &str = "vercel-nextjs-bun-runtime-scripts";
    if vercel_bun_enabled && policy.allows(VERCEL_RULE_ID) {
        let mut changed = false;
        if scripts
            .get("dev")
            .and_then(Value::as_str)
            .map(|value| value.trim() == "next dev")
            .unwrap_or(false)
        {
            scripts.insert(
                "dev".to_string(),
                Value::String("bun run --bun next dev".to_string()),
            );
            changed = true;
        }
        if scripts
            .get("build")
            .and_then(Value::as_str)
            .map(|value| value.trim() == "next build")
            .unwrap_or(false)
        {
            scripts.insert(
                "build".to_string(),
                Value::String("bun run --bun next build".to_string()),
            );
            changed = true;
        }
        if changed {
            rule_ids.push(VERCEL_RULE_ID.to_string());
            descriptions.push("normalize Next.js dev/build scripts for Bun runtime".to_string());
        }
    }
}

fn is_bun_first_repo(root: &Path, root_map: &Map<String, Value>) -> bool {
    root.join("bun.lockb").is_file()
        || root.join("bun.lock").is_file()
        || root_map
            .get("packageManager")
            .and_then(Value::as_str)
            .map(|value| value.starts_with("bun@"))
            .unwrap_or(false)
        || root_map
            .get("devDependencies")
            .and_then(Value::as_object)
            .map(|deps| deps.contains_key("@types/bun") || deps.contains_key("bun-types"))
            .unwrap_or(false)
}

fn has_vercel_bun_runtime(root: &Path) -> bool {
    if let Ok(text) = fs::read_to_string(root.join("vercel.json"))
        && let Ok(json) = serde_json::from_str::<Value>(&text)
        && contains_bun_runtime_config(&json)
    {
        return true;
    }
    fs::read_to_string(root.join("vercel.ts"))
        .map(|text| {
            let Some(captures) = Regex::new(r#"bunVersion\s*:\s*["']([^"']+)["']"#)
                .expect("valid regex")
                .captures(&text)
            else {
                return false;
            };
            text.contains("export")
                && captures
                    .get(1)
                    .map(|value| !value.as_str().trim().is_empty())
                    .unwrap_or(false)
        })
        .unwrap_or(false)
}

fn contains_bun_runtime_config(value: &Value) -> bool {
    match value {
        Value::Array(values) => values.iter().any(contains_bun_runtime_config),
        Value::Object(map) => {
            map.get("runtime")
                .and_then(Value::as_str)
                .map(|runtime| runtime == "bun" || runtime.starts_with("bun@"))
                .unwrap_or(false)
                || map
                    .get("bunVersion")
                    .and_then(Value::as_str)
                    .map(|version| !version.trim().is_empty())
                    .unwrap_or(false)
                || map.values().any(contains_bun_runtime_config)
        }
        _ => false,
    }
}
