use crate::{
    context::SkillContext,
    state::PlatformPaths,
    types::{
        BUN_RELEASE_NOTES_URL, CapabilityClassification, CapabilityReport, REF_BUN_BUILTINS,
        REF_BUN_CAPABILITIES, REF_BUN_CLI, REF_BUN_PM_FALLBACKS, REF_BUN_RELEASE_NOTES,
        REF_VERCEL_BUN_RUNTIME, ReleaseReference, ReleaseSyncPreview, ReleaseSyncReport,
        VERCEL_BUN_RUNTIME_URL, VERIFIED_BUN_VERSION,
    },
};
use anyhow::{Context, Result, bail};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub fn run_release_sync(
    context: &SkillContext,
    paths: &PlatformPaths,
) -> Result<ReleaseSyncReport> {
    paths.ensure()?;

    let staged_root = temp_stage_root()?;
    copy_dir_all(&context.skill_root, &staged_root)?;
    let staged_context = SkillContext {
        skill_root: staged_root.clone(),
        rules_dir: staged_root.join("rules"),
        references_dir: staged_root.join("references"),
    };
    let updates = release_sync_updates(&staged_context)?;
    for (relative_path, content) in &updates {
        fs::write(staged_context.skill_path(relative_path), content)?;
    }

    let result: Result<ReleaseSyncReport> = (|| {
        check_skill_integrity(&staged_context)?;
        let report = create_release_sync_report(&staged_context)?;
        commit_release_sync_updates(context, &updates)?;
        paths.write_release_report(&report)?;
        Ok(report)
    })();
    let _ = fs::remove_dir_all(&staged_root);
    let report = result?;
    Ok(report)
}

pub fn preview_release_sync(context: &SkillContext) -> Result<ReleaseSyncPreview> {
    let candidates = release_sync_updates(context)?;

    let mut would_update = Vec::new();
    let mut unchanged = Vec::new();

    for (relative_path, next_content) in candidates {
        let path = context.skill_path(&relative_path);
        let relative = relative_path.display().to_string();
        let current = fs::read_to_string(&path).unwrap_or_default();
        if current == next_content {
            unchanged.push(relative);
        } else {
            would_update.push(relative);
        }
    }

    Ok(ReleaseSyncPreview {
        checked_at: iso_now()?,
        verified_bun_version: VERIFIED_BUN_VERSION.to_string(),
        would_update,
        unchanged,
        integrity_ok: check_skill_integrity(context).is_ok(),
    })
}

fn release_sync_updates(context: &SkillContext) -> Result<Vec<(PathBuf, String)>> {
    Ok(vec![
        (
            PathBuf::from("references").join(REF_BUN_RELEASE_NOTES),
            fetch_bun_release_snapshot()?,
        ),
        (
            PathBuf::from("references").join(REF_VERCEL_BUN_RUNTIME),
            fetch_vercel_runtime_snapshot()?,
        ),
        (
            PathBuf::from("rules/_index.md"),
            build_rules_index_content(context)?,
        ),
        (
            PathBuf::from("references/index.md"),
            build_references_index_content(),
        ),
    ])
}

fn commit_release_sync_updates(
    context: &SkillContext,
    updates: &[(PathBuf, String)],
) -> Result<()> {
    for (relative_path, content) in updates {
        let target = context.skill_path(relative_path);
        let tmp = target.with_extension("tmp-release-sync");
        fs::write(&tmp, content)?;
        fs::rename(&tmp, &target)
            .with_context(|| format!("failed to replace {}", target.display()))?;
    }
    Ok(())
}

fn temp_stage_root() -> Result<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    Ok(std::env::temp_dir().join(format!("bun-platform-release-sync-{nanos}")))
}

fn copy_dir_all(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let destination = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &destination)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

