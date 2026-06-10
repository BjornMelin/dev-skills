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
static BUN_COMMAND_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bbun(x)?\b").expect("valid regex"));

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
        .as_millis();
    let rollback_path = paths
        .rollback_dir()
        .join(format!("rollback-{timestamp}.json"));
    fs::write(
        &rollback_path,
        serde_json::to_vec_pretty(&serde_json::json!({
          "root": root.display().to_string(),
          "createdAtEpochMillis": timestamp,
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
        || root_map
            .get("scripts")
            .and_then(Value::as_object)
            .map(|scripts| {
                scripts.values().any(|value| {
                    value
                        .as_str()
                        .map(|command| BUN_COMMAND_RE.is_match(command))
                        .unwrap_or(false)
                })
            })
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
            let text = strip_ts_comments(&text);
            if !text.contains("export default") && !text.contains("export const") {
                return false;
            }
            let bun_version_re =
                Regex::new(r#"bunVersion\s*:\s*["']([^"']+)["']"#).expect("valid regex");
            let runtime_re = Regex::new(r#"runtime\s*:\s*["']bun["']"#).expect("valid regex");
            if let Some(captures) = bun_version_re.captures(&text)
                && captures
                    .get(1)
                    .map(|value| !value.as_str().trim().is_empty())
                    .unwrap_or(false)
            {
                return true;
            }
            runtime_re.is_match(&text)
        })
        .unwrap_or(false)
}

pub(crate) fn strip_ts_comments(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_block = false;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_backtick = false;
    let mut template_brace_depth = 0;
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            output.push(ch);
            escape_next = false;
            continue;
        }

        if in_block {
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block = false;
            }
            continue;
        }

        // Inside a template expression: track braces and strip comments,
        // but still respect quotes inside the expression.
        if in_backtick && template_brace_depth > 0 {
            if ch == '{' {
                template_brace_depth += 1;
                output.push(ch);
                continue;
            }
            if ch == '}' {
                template_brace_depth -= 1;
                output.push(ch);
                continue;
            }
            if in_single_quote || in_double_quote {
                output.push(ch);
                if ch == '\\' {
                    escape_next = true;
                } else if in_single_quote && ch == '\'' {
                    in_single_quote = false;
                } else if in_double_quote && ch == '"' {
                    in_double_quote = false;
                }
                continue;
            }
            if ch == '\'' {
                in_single_quote = true;
                output.push(ch);
                continue;
            }
            if ch == '"' {
                in_double_quote = true;
                output.push(ch);
                continue;
            }
            // Fall through to normal comment handling
        } else if in_single_quote || in_double_quote || in_backtick {
            output.push(ch);
            if ch == '\\' {
                escape_next = true;
                continue;
            }
            if in_single_quote && ch == '\'' {
                in_single_quote = false;
            } else if in_double_quote && ch == '"' {
                in_double_quote = false;
            } else if in_backtick && template_brace_depth == 0 {
                if ch == '`' {
                    in_backtick = false;
                } else if ch == '$' && chars.peek() == Some(&'{') {
                    chars.next();
                    output.push('{');
                    template_brace_depth = 1;
                }
            }
            continue;
        }

        if ch == '/' {
            if chars.peek() == Some(&'*') {
                chars.next();
                in_block = true;
                continue;
            }
            if chars.peek() == Some(&'/') {
                chars.next();
                for next in chars.by_ref() {
                    if next == '\n' {
                        output.push('\n');
                        break;
                    }
                }
                continue;
            }
        }

        if ch == '\'' {
            in_single_quote = true;
        } else if ch == '"' {
            in_double_quote = true;
        } else if ch == '`' {
            in_backtick = true;
            template_brace_depth = 0;
        }

        output.push(ch);
    }

    output
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ts_comments_removes_line_comments() {
        let input = "const x = 1; // comment\nconst y = 2;";
        assert_eq!(strip_ts_comments(input), "const x = 1; \nconst y = 2;");
    }

    #[test]
    fn strip_ts_comments_removes_block_comments() {
        let input = "const x = /* block */ 1;";
        assert_eq!(strip_ts_comments(input), "const x =  1;");
    }

    #[test]
    fn strip_ts_comments_preserves_urls_in_strings() {
        let input = r#"destination: "https://example.com", bunVersion: "1.3.0""#;
        assert_eq!(strip_ts_comments(input), input);
    }

    #[test]
    fn strip_ts_comments_preserves_single_quoted_strings() {
        let input = "const s = 'http://example.com';";
        assert_eq!(strip_ts_comments(input), input);
    }

    #[test]
    fn strip_ts_comments_preserves_template_literals() {
        let input = "const t = `https://example.com/${path}`;";
        assert_eq!(strip_ts_comments(input), input);
    }

    #[test]
    fn strip_ts_comments_preserves_escaped_quotes() {
        let input = r#"const s = "he said \"hello\"";//end"#;
        assert_eq!(
            strip_ts_comments(input),
            r#"const s = "he said \"hello\"";"#
        );
    }

    #[test]
    fn strip_ts_comments_strips_comments_inside_template_expressions() {
        let input = "const t = `${
  // expr comment
  1 + 2
}`;";
        assert_eq!(strip_ts_comments(input), "const t = `${\n  \n  1 + 2\n}`;");
    }
}
