use bun_platform_core::{
    PlatformPaths, SkillContext, VERIFIED_BUN_VERSION, apply_safe_fixes,
    create_release_sync_report, load_audit_config, plan_safe_fixes, preview_release_sync,
    run_audit,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct TestEnv {
    root: PathBuf,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl TestEnv {
    fn new(label: &str) -> Self {
        let guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let root = unique_temp_dir(label);
        fs::create_dir_all(&root).expect("create temp root");
        // Serialized by test_lock so environment mutation cannot race.
        unsafe {
            env::set_var("XDG_CONFIG_HOME", root.join("config"));
            env::set_var("XDG_CACHE_HOME", root.join("cache"));
            env::set_var("XDG_STATE_HOME", root.join("state"));
        }
        Self {
            root,
            _guard: guard,
        }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    env::temp_dir().join(format!("dev-skills-{label}-{nanos}"))
}

fn copy_fixture(name: &str) -> PathBuf {
    let source = fixtures_root().join(name);
    let target = unique_temp_dir(name);
    copy_dir(&source, &target);
    target
}

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("create target");
    for entry in fs::read_dir(source).expect("read dir") {
        let entry = entry.expect("dir entry");
        let entry_path = entry.path();
        let destination = target.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_dir(&entry_path, &destination);
        } else {
            fs::copy(&entry_path, &destination).expect("copy file");
        }
    }
}

#[test]
fn reports_mixed_lockfiles_as_an_error() {
    let _env = TestEnv::new("mixed-lockfiles");
    let root = copy_fixture("mixed-lockfiles");
    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");
    let findings = run_audit(&root, &config, &paths).expect("audit");
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "pm-no-mixed-lockfiles"
                && finding.severity == bun_platform_core::Severity::Error)
    );
    assert!(!root.join(".bun-platform").exists());
}

#[test]
fn does_not_report_bunx_warning_for_non_bun_repo() {
    let _env = TestEnv::new("non-bun-npx");
    let root = unique_temp_dir("non-bun-npx-root");
    fs::create_dir_all(&root).expect("root");
    fs::write(
        root.join("package.json"),
        r#"{"private":true,"scripts":{"generate":"npx prisma generate"}}"#,
    )
    .expect("package");

    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");
    let findings = run_audit(&root, &config, &paths).expect("audit");

    assert!(
        !findings
            .iter()
            .any(|finding| finding.rule_id == "pm-bunx-vs-npx")
    );
}

#[test]
fn plans_and_applies_safe_package_json_fixes() {
    let _env = TestEnv::new("safe-fixes");
    let root = copy_fixture("safe-fixes");
    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");

    let planned = plan_safe_fixes(&root, &config).expect("plan");
    let rule_ids = planned[0].rule_ids.clone();
    assert!(rule_ids.contains(&"pm-bunx-vs-npx".to_string()));
    assert!(rule_ids.contains(&"pm-package-manager-field".to_string()));

    apply_safe_fixes(&root, &config, &paths).expect("apply");
    let package_json =
        fs::read_to_string(root.join("package.json")).expect("package.json after fixes");
    assert!(package_json.contains(&format!(
        "\"packageManager\": \"bun@{VERIFIED_BUN_VERSION}\""
    )));
    assert!(package_json.contains("\"gen\": \"bunx prisma generate\""));
    assert!(!root.join(".bun-platform").exists());
}

#[test]
fn parses_jsonc_tsconfig_without_rewriting_it() {
    let _env = TestEnv::new("jsonc-tsconfig");
    let root = copy_fixture("jsonc-tsconfig");
    let tsconfig_path = root.join("tsconfig.json");
    let before = fs::read_to_string(&tsconfig_path).expect("tsconfig before audit");
    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");

    let findings = run_audit(&root, &config, &paths).expect("audit JSONC tsconfig");

    assert!(
        findings
            .iter()
            .all(|finding| !finding.rule_id.starts_with("tsconfig-"))
    );
    assert_eq!(
        fs::read_to_string(tsconfig_path).expect("tsconfig after audit"),
        before
    );
}