pub fn create_release_sync_report(context: &SkillContext) -> Result<ReleaseSyncReport> {
    let bun_release = read_or_empty(context.references_dir.join(REF_BUN_RELEASE_NOTES));
    let vercel_doc = read_or_empty(context.references_dir.join(REF_VERCEL_BUN_RUNTIME));
    let capabilities_doc = read_or_empty(context.references_dir.join(REF_BUN_CAPABILITIES));
    let rule_ids = context.list_rule_ids()?.into_iter().collect::<HashSet<_>>();

    let capability_specs = vec![
        (
            "bun webview automation",
            "bun-release-notes",
            regex_match(&bun_release, r"\bBun\.WebView\b")
                && regex_match(&capabilities_doc, r"\bBun\.WebView\b"),
            vec!["runtime-webview-automation", "runtime-bun-native-apis"],
        ),
        (
            "bun markdown ansi",
            "bun-release-notes",
            regex_match(&bun_release, r"\bBun\.markdown\.ansi\b")
                && regex_match(&capabilities_doc, r"\bmarkdown\.ansi\b"),
            vec!["runtime-markdown-entrypoints", "runtime-bun-native-apis"],
        ),
        (
            "bun cron callback overload",
            "bun-release-notes",
            regex_match(&bun_release, r"\bin-process\b")
                && regex_match(&capabilities_doc, r"\bBun\.cron\(schedule, handler\)\b"),
            vec!["runtime-cron-in-process-vs-os"],
        ),
        (
            "bun build compile browser",
            "bun-release-notes",
            regex_match(&bun_release, r"\bbun build --compile --target=browser\b"),
            vec!["build-bun-compile-browser", "build-bun-build-bundler"],
        ),
        (
            "bun parallel sequential scripts",
            "bun-release-notes",
            regex_match(&bun_release, r"--parallel|--sequential"),
            vec!["scripts-bun-run-parallel-sequential"],
        ),
        (
            "bun test retry",
            "bun-release-notes",
            regex_match(&bun_release, r"\bbun test --retry\b")
                || regex_match(&capabilities_doc, r"\b--retry\b"),
            vec!["test-bun-retry", "test-bun-test-runner"],
        ),
        (
            "vercel bunVersion and next scripts",
            "vercel-bun-runtime",
            regex_match(
                &vercel_doc,
                r"bunVersion|bun run --bun next dev|bun run --bun next build",
            ),
            vec![
                "vercel-bun-runtime-enable",
                "vercel-nextjs-bun-runtime-scripts",
            ],
        ),
    ];

    let capability_map = capability_specs
        .into_iter()
        .map(|(topic, source, matched, rules)| {
            let classification = if matched {
                if rules.iter().all(|rule| rule_ids.contains(*rule)) {
                    CapabilityClassification::CapabilityPresent
                } else {
                    CapabilityClassification::MissingRule
                }
            } else {
                CapabilityClassification::DocsOnly
            };
            CapabilityReport {
                topic: topic.to_string(),
                source: source.to_string(),
                matched,
                rules: rules.into_iter().map(ToOwned::to_owned).collect(),
                classification,
            }
        })
        .collect::<Vec<_>>();

    Ok(ReleaseSyncReport {
        synced_at: iso_now()?,
        verified_bun_version: VERIFIED_BUN_VERSION.to_string(),
        references: vec![
            ReleaseReference {
                file: REF_BUN_RELEASE_NOTES.to_string(),
                hash: hash_file(context.references_dir.join(REF_BUN_RELEASE_NOTES))?,
            },
            ReleaseReference {
                file: REF_BUN_CAPABILITIES.to_string(),
                hash: hash_file(context.references_dir.join(REF_BUN_CAPABILITIES))?,
            },
            ReleaseReference {
                file: REF_VERCEL_BUN_RUNTIME.to_string(),
                hash: hash_file(context.references_dir.join(REF_VERCEL_BUN_RUNTIME))?,
            },
        ],
        capability_map,
    })
}

