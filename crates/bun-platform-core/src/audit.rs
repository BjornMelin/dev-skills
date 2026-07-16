use crate::{
    config::AuditConfig,
    fixes::{contains_bun_runtime_config, vercel_ts_has_bun_runtime},
    state::PlatformPaths,
    types::{Confidence, Finding, Severity},
};
use anyhow::{Context, Result, bail};
use regex::Regex;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::UNIX_EPOCH,
};
use walkdir::{DirEntry, WalkDir};

const ENGINE_VERSION: &str = "bun-platform-rust-v1";
static OTHER_PACKAGE_MANAGER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(npm|pnpm|yarn)\b").expect("valid regex"));
static ORCHESTRATION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bconcurrently\b|\bnpm-run-all\b|\brun-p\b").expect("valid regex")
});
static TS_RUNTIME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(ts-node|tsx)\b").expect("valid regex"));
static BUN_COMMAND_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bbun(x)?\b").expect("valid regex"));
static BUN_RUN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bbun\s+run\s+").expect("valid regex"));
static POSITIVE_INTEGER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[1-9]\d*$").expect("valid regex"));

#[derive(Clone, Debug, Default)]
struct RepoSignals {
    bun_first: bool,
    vercel_bun_enabled: bool,
    has_workspaces: bool,
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(rename = "packageManager")]
    package_manager: Option<String>,
    scripts: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
    workspaces: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct TsConfig {
    #[serde(rename = "compilerOptions")]
    compiler_options: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Clone)]
struct RepoSnapshot<'a> {
    root: &'a Path,
    config: &'a AuditConfig,
}