#[test]
fn reports_missing_bun_types_dependency_when_types_unset() {
    let _env = TestEnv::new("missing-bun-types");
    let root = copy_fixture("jsonc-tsconfig");
    // Bun-first repo with a tsconfig but no @types/bun and `types` unset: TypeScript will
    // not know the Bun global, so the audit must still nudge. Relaxing the scoped-`types`
    // false positive must not silence this missing-dependency case.
    fs::write(
        root.join("package.json"),
        "{\n  \"name\": \"x\",\n  \"private\": true,\n  \"packageManager\": \"bun@1.3.14\"\n}\n",
    )
    .expect("write package.json");
    fs::write(
        root.join("tsconfig.json"),
        "{\n  \"compilerOptions\": {\n    \"moduleResolution\": \"Bundler\",\n    \"target\": \"ESNext\",\n    \"module\": \"Preserve\",\n    \"allowImportingTsExtensions\": true,\n    \"verbatimModuleSyntax\": true,\n    \"noEmit\": true\n  }\n}\n",
    )
    .expect("write tsconfig.json");
    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");

    let findings = run_audit(&root, &config, &paths).expect("audit");

    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "tsconfig-bun-types")
        .expect("tsconfig-bun-types finding for missing @types/bun");
    assert!(
        finding.message.contains("Install @types/bun"),
        "expected the install nudge, got: {}",
        finding.message
    );
}

#[test]
fn reports_empty_types_array_as_scoping_out_bun() {
    let _env = TestEnv::new("empty-types");
    let root = copy_fixture("jsonc-tsconfig"); // fixture ships @types/bun installed
    // `types: []` explicitly scopes ambient types to nothing, so the Bun global is
    // excluded even though @types/bun is installed; the audit should flag it.
    fs::write(
        root.join("tsconfig.json"),
        "{\n  \"compilerOptions\": {\n    \"moduleResolution\": \"Bundler\",\n    \"target\": \"ESNext\",\n    \"module\": \"Preserve\",\n    \"allowImportingTsExtensions\": true,\n    \"verbatimModuleSyntax\": true,\n    \"noEmit\": true,\n    \"types\": []\n  }\n}\n",
    )
    .expect("write tsconfig.json");
    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");

    let findings = run_audit(&root, &config, &paths).expect("audit");

    let finding = findings
        .iter()
        .find(|finding| finding.rule_id == "tsconfig-bun-types")
        .expect("tsconfig-bun-types finding for empty types array");
    assert!(
        finding.message.contains("scoped but omits Bun's types"),
        "expected the scoped-out nudge, got: {}",
        finding.message
    );
}

#[test]
fn rejects_invalid_jsonc_tsconfig_with_location() {
    let _env = TestEnv::new("invalid-jsonc-tsconfig");
    let root = copy_fixture("jsonc-tsconfig");
    let tsconfig_path = root.join("tsconfig.json");
    fs::write(
        &tsconfig_path,
        "{\n  \"compilerOptions\": {\n    target: \"ESNext\"\n  }\n}\n",
    )
    .expect("write invalid tsconfig");
    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");

    let error = run_audit(&root, &config, &paths).expect_err("invalid JSONC must fail");
    let message = format!("{error:#}");

    assert!(message.contains(&format!("failed to parse {}", tsconfig_path.display())));
    assert!(message.contains("Expected string for object property"));
    assert!(message.contains("line 3 column 5"));
}

#[test]
fn normalizes_next_scripts_when_vercel_bun_runtime_is_enabled() {
    let _env = TestEnv::new("vercel-next");
    let root = copy_fixture("vercel-next");
    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");

    let findings = run_audit(&root, &config, &paths).expect("audit");
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "vercel-nextjs-bun-runtime-scripts")
    );

    apply_safe_fixes(&root, &config, &paths).expect("apply");
    let package_json =
        fs::read_to_string(root.join("package.json")).expect("package.json after fixes");
    assert!(package_json.contains("\"dev\": \"bun run --bun next dev\""));
    assert!(package_json.contains("\"build\": \"bun run --bun next build\""));
}