pub fn check_skill_integrity(context: &SkillContext) -> Result<()> {
    let skill_md = context.read_skill_md()?;
    let ids = Regex::new(r"`([a-z0-9-]+)`")?
        .captures_iter(&skill_md)
        .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .collect::<HashSet<_>>();
    let rule_prefixes = [
        "pm-",
        "runtime-",
        "vercel-",
        "scripts-",
        "tsconfig-",
        "test-",
        "build-",
        "perf-",
        "migrate-",
        "troubleshooting-",
    ];
    let existing = context.list_rule_ids()?.into_iter().collect::<HashSet<_>>();
    let rule_shape_re = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)+$")?;
    let mut missing = Vec::new();

    for id in ids {
        if id.ends_with('-') {
            continue;
        }
        if !rule_shape_re.is_match(&id) {
            continue;
        }
        if !rule_prefixes.iter().any(|prefix| id.starts_with(prefix)) {
            continue;
        }
        if !existing.contains(&id) {
            missing.push(id);
        }
    }

    if !missing.is_empty() {
        bail!("missing rule files for ids:\n- {}", missing.join("\n- "));
    }
    if !context.skill_path("rules/_index.md").is_file() {
        bail!("missing rules/_index.md");
    }
    if !context.skill_path("references/index.md").is_file() {
        bail!("missing references/index.md");
    }
    if !context.is_references_flat()? {
        bail!("references/ must be flat");
    }
    Ok(())
}

fn build_rules_index_content(context: &SkillContext) -> Result<String> {
    let names = context.list_rule_ids()?;
    let groups = [
        ("Package Manager + Lockfiles (P1)", "pm-"),
        ("Runtime Selection (P1)", "runtime-"),
        ("Vercel Bun Runtime (P1)", "vercel-"),
        ("Scripts + Monorepos (P2)", "scripts-"),
        ("TypeScript + Tooling (P2)", "tsconfig-"),
        ("Testing (P3)", "test-"),
        ("Build + Bundling (P3)", "build-"),
        ("Performance (P4)", "perf-"),
        ("Migration (P5)", "migrate-"),
        ("Troubleshooting (P5)", "troubleshooting-"),
    ];
    let mut remaining = names.clone();
    let mut out = vec![
        "# Rules Index".to_string(),
        "".to_string(),
        "Open `SKILL.md` first to route by priority. Prefer opening specific rules over references."
            .to_string(),
        "".to_string(),
    ];

    for (title, prefix) in groups {
        let group_names = names
            .iter()
            .filter(|name| name.starts_with(prefix))
            .cloned()
            .collect::<Vec<_>>();
        if group_names.is_empty() {
            continue;
        }
        out.push(format!("## {title}"));
        out.push(String::new());
        for name in &group_names {
            out.push(format!("- `{name}`"));
        }
        out.push(String::new());
        remaining.retain(|name| !group_names.contains(name));
    }

    if !remaining.is_empty() {
        out.push("## Other".to_string());
        out.push(String::new());
        for name in remaining {
            out.push(format!("- `{name}`"));
        }
        out.push(String::new());
    }

    Ok(format!("{}\n", out.join("\n")))
}