pub fn run_audit(root: &Path, config: &AuditConfig, paths: &PlatformPaths) -> Result<Vec<Finding>> {
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", root.display()))?;
    let snapshot = RepoSnapshot {
        root: &root,
        config,
    };
    let fingerprint = build_repo_fingerprint(&snapshot)?;
    if config.write_cache {
        paths.ensure()?;
        if let Some(cached) = paths.read_cache(&root, &fingerprint)? {
            let findings = serde_json::from_str::<Vec<Finding>>(&cached)
                .context("failed to parse cached findings")?;
            return Ok(findings);
        }
    }

    let signals = infer_signals(&snapshot)?;
    let mut findings = Vec::new();
    let package_json_path = root.join("package.json");
    let package_json = snapshot.read_json::<PackageJson>(&package_json_path)?;
    let tsconfig = snapshot.read_jsonc::<TsConfig>(&root.join("tsconfig.json"))?;
    let bunfig = snapshot.read_text(&root.join("bunfig.toml"))?;
    let gitignore = snapshot.read_text(&root.join(".gitignore"))?;

    let lockfiles = [
        "bun.lockb",
        "bun.lock",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
    ]
    .iter()
    .filter(|name| root.join(name).is_file())
    .map(|name| name.to_string())
    .collect::<Vec<_>>();
    if lockfiles.len() > 1 {
        findings.push(create_finding(
            &root,
            "pm-no-mixed-lockfiles",
            Severity::Error,
            &root.join(&lockfiles[0]),
            &format!(
                "Multiple lockfiles detected: {}. Pick one package manager and delete the others.",
                lockfiles.join(", ")
            ),
            FindingOptions {
                suggested_fix: Some(
                    "Remove non-Bun lockfiles and keep Bun's lockfile as the single source of truth."
                        .to_string(),
                ),
                ..Default::default()
            },
        ));
    }

    if let Some(package_json) = package_json.as_ref() {
        if package_json.package_manager.is_none() {
            findings.push(create_finding(
                &root,
                "pm-package-manager-field",
                Severity::Info,
                &package_json_path,
                "Missing `packageManager` field.",
                FindingOptions {
                    suggested_fix: Some(format!(
                        "Add `\"packageManager\": \"bun@{}\"` when the repo is Bun-first.",
                        crate::types::VERIFIED_BUN_VERSION
                    )),
                    ..Default::default()
                },
            ));
        } else if signals.bun_first
            && !package_json
                .package_manager
                .as_deref()
                .unwrap_or_default()
                .starts_with("bun@")
        {
            findings.push(create_finding(
                &root,
                "pm-package-manager-field",
                Severity::Warn,
                &package_json_path,
                &format!(
                    "packageManager is \"{}\". If this repo is Bun-first, set it to \"bun@{}\".",
                    package_json.package_manager.as_deref().unwrap_or_default(),
                    crate::types::VERIFIED_BUN_VERSION
                ),
                FindingOptions::default(),
            ));
        }

        for (name, command) in package_json.scripts.clone().unwrap_or_default() {
            if signals.bun_first && command.contains("npx") {
                findings.push(create_finding(
                    &root,
                    "pm-bunx-vs-npx",
                    Severity::Warn,
                    &package_json_path,
                    &format!("Script \"{name}\" uses npx. Prefer bunx."),
                    FindingOptions {
                        suggested_fix: Some(
                            "Replace `npx` with `bunx` in package.json scripts.".to_string(),
                        ),
                        ..Default::default()
                    },
                ));
            }
            if signals.bun_first && OTHER_PACKAGE_MANAGER_RE.is_match(&command) {
                findings.push(create_finding(
          &root,
          "scripts-no-npm-in-bun-repos",
          Severity::Warn,
          &package_json_path,
          &format!(
            "Script \"{name}\" uses another package manager. Prefer bun install/bun run/bunx for consistency."
          ),
          FindingOptions::default(),
        ));
            }
            if ORCHESTRATION_RE.is_match(&command) {
                findings.push(create_finding(
                    &root,
                    "scripts-bun-run-parallel-sequential",
                    Severity::Info,
                    &package_json_path,
                    &format!(
            "Script \"{name}\" uses an orchestration package. Bun supports --parallel/--sequential."
          ),
                    FindingOptions::default(),
                ));
            }
            if command.contains("nodemon") {
                findings.push(create_finding(
                    &root,
                    "runtime-watch-and-hot-reload",
                    Severity::Info,
                    &package_json_path,
                    &format!("Script \"{name}\" uses nodemon. Bun has built-in --watch/--hot."),
                    FindingOptions::default(),
                ));
            }
            if TS_RUNTIME_RE.is_match(&command) {
                findings.push(create_finding(
          &root,
          "runtime-ts-direct-execution",
          Severity::Info,
          &package_json_path,
          &format!(
            "Script \"{name}\" uses ts-node/tsx. If Bun is your runtime, prefer running TS directly with bun."
          ),
          FindingOptions::default(),
        ));
            }
            if signals.vercel_bun_enabled && name == "dev" && command.trim() == "next dev" {
                findings.push(create_finding(
          &root,
          "vercel-nextjs-bun-runtime-scripts",
          Severity::Warn,
          &package_json_path,
          "Vercel Bun runtime is enabled, but the dev script still uses `next dev` directly.",
          FindingOptions {
            suggested_fix: Some("Use `bun run --bun next dev`.".to_string()),
            ..Default::default()
          },
        ));
            }
            if signals.vercel_bun_enabled && name == "build" && command.trim() == "next build" {
                findings.push(create_finding(
          &root,
          "vercel-nextjs-bun-runtime-scripts",
          Severity::Warn,
          &package_json_path,
          "Vercel Bun runtime is enabled, but the build script still uses `next build` directly.",
          FindingOptions {
            suggested_fix: Some("Use `bun run --bun next build`.".to_string()),
            ..Default::default()
          },
        ));
            }
        }

        let has_bun_types = package_json
            .dev_dependencies
            .as_ref()
            .map(|deps| deps.contains_key("@types/bun"))
            .unwrap_or(false);
        if let Some(tsconfig) = tsconfig.as_ref() {
            let compiler_options = tsconfig.compiler_options.clone().unwrap_or_default();
            if let Some(module_resolution) = compiler_options
                .get("moduleResolution")
                .and_then(|value| value.as_str())
                && !module_resolution.eq_ignore_ascii_case("bundler")
            {
                findings.push(create_finding(
            &root,
            "tsconfig-module-resolution-bundler",
            Severity::Warn,
            &root.join("tsconfig.json"),
            &format!(
              "compilerOptions.moduleResolution is \"{module_resolution}\". Bun generally expects \"Bundler\"."
            ),
            FindingOptions::default(),
          ));
            }

            let recommended = [("target", "ESNext"), ("module", "Preserve")];
            let mut missing = Vec::new();
            for (key, expected) in recommended {
                if compiler_options
                    .get(key)
                    .and_then(|value| value.as_str())
                    .map(|value| !value.eq_ignore_ascii_case(expected))
                    .unwrap_or(true)
                {
                    missing.push(format!("{key}: \"{expected}\""));
                }
            }
            for key in [
                "allowImportingTsExtensions",
                "verbatimModuleSyntax",
                "noEmit",
            ] {
                if compiler_options.get(key).and_then(|value| value.as_bool()) != Some(true) {
                    missing.push(format!("{key}: true"));
                }
            }
            if !missing.is_empty() {
                findings.push(create_finding(
                    &root,
                    "tsconfig-bun-recommended",
                    Severity::Info,
                    &root.join("tsconfig.json"),
                    &format!(
                        "tsconfig.json is missing Bun-friendly options: {}.",
                        missing.join(", ")
                    ),
                    FindingOptions::default(),
                ));
            }

            let types = compiler_options
                .get("types")
                .and_then(|value| value.as_array())
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if (has_bun_types || signals.bun_first)
                && !types.iter().any(|value| value == "bun-types")
            {
                findings.push(create_finding(
                    &root,
                    "tsconfig-bun-types",
                    Severity::Info,
                    &root.join("tsconfig.json"),
                    "Consider adding compilerOptions.types: [\"bun-types\"] for Bun globals/types.",
                    FindingOptions::default(),
                ));
            }
        }
    }

    if gitignore
        .as_deref()
        .unwrap_or_default()
        .contains("bun.lockb")
        || gitignore
            .as_deref()
            .unwrap_or_default()
            .contains("bun.lock")
    {
        findings.push(create_finding(
            &root,
            "pm-commit-bun-lockb",
            Severity::Warn,
            &root.join(".gitignore"),
            "Bun lockfiles are ignored. Prefer committing bun.lock or bun.lockb for deterministic installs.",
            FindingOptions::default(),
        ));
    }

    if root.join(".nvmrc").is_file() && has_bun_lockfile(&root) {
        findings.push(create_finding(
      &root,
      "runtime-bun-vs-node-choose",
      Severity::Info,
      &root.join(".nvmrc"),
      "Found both .nvmrc and a Bun lockfile. If Node is not required, consider removing Node-only runtime pinning.",
      FindingOptions::default(),
    ));
    }

    if let Some(bunfig) = bunfig {
        if toml_table_has_positive_integer(&bunfig, "test", "retry") {
            findings.push(create_finding(
        &root,
        "test-bun-retry",
        Severity::Info,
        &root.join("bunfig.toml"),
        "bunfig.toml sets a default test retry count. Keep retries low and prefer fixing flaky tests.",
        FindingOptions {
          confidence: Confidence::Medium,
          ..Default::default()
        },
      ));
        }
        if has_bun_lockfile(&root)
            && toml_table_has_bool(&bunfig, "install", "frozenLockfile", false)
        {
            findings.push(create_finding(
        &root,
        "pm-bun-install-ci-frozen-lockfile",
        Severity::Info,
        &root.join("bunfig.toml"),
        "bunfig.toml explicitly disables frozen lockfile installs. CI should prefer frozen lockfile mode.",
        FindingOptions {
          suggested_fix: Some(
            "Use `bun install --frozen-lockfile` or `bun ci` in CI.".to_string(),
          ),
          ..Default::default()
        },
      ));
        }
        if signals.bun_first && toml_table_has_bool(&bunfig, "run", "bun", false) {
            findings.push(create_finding(
        &root,
        "runtime-bun-run-bun-flag",
        Severity::Info,
        &root.join("bunfig.toml"),
        "bunfig.toml disables run.bun. Binaries with Node shebangs may execute under Node instead of Bun.",
        FindingOptions {
          suggested_fix: Some(
            "Enable `[run] bun = true` when you intentionally want Bun to execute Node-shebang binaries."
              .to_string(),
          ),
          ..Default::default()
        },
      ));
        }
    }

    findings.extend(run_adapters(&snapshot, &signals)?);
    let findings = normalize_findings(findings, config);
    if config.write_cache {
        paths.write_cache(&root, &fingerprint, &serde_json::to_string(&findings)?)?;
    }
    Ok(findings)
}