#[test]
fn respects_disabled_rules_and_baseline_suppressions() {
    let _env = TestEnv::new("baseline");
    let root = copy_fixture("safe-fixes");
    fs::write(
        root.join("baseline.json"),
        "[\"pm-package-manager-field:package.json\"]",
    )
    .expect("write baseline");
    fs::write(
        root.join("bun-platform.config.json"),
        r#"{
  "disabledRules": ["pm-bunx-vs-npx"],
  "baseline": "./baseline.json"
}"#,
    )
    .expect("write config");

    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");
    let findings = run_audit(&root, &config, &paths).expect("audit");
    assert!(
        !findings
            .iter()
            .any(|finding| finding.rule_id == "pm-package-manager-field")
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.rule_id == "pm-bunx-vs-npx")
    );
    assert!(plan_safe_fixes(&root, &config).expect("plan").is_empty());
}

#[test]
fn recognizes_current_bun_lockfile_name() {
    let _env = TestEnv::new("bun-lock");
    let root = copy_fixture("mixed-lockfiles");
    // TestEnv::new isolates state; copy_fixture starts with bun.lockb, then
    // fs::remove_file/fs::write swaps in bun.lock so PlatformPaths::discover,
    // load_audit_config, and run_audit must still produce pm-no-mixed-lockfiles
    // in findings.
    fs::remove_file(root.join("bun.lockb")).expect("remove old lockfile");
    fs::write(root.join("bun.lock"), "").expect("write current lockfile");
    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");
    let findings = run_audit(&root, &config, &paths).expect("audit");

    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "pm-no-mixed-lockfiles")
    );
}

#[test]
fn run_audit_does_not_write_cache_by_default() {
    let _env = TestEnv::new("external-state");
    let root = copy_fixture("safe-fixes");
    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");

    let _ = run_audit(&root, &config, &paths).expect("audit");

    assert!(!paths.scan_cache_dir().exists());
    assert!(!root.join(".bun-platform").exists());
}

#[test]
fn writes_external_rollbacks_and_opt_in_cache() {
    let _env = TestEnv::new("external-state-cache");
    let root = copy_fixture("safe-fixes");
    let paths = PlatformPaths::discover().expect("paths");
    let overrides = bun_platform_core::CliOverrides {
        write_cache: true,
        ..Default::default()
    };
    let config = load_audit_config(&root, None, &overrides).expect("config");

    let _ = run_audit(&root, &config, &paths).expect("audit");
    apply_safe_fixes(&root, &config, &paths).expect("apply");

    let cache_entries = fs::read_dir(paths.scan_cache_dir())
        .expect("scan cache dir")
        .count();
    let rollback_entries = fs::read_dir(paths.rollback_dir())
        .expect("rollback dir")
        .count();
    assert!(cache_entries > 0);
    assert!(rollback_entries > 0);
    assert!(!root.join(".bun-platform").exists());
}

#[test]
fn reports_adapter_findings() {
    let _env = TestEnv::new("adapters");
    let paths = PlatformPaths::discover().expect("paths");

    let github_root = copy_fixture("github-actions");
    let github_config =
        load_audit_config(&github_root, None, &Default::default()).expect("github config");
    let github_findings = run_audit(&github_root, &github_config, &paths).expect("github audit");
    assert!(
        github_findings
            .iter()
            .any(|finding| finding.rule_id == "scripts-no-npm-in-bun-repos")
    );
    assert!(
        github_findings
            .iter()
            .any(|finding| finding.rule_id == "pm-bun-install-ci-frozen-lockfile")
    );

    let docker_root = copy_fixture("docker");
    let docker_config =
        load_audit_config(&docker_root, None, &Default::default()).expect("docker config");
    let docker_findings = run_audit(&docker_root, &docker_config, &paths).expect("docker audit");
    assert!(
        docker_findings
            .iter()
            .any(|finding| finding.rule_id == "runtime-bun-vs-node-choose")
    );

    let monorepo_root = copy_fixture("monorepo");
    let monorepo_config =
        load_audit_config(&monorepo_root, None, &Default::default()).expect("monorepo config");
    let monorepo_findings =
        run_audit(&monorepo_root, &monorepo_config, &paths).expect("monorepo audit");
    assert!(
        monorepo_findings
            .iter()
            .any(|finding| finding.rule_id == "scripts-bun-filter-and-workspaces")
    );
}