fn build_references_index_content() -> String {
    let lines = vec![
        "# References Index".to_string(),
        "".to_string(),
        "Prefer rules for decisions, references for exact commands and API detail.".to_string(),
        "".to_string(),
        "Verified version pin:".to_string(),
        "".to_string(),
        format!("- Bun CLI `{VERIFIED_BUN_VERSION}`"),
        format!("- Bun release `v{VERIFIED_BUN_VERSION}`"),
        "".to_string(),
        "Refresh vendor-backed refs:".to_string(),
        "".to_string(),
        "```bash".to_string(),
        "bun-platform release-sync".to_string(),
        "bun-platform release-sync --status --format json".to_string(),
        "bun-platform release-sync --dry-run --format json".to_string(),
        "```".to_string(),
        "".to_string(),
        "## Bun".to_string(),
        "".to_string(),
        format!("- Bun release notes snapshot:\n  - `{REF_BUN_RELEASE_NOTES}`"),
        format!("- Bun capability map:\n  - `{REF_BUN_CAPABILITIES}`"),
        format!("- Bun CLI reference:\n  - `{REF_BUN_CLI}`"),
        format!("- Bun runtime APIs reference:\n  - `{REF_BUN_BUILTINS}`"),
        format!("- Package-manager fallback lanes:\n  - `{REF_BUN_PM_FALLBACKS}`"),
        "".to_string(),
        "## Vercel".to_string(),
        "".to_string(),
        format!("- Bun runtime docs:\n  - `{REF_VERCEL_BUN_RUNTIME}`"),
        "".to_string(),
        "## Fast Lookup".to_string(),
        "".to_string(),
        "```bash".to_string(),
        format!(
            "rg -n \"Bun.WebView|markdown\\\\.ansi|Bun\\\\.cron|availableParallelism|stripANSI\" ~/.agents/skills/bun-dev/references/{REF_BUN_CAPABILITIES}"
        ),
        format!(
            "rg -n \"bun (install|add|update|audit|outdated|pm|build|test|run)\" ~/.agents/skills/bun-dev/references/{REF_BUN_CLI}"
        ),
        format!(
            "rg -n \"Node runtime|pnpm|npm|Yarn|package manager only\" ~/.agents/skills/bun-dev/references/{REF_BUN_PM_FALLBACKS}"
        ),
        format!(
            "rg -n \"bunVersion|Bun\\\\.serve|runtime\" ~/.agents/skills/bun-dev/references/{REF_VERCEL_BUN_RUNTIME}"
        ),
        "```".to_string(),
        "".to_string(),
    ];
    format!("{}\n", lines.join("\n"))
}

fn fetch_bun_release_snapshot() -> Result<String> {
    fetch_markdown_snapshot(
        BUN_RELEASE_NOTES_URL,
        "bun-dev-skill/1.0 (+https://bun.com)",
    )
}

fn fetch_markdown_snapshot(url: &str, user_agent: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(10))
        .build()?;
    let markdown_url = resolve_markdown_url(&client, url)?;
    let body = client
        .get(&markdown_url)
        .header("accept", "text/markdown,text/plain;q=0.9,*/*;q=0.8")
        .send()?
        .error_for_status()?
        .text()?;
    Ok(tidy_markdown(&body))
}

fn fetch_vercel_runtime_snapshot() -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("bun-dev-skill/1.0 (+https://vercel.com)")
        .timeout(Duration::from_secs(10))
        .build()?;
    let html = client
        .get(VERCEL_BUN_RUNTIME_URL)
        .header("accept", "text/html,application/xhtml+xml")
        .send()?
        .error_for_status()?
        .text()?;
    match resolve_markdown_url(&client, VERCEL_BUN_RUNTIME_URL) {
        Ok(markdown_url) => {
            let markdown = client
                .get(markdown_url)
                .header("accept", "text/markdown,text/plain;q=0.9,*/*;q=0.8")
                .send()
                .ok()
                .and_then(|response| response.error_for_status().ok())
                .and_then(|response| response.text().ok())
                .map(|body| tidy_markdown(&body))
                .unwrap_or_else(|| html_to_markdown(&html));
            Ok(markdown)
        }
        Err(_) => Ok(html_to_markdown(&html)),
    }
}

fn resolve_markdown_url(client: &reqwest::blocking::Client, url: &str) -> Result<String> {
    if url.ends_with(".md") {
        return Ok(url.to_string());
    }
    let html = client
        .get(url)
        .header("accept", "text/html,application/xhtml+xml")
        .send()?
        .error_for_status()?
        .text()?;
    let href = Regex::new(
        r#"<link\b[^>]*\btype=["']text/markdown["'][^>]*\bhref=["']([^"']+)["'][^>]*>"#,
    )?
    .captures_iter(&html)
    .find_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
    .context("no markdown alternate link found")?;
    Ok(reqwest::Url::parse(url)?.join(&href)?.to_string())
}