#[derive(Default)]
struct FindingOptions {
    line: Option<usize>,
    column: Option<usize>,
    snippet: Option<String>,
    why: Option<String>,
    suggested_fix: Option<String>,
    confidence: Confidence,
}

fn create_finding(
    root: &Path,
    rule_id: &str,
    severity: Severity,
    file: &Path,
    message: &str,
    options: FindingOptions,
) -> Finding {
    let relative = file
        .strip_prefix(root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| file.to_path_buf());
    Finding {
        rule_id: rule_id.to_string(),
        category: get_rule_category(rule_id).to_string(),
        severity,
        confidence: options.confidence,
        file: root.join(&relative).display().to_string(),
        line: options.line.unwrap_or(1),
        column: options.column.unwrap_or(1),
        message: message.to_string(),
        why: options.why,
        suggested_fix: options.suggested_fix,
        snippet: options.snippet,
        suppression_key: format!("{rule_id}:{}", relative.display()),
    }
}

fn get_rule_category(rule_id: &str) -> &'static str {
    match rule_id.split('-').next().unwrap_or("other") {
        "pm" => "package-manager",
        "runtime" => "runtime",
        "vercel" => "vercel",
        "scripts" => "scripts",
        "tsconfig" => "typescript",
        "test" => "testing",
        "build" => "build",
        "perf" => "performance",
        "migrate" => "migration",
        "troubleshooting" => "troubleshooting",
        _ => "other",
    }
}