#[test]
fn ignores_commented_vercel_ts_bun_version() {
    let _env = TestEnv::new("commented-vercel-ts");
    let root = unique_temp_dir("commented-vercel-ts-root");
    fs::create_dir_all(&root).expect("root");
    fs::write(
        root.join("package.json"),
        r#"{"private":true,"scripts":{"dev":"next dev","build":"next build"}}"#,
    )
    .expect("package");
    fs::write(
        root.join("vercel.ts"),
        "export default {\n  // bunVersion: \"1.x\",\n};\n",
    )
    .expect("vercel.ts");

    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");
    let findings = run_audit(&root, &config, &paths).expect("audit");

    assert!(
        !findings
            .iter()
            .any(|finding| finding.rule_id == "vercel-nextjs-bun-runtime-scripts")
    );
}

#[test]
fn ignores_empty_vercel_ts_bun_version() {
    let _env = TestEnv::new("empty-vercel-ts");
    let root = unique_temp_dir("empty-vercel-ts-root");
    fs::create_dir_all(&root).expect("root");
    fs::write(
        root.join("package.json"),
        r#"{"private":true,"scripts":{"dev":"next dev","build":"next build"}}"#,
    )
    .expect("package");
    fs::write(
        root.join("vercel.ts"),
        "export default {\n  bunVersion: \"\",\n};\n",
    )
    .expect("vercel.ts");

    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");
    let findings = run_audit(&root, &config, &paths).expect("audit");

    assert!(
        !findings
            .iter()
            .any(|finding| finding.rule_id == "vercel-nextjs-bun-runtime-scripts")
    );
}

#[test]
fn detects_only_active_vercel_ts_bun_runtime_config() {
    let _env = TestEnv::new("active-vercel-ts");
    let root = unique_temp_dir("active-vercel-ts-root");
    fs::create_dir_all(&root).expect("root");
    fs::write(
        root.join("package.json"),
        r#"{"private":true,"scripts":{"dev":"next dev","build":"next build"}}"#,
    )
    .expect("package");
    fs::write(
        root.join("vercel.ts"),
        "export default {\n  bunVersion: \"\",\n  runtime: \"bun\",\n};\n",
    )
    .expect("vercel.ts");

    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");
    let findings = run_audit(&root, &config, &paths).expect("audit");

    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "vercel-nextjs-bun-runtime-scripts")
    );
}

#[test]
fn detects_version_pinned_vercel_ts_bun_runtime() {
    let _env = TestEnv::new("pinned-vercel-ts");
    let root = unique_temp_dir("pinned-vercel-ts-root");
    fs::create_dir_all(&root).expect("root");
    fs::write(
        root.join("package.json"),
        r#"{"private":true,"scripts":{"dev":"next dev","build":"next build"}}"#,
    )
    .expect("package");
    fs::write(
        root.join("vercel.ts"),
        "export default {\n  runtime: \"bun@1.3.14\",\n};\n",
    )
    .expect("vercel.ts");

    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");
    let findings = run_audit(&root, &config, &paths).expect("audit");

    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "vercel-nextjs-bun-runtime-scripts")
    );
}

#[test]
fn include_scope_excludes_github_workflow_findings() {
    let _env = TestEnv::new("include-workflow");
    let root = copy_fixture("github-actions");
    let paths = PlatformPaths::discover().expect("paths");
    let overrides = bun_platform_core::CliOverrides {
        include_paths: vec![PathBuf::from("package.json")],
        ..Default::default()
    };
    let config = load_audit_config(&root, None, &overrides).expect("config");
    let findings = run_audit(&root, &config, &paths).expect("audit");

    assert!(
        !findings
            .iter()
            .any(|finding| finding.file.contains(".github/workflows"))
    );
}