fn tidy_markdown(markdown: &str) -> String {
    let mut output = markdown.replace('\r', "");
    if output.starts_with("---\n")
        && let Some(index) = output[4..].find("\n---\n")
    {
        output = output[(4 + index + 5)..].to_string();
    }

    let liquid_tag_re = Regex::new(r"^\{%\s*.*\s*%\}$").expect("valid regex");
    let collapse_breaks_re = Regex::new(r"\n{3,}").expect("valid regex");

    output = output
        .lines()
        .filter(|line| !liquid_tag_re.is_match(line.trim()))
        .collect::<Vec<_>>()
        .join("\n");
    let collapsed = collapse_breaks_re
        .replace_all(output.trim(), "\n\n")
        .to_string();
    format!("{}\n", collapsed.trim())
}

fn html_to_markdown(html: &str) -> String {
    let main_re = Regex::new(r"(?is)<main\b[^>]*>(.*?)</main>").expect("valid regex");
    let mut content = main_re
        .captures(html)
        .and_then(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .unwrap_or_else(|| html.to_string());

    for pattern in [
        r"(?is)<script\b[^>]*>.*?</script>",
        r"(?is)<style\b[^>]*>.*?</style>",
        r"(?is)<svg\b[^>]*>.*?</svg>",
    ] {
        let re = Regex::new(pattern).expect("valid regex");
        content = re.replace_all(&content, "").to_string();
    }

    let heading_re = Regex::new(r"(?is)<h([1-6])\b[^>]*>(.*?)</h[1-6]>").expect("valid regex");
    content = heading_re
        .replace_all(&content, |captures: &regex::Captures| {
            let level = captures[1].parse::<usize>().unwrap_or(1);
            format!(
                "\n{} {}\n",
                "#".repeat(level),
                strip_tags(&decode_entities(&captures[2]))
            )
        })
        .to_string();

    let list_item_re = Regex::new(r"(?is)<li\b[^>]*>(.*?)</li>").expect("valid regex");
    content = list_item_re
        .replace_all(&content, |captures: &regex::Captures| {
            format!("\n- {}", strip_tags(&decode_entities(&captures[1])))
        })
        .to_string();

    for pattern in [
        r"(?i)<(p|div|section|article|pre|table|tr)\b[^>]*>",
        r"(?i)</(p|div|section|article|pre|table|tr)>",
    ] {
        let re = Regex::new(pattern).expect("valid regex");
        content = re.replace_all(&content, "\n").to_string();
    }

    let code_re = Regex::new(r"(?is)<code\b[^>]*>(.*?)</code>").expect("valid regex");
    content = code_re
        .replace_all(&content, |captures: &regex::Captures| {
            format!("`{}`", decode_entities(&captures[1]))
        })
        .to_string();

    let link_re =
        Regex::new(r#"(?is)<a\b[^>]*href=["']([^"']+)["'][^>]*>(.*?)</a>"#).expect("valid regex");
    content = link_re
        .replace_all(&content, |captures: &regex::Captures| {
            let text = strip_tags(&decode_entities(&captures[2]));
            if text.is_empty() {
                captures[1].to_string()
            } else {
                format!("[{text}]({})", &captures[1])
            }
        })
        .to_string();

    let tag_re = Regex::new(r"(?is)<[^>]+>").expect("valid regex");
    content = tag_re.replace_all(&content, " ").to_string();

    tidy_markdown(&decode_entities(&content))
}

fn strip_tags(input: &str) -> String {
    let tag_re = Regex::new(r"(?is)<[^>]+>").expect("valid regex");
    tag_re
        .replace_all(input, " ")
        .to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn hash_file(path: PathBuf) -> Result<String> {
    if !path.is_file() {
        return Ok("missing".to_string());
    }
    let data = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

fn read_or_empty(path: PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn regex_match(content: &str, pattern: &str) -> bool {
    Regex::new(pattern)
        .map(|re| re.is_match(content))
        .unwrap_or(false)
}

fn iso_now() -> Result<String> {
    Ok(OffsetDateTime::now_utc().format(&Rfc3339)?)
}