impl<'a> RepoSnapshot<'a> {
    fn read_text(&self, path: &Path) -> Result<Option<String>> {
        match fs::read_to_string(path) {
            Ok(text) => Ok(Some(text)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
        }
    }

    fn read_json<T: DeserializeOwned>(&self, path: &Path) -> Result<Option<T>> {
        let Some(text) = self.read_text(path)? else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_str::<T>(&text).with_context(
            || format!("failed to parse {}", path.display()),
        )?))
    }

    fn read_jsonc<T: DeserializeOwned>(&self, path: &Path) -> Result<Option<T>> {
        let Some(text) = self.read_text(path)? else {
            return Ok(None);
        };
        let options = jsonc_parser::ParseOptions {
            allow_comments: true,
            allow_loose_object_property_names: false,
            allow_trailing_commas: true,
            allow_missing_commas: false,
            allow_single_quoted_strings: false,
            allow_hexadecimal_numbers: false,
            allow_unary_plus_numbers: false,
        };
        Ok(Some(
            jsonc_parser::parse_to_serde_value::<T>(&text, &options)
                .with_context(|| format!("failed to parse {}", path.display()))?,
        ))
    }

    fn walk_files(&self) -> Result<Vec<PathBuf>> {
        let include_paths = self
            .config
            .include_paths
            .iter()
            .map(|path| path.canonicalize().unwrap_or_else(|_| path.clone()))
            .collect::<Vec<_>>();
        let mut total_bytes = 0u64;
        let mut files = Vec::new();
        for entry in WalkDir::new(self.root)
            .into_iter()
            .filter_entry(|entry| self.allow_entry(entry))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path().to_path_buf();
            if !include_paths.is_empty()
                && !include_paths
                    .iter()
                    .any(|candidate| path == *candidate || path.starts_with(candidate))
            {
                continue;
            }
            let metadata = entry.metadata()?;
            total_bytes = total_bytes.saturating_add(metadata.len());
            if files.len() >= self.config.max_files || total_bytes > self.config.max_bytes {
                bail!(
                    "Repo scan limits exceeded (files={}, bytes={}). Increase --max-files/--max-bytes or narrow the scope with --include.",
                    files.len(),
                    total_bytes
                );
            }
            files.push(path);
        }
        Ok(files)
    }

    fn allow_entry(&self, entry: &DirEntry) -> bool {
        if entry.depth() == 0 {
            return true;
        }
        let name = entry.file_name().to_string_lossy();
        !entry.file_type().is_dir() || !self.config.exclude_dirs.iter().any(|value| value == &name)
    }
}