#[test]
fn multiline_frozen_bun_install_is_not_reported() {
    let _env = TestEnv::new("multiline-frozen-install");
    let root = copy_fixture("github-actions");
    fs::write(
        root.join(".github/workflows/ci.yml"),
        "name: ci\non: [push]\njobs:\n  validate:\n    runs-on: ubuntu-latest\n    steps:\n      - run: |\n          bun install \\\n            --production \\\n            --frozen-lockfile\n",
    )
    .expect("workflow");
    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");
    let findings = run_audit(&root, &config, &paths).expect("audit");

    assert!(
        !findings
            .iter()
            .any(|finding| finding.rule_id == "pm-bun-install-ci-frozen-lockfile")
    );
}

#[test]
fn cache_fingerprint_includes_direct_root_inputs() {
    let _env = TestEnv::new("cache-root-inputs");
    let root = copy_fixture("github-actions");
    let paths = PlatformPaths::discover().expect("paths");
    let overrides = bun_platform_core::CliOverrides {
        include_paths: vec![PathBuf::from("package.json")],
        write_cache: true,
        ..Default::default()
    };
    let config = load_audit_config(&root, None, &overrides).expect("config");

    let initial = run_audit(&root, &config, &paths).expect("initial audit");
    assert!(
        !initial
            .iter()
            .any(|finding| finding.file.ends_with(".nvmrc"))
    );

    fs::write(root.join(".nvmrc"), "22\n").expect("nvmrc");
    let updated = run_audit(&root, &config, &paths).expect("updated audit");

    assert!(
        updated
            .iter()
            .any(|finding| finding.file.ends_with(".nvmrc"))
    );
}

#[test]
fn cache_fingerprint_distinguishes_missing_and_empty_files() {
    let _env = TestEnv::new("cache-file-existence");
    let root = copy_fixture("github-actions");
    let paths = PlatformPaths::discover().expect("paths");
    let overrides = bun_platform_core::CliOverrides {
        include_paths: vec![PathBuf::from("package.json")],
        write_cache: true,
        ..Default::default()
    };
    let config = load_audit_config(&root, None, &overrides).expect("config");

    let initial = run_audit(&root, &config, &paths).expect("initial audit");
    assert!(
        !initial
            .iter()
            .any(|finding| finding.file.ends_with(".nvmrc"))
    );

    fs::write(root.join(".nvmrc"), "").expect("empty nvmrc");
    let updated = run_audit(&root, &config, &paths).expect("updated audit");

    assert!(
        updated
            .iter()
            .any(|finding| finding.file.ends_with(".nvmrc"))
    );
}

#[test]
fn explicit_vercel_adapter_reports_missing_bun_version() {
    let _env = TestEnv::new("explicit-vercel");
    let root = unique_temp_dir("explicit-vercel-root");
    fs::create_dir_all(&root).expect("root");
    fs::write(root.join("package.json"), r#"{"private":true}"#).expect("package");
    fs::write(root.join("vercel.json"), "{}").expect("vercel");
    fs::write(
        root.join("bun-platform.config.json"),
        r#"{"adapters":["vercel"]}"#,
    )
    .expect("config");
    let paths = PlatformPaths::discover().expect("paths");
    let config = load_audit_config(&root, None, &Default::default()).expect("config");
    let findings = run_audit(&root, &config, &paths).expect("audit");

    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "vercel-bun-runtime-enable")
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.rule_id == "vercel-bun-install-detection")
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.rule_id == "vercel-bun-runtime-limitations")
    );
}

#[test]
fn creates_release_sync_report_from_current_skill_refs() {
    let _env = TestEnv::new("release-report");
    let context =
        SkillContext::discover(Some(repo_root().join("skills/bun-dev"))).expect("skill context");
    let report = create_release_sync_report(&context).expect("report");
    assert!(!report.references.is_empty());
    assert!(
        report
            .capability_map
            .iter()
            .any(|entry| entry.topic == "bun markdown ansi")
    );
}