fn build_repo_fingerprint(snapshot: &RepoSnapshot<'_>) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(ENGINE_VERSION.as_bytes());
    hasher.update(serde_json::to_vec(&serde_json::json!({
      "disabledRules": snapshot.config.disabled_rules,
      "severityOverrides": snapshot.config.severity_overrides,
      "excludeDirs": snapshot.config.exclude_dirs,
      "baselineKeys": snapshot.config.baseline_keys,
      "includePaths": snapshot
        .config
        .include_paths
        .iter()
        .map(|value| value.display().to_string())
        .collect::<Vec<_>>(),
      "adapters": snapshot.config.adapters,
      "maxFiles": snapshot.config.max_files,
      "maxBytes": snapshot.config.max_bytes,
    }))?);
    // Hash unconditional root inputs so cache invalidates when they change
    // even if the scan is scoped with --include.
    for name in [
        "package.json",
        "tsconfig.json",
        "bunfig.toml",
        ".gitignore",
        ".nvmrc",
        "bun.lock",
        "bun.lockb",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "middleware.ts",
        "vercel.json",
        "vercel.ts",
    ] {
        let path = snapshot.root.join(name);
        hasher.update(name.as_bytes());
        hasher.update([u8::from(path.is_file())]);
        if let Ok(content) = fs::read(&path) {
            hasher.update(&content);
        }
    }

    let mut files = snapshot.walk_files()?;
    files.sort();
    for path in files {
        let metadata = fs::metadata(&path)?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|value| value.as_millis())
            .unwrap_or(0);
        hasher.update(
            path.strip_prefix(snapshot.root)
                .unwrap_or(&path)
                .display()
                .to_string()
                .as_bytes(),
        );
        hasher.update(metadata.len().to_string().as_bytes());
        hasher.update(modified.to_string().as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn infer_signals(snapshot: &RepoSnapshot<'_>) -> Result<RepoSignals> {
    let package_json = snapshot
        .read_json::<PackageJson>(&snapshot.root.join("package.json"))
        .ok()
        .flatten();
    let vercel_bun_enabled = detect_vercel_bun_enabled(snapshot)?;
    let scripts = package_json
        .as_ref()
        .and_then(|pkg| pkg.scripts.clone())
        .unwrap_or_default()
        .into_values()
        .collect::<Vec<_>>();
    let has_workspaces = package_json
        .as_ref()
        .and_then(|pkg| pkg.workspaces.clone())
        .map(|value| !value.is_null())
        .unwrap_or(false);
    let bun_first = has_bun_lockfile(snapshot.root)
        || package_json
            .as_ref()
            .and_then(|pkg| pkg.package_manager.as_deref())
            .map(|value| value.starts_with("bun@"))
            .unwrap_or(false)
        || vercel_bun_enabled
        || scripts.iter().any(|value| BUN_COMMAND_RE.is_match(value));
    Ok(RepoSignals {
        bun_first,
        vercel_bun_enabled,
        has_workspaces,
    })
}

fn detect_vercel_bun_enabled(snapshot: &RepoSnapshot<'_>) -> Result<bool> {
    let vercel_json_path = snapshot.root.join("vercel.json");
    match snapshot.read_json::<serde_json::Value>(&vercel_json_path)? {
        Some(json) => Ok(contains_bun_runtime_config(&json)),
        None => Ok(snapshot
            .read_text(&snapshot.root.join("vercel.ts"))?
            .map(|text| vercel_ts_has_bun_runtime(&text))
            .unwrap_or(false)),
    }
}

fn normalize_findings(mut findings: Vec<Finding>, config: &AuditConfig) -> Vec<Finding> {
    let mut seen = HashSet::new();
    findings.retain(|finding| {
        if config
            .disabled_rules
            .iter()
            .any(|value| value == &finding.rule_id)
        {
            return false;
        }
        if config
            .baseline_keys
            .iter()
            .any(|value| value == &finding.suppression_key)
        {
            return false;
        }
        true
    });
    for finding in &mut findings {
        if let Some(override_severity) = config.severity_overrides.get(&finding.rule_id) {
            finding.severity = *override_severity;
        }
    }
    findings.retain(|finding| {
        let key = format!(
            "{}::{}::{}::{}::{}",
            finding.rule_id, finding.file, finding.line, finding.column, finding.message
        );
        seen.insert(key)
    });
    findings.sort_by(|left, right| {
        right
            .severity
            .rank()
            .cmp(&left.severity.rank())
            .then(left.file.cmp(&right.file))
            .then(left.line.cmp(&right.line))
            .then(left.column.cmp(&right.column))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    findings
}

fn run_adapters(snapshot: &RepoSnapshot<'_>, signals: &RepoSignals) -> Result<Vec<Finding>> {
    let requested = resolve_adapter_ids(signals, &snapshot.config.adapters);
    let mut findings = Vec::new();
    let files = if requested.contains("github-actions") || requested.contains("docker") {
        snapshot.walk_files()?
    } else {
        Vec::new()
    };
    if requested.contains("vercel") {
        findings.extend(run_vercel_adapter(snapshot, signals)?);
    }
    if requested.contains("github-actions") {
        findings.extend(run_github_actions_adapter(snapshot, signals, &files)?);
    }
    if requested.contains("docker") && files.iter().any(|path| is_dockerfile(path)) {
        findings.extend(run_docker_adapter(snapshot, signals, &files)?);
    }
    if requested.contains("monorepo") && signals.has_workspaces {
        findings.extend(run_monorepo_adapter(snapshot)?);
    }
    Ok(findings)
}

fn resolve_adapter_ids(signals: &RepoSignals, configured: &[String]) -> HashSet<String> {
    if configured.is_empty() || configured.iter().any(|value| value == "auto") {
        let mut auto = HashSet::from(["github-actions".to_string(), "docker".to_string()]);
        if signals.vercel_bun_enabled {
            auto.insert("vercel".to_string());
        }
        if signals.has_workspaces {
            auto.insert("monorepo".to_string());
        }
        return auto;
    }
    configured.iter().cloned().collect()
}

fn run_vercel_adapter(snapshot: &RepoSnapshot<'_>, signals: &RepoSignals) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    let root = snapshot.root;
    let vercel_json_path = root.join("vercel.json");
    if let Some(vercel_json) = snapshot.read_json::<serde_json::Value>(&vercel_json_path)? {
        if let Some(version) = vercel_json
            .get("bunVersion")
            .and_then(|value| value.as_str())
        {
            if version != "1.x" {
                findings.push(create_finding(
                    root,
                    "vercel-bun-runtime-enable",
                    Severity::Warn,
                    &vercel_json_path,
                    &format!(
            "bunVersion is \"{version}\". Prefer \"1.x\" unless you have a strong pinning reason."
          ),
                    FindingOptions::default(),
                ));
            }
        } else {
            findings.push(create_finding(
        root,
        "vercel-bun-runtime-enable",
        Severity::Info,
        &vercel_json_path,
        "vercel.json exists but no bunVersion was detected. If you intend to use Bun runtime, set bunVersion.",
        FindingOptions::default(),
      ));
        }
    }
    if signals.vercel_bun_enabled && !has_bun_lockfile(root) {
        findings.push(create_finding(
      root,
      "vercel-bun-install-detection",
      Severity::Warn,
      &vercel_json_path,
      "Bun runtime is enabled but no Bun lockfile is committed. Add and commit bun.lock or bun.lockb to ensure Bun installs on Vercel.",
      FindingOptions::default(),
    ));
    }

    if signals.vercel_bun_enabled
        && let Some(middleware) = snapshot.read_text(&root.join("middleware.ts"))?
        && !Regex::new(r#"runtime\s*=\s*['"]nodejs['"]"#)?.is_match(&middleware)
    {
        findings.push(create_finding(
        root,
        "vercel-bun-runtime-limitations",
        Severity::Info,
        &root.join("middleware.ts"),
        "When using Vercel Routing Middleware with Bun runtime, middleware should declare the nodejs runtime.",
        FindingOptions {
          confidence: Confidence::Medium,
          ..Default::default()
        },
      ));
    }

    if signals.vercel_bun_enabled {
        let bun_serve = Regex::new(r"\bBun\.serve\s*\(")?;
        for file in snapshot.walk_files()? {
            if !is_js_like(&file) {
                continue;
            }
            let Some(content) = snapshot.read_text(&file)? else {
                continue;
            };
            if let Some(hit) = find_first(&content, &bun_serve) {
                findings.push(create_finding(
          root,
          "vercel-bun-runtime-limitations",
          Severity::Warn,
          &file,
          "Bun.serve() is not supported in Vercel Functions; use a supported handler or framework adapter.",
          FindingOptions {
            line: Some(hit.line),
            column: Some(hit.column),
            snippet: Some(hit.snippet),
            ..Default::default()
          },
        ));
            }
        }
    }

    Ok(findings)
}

fn run_github_actions_adapter(
    snapshot: &RepoSnapshot<'_>,
    signals: &RepoSignals,
    files: &[PathBuf],
) -> Result<Vec<Finding>> {
    let workflow_dir = snapshot.root.join(".github/workflows");
    let mut findings = Vec::new();
    let install_re = Regex::new(r"\b(npm ci|npm install|pnpm install|yarn install)\b")?;
    let npx_re = Regex::new(r"\bnpx\b")?;
    let bun_install_re = Regex::new(r"\bbun install\b")?;
    let bun_frozen_re = Regex::new(r"(?s)\bbun install\b.*--frozen-lockfile\b|\bbun ci\b")?;
    if !workflow_dir.is_dir() {
        return Ok(findings);
    }
    for path in files {
        if !path.starts_with(&workflow_dir) {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if !name.ends_with(".yml") && !name.ends_with(".yaml") {
            continue;
        }
        let Some(content) = snapshot.read_text(path)? else {
            continue;
        };
        if signals.bun_first && install_re.is_match(&content) {
            findings.push(create_finding(
                snapshot.root,
                "scripts-no-npm-in-bun-repos",
                Severity::Warn,
                path,
                "GitHub Actions workflow uses npm/pnpm/yarn install steps in a Bun-first repo.",
                line_hit(&content, &install_re),
            ));
        }
        if npx_re.is_match(&content) {
            findings.push(create_finding(
                snapshot.root,
                "pm-bunx-vs-npx",
                Severity::Warn,
                path,
                "GitHub Actions workflow uses npx. Prefer bunx in Bun-first repos.",
                line_hit(&content, &npx_re),
            ));
        }
        if has_bun_lockfile(snapshot.root)
            && let Some(mut options) =
                find_first_unfrozen_bun_install(&content, &bun_install_re, &bun_frozen_re)
        {
            options.suggested_fix =
                Some("Prefer `bun install --frozen-lockfile` or `bun ci` in CI.".to_string());
            findings.push(create_finding(
                snapshot.root,
                "pm-bun-install-ci-frozen-lockfile",
                Severity::Info,
                path,
                "GitHub Actions workflow runs `bun install` without frozen lockfile mode.",
                options,
            ));
        }
    }
    Ok(findings)
}

fn run_docker_adapter(
    snapshot: &RepoSnapshot<'_>,
    signals: &RepoSignals,
    files: &[PathBuf],
) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    let node_from_re = Regex::new(r"\bFROM\s+node[:\s]")?;
    let install_re = Regex::new(r"\b(npm ci|npm install|pnpm install|yarn install)\b")?;
    let bun_install_re = Regex::new(r"\bbun install\b")?;
    let bun_frozen_re = Regex::new(r"(?s)\bbun install\b.*--frozen-lockfile\b|\bbun ci\b")?;
    for file in files {
        if !is_dockerfile(file) {
            continue;
        }
        let Some(content) = snapshot.read_text(file)? else {
            continue;
        };
        if signals.bun_first && node_from_re.is_match(&content) {
            let mut options = line_hit(&content, &node_from_re);
            options.suggested_fix = Some(
                "Use an explicit Bun image or document why Node remains required.".to_string(),
            );
            findings.push(create_finding(
                snapshot.root,
                "runtime-bun-vs-node-choose",
                Severity::Info,
                file,
                "Dockerfile still uses a Node base image in a Bun-first repo.",
                options,
            ));
        }
        if signals.bun_first && install_re.is_match(&content) {
            findings.push(create_finding(
                snapshot.root,
                "scripts-no-npm-in-bun-repos",
                Severity::Warn,
                file,
                "Dockerfile uses npm/pnpm/yarn install commands in a Bun-first repo.",
                line_hit(&content, &install_re),
            ));
        }
        if has_bun_lockfile(snapshot.root)
            && let Some(mut options) =
                find_first_unfrozen_bun_install(&content, &bun_install_re, &bun_frozen_re)
        {
            options.suggested_fix =
                Some("Prefer `bun install --frozen-lockfile` or `bun ci` in CI.".to_string());
            findings.push(create_finding(
                snapshot.root,
                "pm-bun-install-ci-frozen-lockfile",
                Severity::Info,
                file,
                "Dockerfile runs `bun install` without frozen lockfile mode.",
                options,
            ));
        }
    }
    Ok(findings)
}

fn run_monorepo_adapter(snapshot: &RepoSnapshot<'_>) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    let filter_re = Regex::new(r"--filter|--workspaces")?;
    let package_json = snapshot.read_json::<PackageJson>(&snapshot.root.join("package.json"))?;
    for (name, command) in package_json.and_then(|pkg| pkg.scripts).unwrap_or_default() {
        if !["build", "test", "lint", "typecheck"].contains(&name.as_str()) {
            continue;
        }
        if BUN_RUN_RE.is_match(&command) && !filter_re.is_match(&command) {
            findings.push(create_finding(
        snapshot.root,
        "scripts-bun-filter-and-workspaces",
        Severity::Info,
        &snapshot.root.join("package.json"),
        &format!(
          "Root monorepo script \"{name}\" runs via bun without --filter/--workspaces."
        ),
        FindingOptions {
          suggested_fix: Some(
            "Use `bun run --workspaces <script>` or `bun run --filter <glob> <script>` when coordinating workspace tasks."
              .to_string(),
          ),
          ..Default::default()
        },
      ));
        }
    }
    Ok(findings)
}

fn is_dockerfile(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.starts_with("Dockerfile"))
        .unwrap_or(false)
}

fn has_bun_lockfile(root: &Path) -> bool {
    root.join("bun.lockb").is_file() || root.join("bun.lock").is_file()
}

fn is_js_like(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs")
    )
}

fn toml_table_has_positive_integer(content: &str, table: &str, key: &str) -> bool {
    let target_header = format!("[{table}]");
    let key_prefix = format!("{key}=");
    let mut in_table = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_table = trimmed == target_header;
            continue;
        }
        if !in_table {
            continue;
        }
        let normalized = trimmed.replace(' ', "");
        if let Some(value) = normalized.strip_prefix(&key_prefix) {
            return POSITIVE_INTEGER_RE.is_match(value);
        }
    }

    false
}

fn toml_table_has_bool(content: &str, table: &str, key: &str, expected: bool) -> bool {
    let target_header = format!("[{table}]");
    let key_prefix = format!("{key}=");
    let expected_str = if expected { "true" } else { "false" };
    let mut in_table = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_table = trimmed == target_header;
            continue;
        }
        if !in_table {
            continue;
        }
        let normalized = trimmed.replace(' ', "");
        if let Some(value) = normalized.strip_prefix(&key_prefix) {
            return value == expected_str;
        }
    }

    false
}

struct MatchHit {
    line: usize,
    column: usize,
    snippet: String,
}

fn find_first(content: &str, regex: &Regex) -> Option<MatchHit> {
    let matched = regex.find(content)?;
    let index = matched.start();
    let line = content[..index].chars().filter(|ch| *ch == '\n').count() + 1;
    let last_line_start = content[..index]
        .rfind('\n')
        .map(|value| value + 1)
        .unwrap_or(0);
    let next_line_end = content[index..]
        .find('\n')
        .map(|value| index + value)
        .unwrap_or(content.len());
    Some(MatchHit {
        line,
        column: index - last_line_start + 1,
        snippet: content[last_line_start..next_line_end]
            .trim_end()
            .to_string(),
    })
}

fn line_hit(content: &str, regex: &Regex) -> FindingOptions {
    find_first(content, regex)
        .map(|hit| FindingOptions {
            line: Some(hit.line),
            column: Some(hit.column),
            snippet: Some(hit.snippet),
            ..Default::default()
        })
        .unwrap_or_default()
}

fn find_first_unfrozen_bun_install(
    content: &str,
    bun_install_re: &Regex,
    bun_frozen_re: &Regex,
) -> Option<FindingOptions> {
    for mat in bun_install_re.find_iter(content) {
        let start = mat.start();
        let block = extract_command_block(content, start);
        if !bun_frozen_re.is_match(block) {
            let line_num = content[..start].chars().filter(|&c| c == '\n').count() + 1;
            let line_start = content[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_end = content[start..]
                .find('\n')
                .map(|i| start + i)
                .unwrap_or(content.len());
            let line = &content[line_start..line_end];
            return Some(FindingOptions {
                line: Some(line_num),
                column: Some(start - line_start + 1),
                snippet: Some(line.trim_end().to_string()),
                ..Default::default()
            });
        }
    }
    None
}

fn extract_command_block(content: &str, start: usize) -> &str {
    let line_start = content[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let mut block_end = content[start..]
        .find('\n')
        .map(|i| start + i)
        .unwrap_or(content.len());
    loop {
        let trimmed = content[line_start..block_end].trim_end();
        if !trimmed.ends_with('\\') || block_end >= content.len() {
            break;
        }
        if let Some(next_nl) = content[block_end + 1..].find('\n') {
            block_end += 1 + next_nl;
        } else {
            block_end = content.len();
            break;
        }
    }
    &content[line_start..block_end]
}