#[test]
fn rejects_missing_explicit_config_path() {
    let _env = TestEnv::new("missing-config");
    let root = copy_fixture("safe-fixes");
    let missing = root.join("missing-config.json");
    let error = load_audit_config(&root, Some(&missing), &Default::default()).expect_err("error");
    assert!(error.to_string().contains("config file does not exist"));
}

#[test]
fn rejects_unknown_config_keys() {
    let _env = TestEnv::new("unknown-config-key");
    let root = copy_fixture("safe-fixes");
    fs::write(
        root.join("bun-platform.config.json"),
        r#"{
  "disabledRules": [],
  "unknownKey": true
}"#,
    )
    .expect("write config");

    let error = load_audit_config(&root, None, &Default::default()).expect_err("error");
    assert!(format!("{error:#}").contains("unknown field"));
}

#[test]
fn rejects_invalid_inline_baseline_object_shape() {
    let _env = TestEnv::new("bad-baseline-object");
    let root = copy_fixture("safe-fixes");
    fs::write(
        root.join("bun-platform.config.json"),
        r#"{
  "baseline": {
    "keys": [
      "pm-package-manager-field:package.json"
    ]
  }
}"#,
    )
    .expect("write config");

    let error = load_audit_config(&root, None, &Default::default()).expect_err("error");
    assert!(error.to_string().contains("suppressionKeys"));
}

#[test]
fn rejects_invalid_baseline_file_shape() {
    let _env = TestEnv::new("bad-baseline-file");
    let root = copy_fixture("safe-fixes");
    fs::write(
        root.join("baseline.json"),
        r#"{
  "keys": [
    "pm-package-manager-field:package.json"
  ]
}"#,
    )
    .expect("write baseline");
    fs::write(
        root.join("bun-platform.config.json"),
        r#"{
  "baseline": "./baseline.json"
}"#,
    )
    .expect("write config");

    let error = load_audit_config(&root, None, &Default::default()).expect_err("error");
    assert!(error.to_string().contains("suppressionKeys"));
}

#[test]
fn config_schema_and_template_stay_aligned() {
    let _env = TestEnv::new("template-config");
    let root = copy_fixture("safe-fixes");
    let template_path =
        repo_root().join("skills/bun-dev/assets/templates/bun-platform.config.example.json");
    let template = serde_json::from_str::<serde_json::Value>(
        &fs::read_to_string(&template_path).expect("read template"),
    )
    .expect("parse template");
    let template_object = template.as_object().expect("template object");

    fs::write(
        root.join("bun-platform.config.json"),
        serde_json::to_string_pretty(&template).expect("template json"),
    )
    .expect("write template config");
    fs::write(root.join("bun-platform-baseline.json"), "[]").expect("write template baseline");
    load_audit_config(&root, None, &Default::default()).expect("template config loads");

    assert_eq!(
        template_object.get("writeCache").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        template["baseline"].as_str(),
        Some("./bun-platform-baseline.json")
    );
    assert_eq!(
        template["severityOverrides"]["pm-package-manager-field"].as_str(),
        Some("warn")
    );
}

#[test]
#[ignore = "networked release-sync preview"]
fn previews_release_sync_without_mutating_the_skill() {
    let _env = TestEnv::new("release-preview");
    let skill_root = repo_root().join("skills/bun-dev");
    if !skill_root.is_dir() {
        return;
    }

    let context = SkillContext::discover(Some(skill_root)).expect("skill context");
    let before = fs::read_to_string(context.references_dir.join("index.md")).expect("before");
    let preview = preview_release_sync(&context).expect("preview");
    let after = fs::read_to_string(context.references_dir.join("index.md")).expect("after");

    assert_eq!(before, after);
    assert!(preview.integrity_ok);
    assert!(
        preview
            .would_update
            .iter()
            .chain(preview.unchanged.iter())
            .any(|entry| entry == "references/index.md")
    );
}
