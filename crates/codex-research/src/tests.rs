use super::*;

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "codex-research-{name}-{}",
        short_hash(format!("{}-{}", std::process::id(), Utc::now()))
    ))
}

#[test]
fn dependency_plan_prefers_docs_source_and_github_routes() {
    let plan = build_plan(
        "verify dependency behavior",
        ResearchProfile::Deep,
        TopicKind::Dependency,
        &ResearchConfig::default(),
    );

    assert_eq!(plan.profile.to_string(), "deep");
    assert_eq!(plan.budgets.context7_calls, 4);
    assert_eq!(plan.budgets.github_calls, 8);
    assert_eq!(plan.route_order[0], Route::Context7);
    assert_eq!(plan.route_order[1], Route::Opensrc);
    assert!(plan.route_order.contains(&Route::Github));
}

fn evaluate_test_task(task: &EvalTask) -> EvalAssertions {
    evaluate_eval_task(task, &ResearchConfig::default())
}

#[test]
fn default_eval_suite_is_manifest_backed_and_passes_offline() -> Result<()> {
    let suite = load_eval_suite(None)?;
    assert_eq!(suite.suite, "research-core");
    assert!(suite.tasks.len() >= 5);
    let config = ResearchConfig::default();

    for task in select_eval_tasks(&suite, &[])? {
        let outcome = evaluate_eval_task(task, &config);
        assert!(
            outcome.failures.is_empty(),
            "{} failed: {:?}",
            task.id,
            outcome.failures
        );
    }
    Ok(())
}

#[test]
fn eval_task_filter_reports_unknown_ids() -> Result<()> {
    let suite = load_eval_suite(None)?;
    let result = select_eval_tasks(&suite, &["missing-task".to_string()]);

    assert!(result.is_err());
    Ok(())
}

#[test]
fn privacy_eval_rejects_empty_tasks() {
    let task = EvalTask {
        id: "empty-privacy".to_string(),
        kind: "privacy-redaction".to_string(),
        description: "empty".to_string(),
        input: json!({}),
        expected: json!({}),
    };
    let outcome = evaluate_test_task(&task);

    assert!(outcome.failures[0].contains("requires `url` or `metadata_text`"));
}

#[test]
fn report_contract_requires_explicit_headings() {
    let task = EvalTask {
        id: "missing-heading".to_string(),
        kind: "report-contract".to_string(),
        description: "heading check".to_string(),
        input: json!({
            "report": "- Provider limits are mentioned in prose only."
        }),
        expected: json!({
            "required_sections": ["Provider limits"]
        }),
    };
    let outcome = evaluate_test_task(&task);

    assert!(
        outcome
            .failures
            .iter()
            .any(|failure| failure.contains("missing required section"))
    );
}

#[test]
fn eval_rejects_malformed_expected_string_arrays() {
    let task = EvalTask {
        id: "bad-array".to_string(),
        kind: "report-contract".to_string(),
        description: "array validation".to_string(),
        input: json!({
            "report": "## Claims\n- cited"
        }),
        expected: json!({
            "required_sections": ["Claims", 42]
        }),
    };
    let outcome = evaluate_test_task(&task);

    assert!(
        outcome
            .failures
            .iter()
            .any(|failure| failure.contains("required_sections[1]"))
    );
}

#[test]
fn eval_rejects_malformed_confidence_thresholds() {
    let task = EvalTask {
        id: "bad-confidence".to_string(),
        kind: "evidence-contract".to_string(),
        description: "threshold validation".to_string(),
        input: json!({
            "sources": [{"id": "source-1"}],
            "claims": [{"id": "claim-1", "sources": ["source-1"], "confidence": 0.9}]
        }),
        expected: json!({
            "min_confidence": "high"
        }),
    };
    let outcome = evaluate_test_task(&task);

    assert!(
        outcome
            .failures
            .iter()
            .any(|failure| failure.contains("min_confidence"))
    );
}

#[test]
fn evidence_contract_rejects_malformed_source_entries() {
    let task = EvalTask {
        id: "bad-source".to_string(),
        kind: "evidence-contract".to_string(),
        description: "source validation".to_string(),
        input: json!({
            "sources": [{"id": 42}],
            "claims": [{"id": "claim-1", "sources": ["source-1"]}]
        }),
        expected: json!({}),
    };
    let outcome = evaluate_test_task(&task);

    assert!(
        outcome
            .failures
            .iter()
            .any(|failure| failure.contains("sources[0].id"))
    );
}

#[test]
fn evidence_contract_rejects_malformed_claim_citations() {
    let task = EvalTask {
        id: "bad-citation".to_string(),
        kind: "evidence-contract".to_string(),
        description: "citation validation".to_string(),
        input: json!({
            "sources": [{"id": "source-1"}],
            "claims": [{"id": "claim-1", "sources": ["source-1", 42]}]
        }),
        expected: json!({}),
    };
    let outcome = evaluate_test_task(&task);

    assert!(
        outcome
            .failures
            .iter()
            .any(|failure| failure.contains("claim-1.sources[1]"))
    );
}

#[test]
fn evidence_bundle_eval_requires_closeout_shape() {
    let task = EvalTask {
        id: "bundle-shape".to_string(),
        kind: "evidence-bundle".to_string(),
        description: "bundle validation".to_string(),
        input: json!({
            "bundle": {
                "schema": EVIDENCE_BUNDLE_SCHEMA,
                "generated_at": "2026-05-11T12:00:00Z",
                "status": "passed",
                "strict": true,
                "run": {},
                "budget": {},
                "provider_errors": [],
                "ledger": {"source_count": 1, "claim_count": 1},
                "citation_coverage": {"uncited_claims": 0},
                "source_freshness": {"by_status": {"current": 1}},
                "report": {"exists": true},
                "artifacts": [],
                "warnings": [],
                "failures": []
            }
        }),
        expected: json!({
            "status": "passed",
            "min_sources": 1,
            "min_claims": 1,
            "max_uncited_claims": 0,
            "max_provider_errors": 0,
            "max_failures": 0,
            "report_exists": true,
            "required_source_freshness_statuses": ["current"],
            "required_top_level_keys": [
                "schema",
                "generated_at",
                "status",
                "strict",
                "run",
                "budget",
                "provider_errors",
                "ledger",
                "citation_coverage",
                "source_freshness",
                "report",
                "artifacts",
                "warnings",
                "failures"
            ]
        }),
    };
    let outcome = evaluate_test_task(&task);

    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

#[test]
fn evidence_bundle_closeout_reports_budget_citations_and_artifacts() -> Result<()> {
    let dir = temp_path("evidence-bundle-pass");
    fs::create_dir_all(&dir)?;
    let run_path = dir.join("run.json");
    let ledger_path = dir.join("ledger.jsonl");
    let report_path = dir.join("report.md");
    let bundle_path = dir.join("bundle.json");
    let markdown_path = dir.join("bundle.md");
    let generated_at: DateTime<Utc> = "2026-05-11T12:00:00Z".parse()?;
    let source_id = format!("src-{}", short_hash(dir.display().to_string()));
    let claim_id = format!("claim-{}", short_hash(format!("{source_id}-claim")));
    let mut budgets = standard_budget();
    budgets.github_calls = 4;
    let spent = ProviderBudgets {
        github_calls: 1,
        ..ProviderBudgets::default()
    };
    let state = ResearchRunState {
        query: "verify source-backed claim".to_string(),
        profile: ResearchProfile::Standard,
        topic: TopicKind::Github,
        status: RunStatus::Closed,
        created_at: generated_at,
        updated_at: generated_at,
        budgets,
        spent,
        debits: vec![RunDebit {
            provider: ProviderKind::Github,
            count: 1,
            note: Some("github search".to_string()),
            created_at: generated_at,
        }],
        provider_errors: Vec::new(),
        source_ids: vec![source_id.clone()],
    };
    write_run_state(&run_path, &state)?;
    append_ledger_record(
        &ledger_path,
        &LedgerRecord::Source(SourceRecord {
            id: source_id.clone(),
            provider: "github".to_string(),
            url: "https://github.com/openai/codex".to_string(),
            title: Some("openai/codex".to_string()),
            route: Some("github".to_string()),
            fetched_at: generated_at,
        }),
    )?;
    append_ledger_record(
        &ledger_path,
        &LedgerRecord::Claim(ClaimRecord {
            id: claim_id,
            text: "Repository evidence is source backed.".to_string(),
            confidence: 0.9,
            sources: vec![source_id],
            note: None,
            created_at: generated_at,
        }),
    )?;
    fs::write(&report_path, "# Research Report\n")?;

    let args = BundleArgs {
        run: run_path,
        ledger: ledger_path,
        report: report_path,
        out: Some(bundle_path),
        markdown_out: Some(markdown_path),
        generated_at: Some(generated_at),
        strict: false,
    };
    let (bundle, markdown) = build_evidence_bundle(&args, generated_at)?;
    fs::remove_dir_all(&dir)?;

    assert_eq!(bundle.schema, EVIDENCE_BUNDLE_SCHEMA);
    assert_eq!(bundle.status, "passed");
    assert_eq!(bundle.ledger.source_count, 1);
    assert_eq!(bundle.citation_coverage.uncited_claims, 0);
    assert_eq!(bundle.budget.by_provider[2].provider, "github");
    assert_eq!(bundle.budget.by_provider[2].spent, 1);
    assert!(markdown.contains("# Research Evidence Bundle"));
    Ok(())
}

#[test]
fn evidence_bundle_strict_fails_uncited_claims_and_provider_errors() -> Result<()> {
    let dir = temp_path("evidence-bundle-fail");
    fs::create_dir_all(&dir)?;
    let run_path = dir.join("run.json");
    let ledger_path = dir.join("ledger.jsonl");
    let generated_at: DateTime<Utc> = "2026-05-11T12:00:00Z".parse()?;
    let state = ResearchRunState {
        query: "verify failed closeout".to_string(),
        profile: ResearchProfile::Standard,
        topic: TopicKind::General,
        status: RunStatus::Open,
        created_at: generated_at,
        updated_at: generated_at,
        budgets: standard_budget(),
        spent: ProviderBudgets::default(),
        debits: Vec::new(),
        provider_errors: vec![ProviderError {
            provider: ProviderKind::Github,
            message: "hydration failed".to_string(),
            created_at: generated_at,
        }],
        source_ids: Vec::new(),
    };
    write_run_state(&run_path, &state)?;
    append_ledger_record(
        &ledger_path,
        &LedgerRecord::Claim(ClaimRecord {
            id: "claim-uncited".to_string(),
            text: "This claim has no sources.".to_string(),
            confidence: 0.8,
            sources: Vec::new(),
            note: None,
            created_at: generated_at,
        }),
    )?;
    let args = BundleArgs {
        run: run_path,
        ledger: ledger_path,
        report: dir.join("missing-report.md"),
        out: None,
        markdown_out: None,
        generated_at: Some(generated_at),
        strict: true,
    };
    let (bundle, _) = build_evidence_bundle(&args, generated_at)?;
    fs::remove_dir_all(&dir)?;

    assert_eq!(bundle.status, "failed");
    assert!(
        bundle
            .failures
            .iter()
            .any(|failure| failure.contains("uncited"))
    );
    assert!(
        bundle
            .failures
            .iter()
            .any(|failure| failure.contains("provider error"))
    );
    assert!(
        bundle
            .failures
            .iter()
            .any(|failure| failure.contains("report path"))
    );
    Ok(())
}

#[test]
fn evidence_bundle_strict_fails_missing_ledger_evidence() -> Result<()> {
    let dir = temp_path("evidence-bundle-missing-ledger");
    fs::create_dir_all(&dir)?;
    let run_path = dir.join("run.json");
    let generated_at: DateTime<Utc> = "2026-05-11T12:00:00Z".parse()?;
    let state = ResearchRunState {
        query: "verify missing ledger".to_string(),
        profile: ResearchProfile::Standard,
        topic: TopicKind::General,
        status: RunStatus::Closed,
        created_at: generated_at,
        updated_at: generated_at,
        budgets: standard_budget(),
        spent: ProviderBudgets::default(),
        debits: Vec::new(),
        provider_errors: Vec::new(),
        source_ids: Vec::new(),
    };
    write_run_state(&run_path, &state)?;
    let args = BundleArgs {
        run: run_path,
        ledger: dir.join("missing-ledger.jsonl"),
        report: dir.join("missing-report.md"),
        out: None,
        markdown_out: None,
        generated_at: Some(generated_at),
        strict: true,
    };
    let (bundle, _) = build_evidence_bundle(&args, generated_at)?;
    fs::remove_dir_all(&dir)?;

    assert_eq!(bundle.status, "failed");
    assert!(
        bundle
            .failures
            .iter()
            .any(|failure| failure.contains("ledger path"))
    );
    assert!(
        bundle
            .failures
            .iter()
            .any(|failure| failure.contains("no claims"))
    );
    assert!(
        bundle
            .failures
            .iter()
            .any(|failure| failure.contains("no sources"))
    );
    Ok(())
}

#[test]
fn evidence_bundle_strict_fails_unknown_source_freshness() -> Result<()> {
    let dir = temp_path("evidence-bundle-unknown-freshness");
    fs::create_dir_all(&dir)?;
    let run_path = dir.join("run.json");
    let ledger_path = dir.join("ledger.jsonl");
    let report_path = dir.join("report.md");
    let generated_at: DateTime<Utc> = "2026-05-11T12:00:00Z".parse()?;
    let source_id = format!("src-{}", short_hash(format!("{}-fresh", dir.display())));
    let state = ResearchRunState {
        query: "verify freshness evidence".to_string(),
        profile: ResearchProfile::Standard,
        topic: TopicKind::General,
        status: RunStatus::Closed,
        created_at: generated_at,
        updated_at: generated_at,
        budgets: standard_budget(),
        spent: ProviderBudgets::default(),
        debits: Vec::new(),
        provider_errors: Vec::new(),
        source_ids: vec![source_id.clone()],
    };
    write_run_state(&run_path, &state)?;
    append_ledger_record(
        &ledger_path,
        &LedgerRecord::Source(SourceRecord {
            id: source_id.clone(),
            provider: "direct".to_string(),
            url: "https://example.com/doc".to_string(),
            title: Some("example docs".to_string()),
            route: Some("direct".to_string()),
            fetched_at: generated_at,
        }),
    )?;
    append_ledger_record(
        &ledger_path,
        &LedgerRecord::Claim(ClaimRecord {
            id: "claim-cited".to_string(),
            text: "The claim is cited.".to_string(),
            confidence: 0.9,
            sources: vec![source_id],
            note: None,
            created_at: generated_at,
        }),
    )?;
    fs::write(&report_path, "# Research Report\n")?;
    let args = BundleArgs {
        run: run_path,
        ledger: ledger_path,
        report: report_path,
        out: None,
        markdown_out: None,
        generated_at: Some(generated_at),
        strict: true,
    };
    let (bundle, _) = build_evidence_bundle(&args, generated_at)?;
    fs::remove_dir_all(&dir)?;

    assert_eq!(bundle.status, "failed");
    assert!(
        bundle
            .failures
            .iter()
            .any(|failure| failure.contains("cache freshness record"))
    );
    Ok(())
}

#[test]
fn evidence_bundle_command_non_strict_reports_failures_without_error() -> Result<()> {
    let dir = temp_path("evidence-bundle-nonstrict");
    fs::create_dir_all(&dir)?;
    let run_path = dir.join("run.json");
    let ledger_path = dir.join("ledger.jsonl");
    let markdown_path = dir.join("bundle.md");
    let generated_at: DateTime<Utc> = "2026-05-11T12:00:00Z".parse()?;
    let state = ResearchRunState {
        query: "verify non-strict exit".to_string(),
        profile: ResearchProfile::Standard,
        topic: TopicKind::General,
        status: RunStatus::Closed,
        created_at: generated_at,
        updated_at: generated_at,
        budgets: standard_budget(),
        spent: ProviderBudgets::default(),
        debits: Vec::new(),
        provider_errors: Vec::new(),
        source_ids: Vec::new(),
    };
    write_run_state(&run_path, &state)?;
    append_ledger_record(
        &ledger_path,
        &LedgerRecord::Claim(ClaimRecord {
            id: "claim-uncited".to_string(),
            text: "This claim has no sources.".to_string(),
            confidence: 0.8,
            sources: Vec::new(),
            note: None,
            created_at: generated_at,
        }),
    )?;
    let args = BundleArgs {
        run: run_path,
        ledger: ledger_path,
        report: dir.join("missing-report.md"),
        out: None,
        markdown_out: Some(markdown_path),
        generated_at: Some(generated_at),
        strict: false,
    };
    let (bundle, _) = build_evidence_bundle(&args, generated_at)?;
    let result = build_evidence_bundle_command(args, false);
    fs::remove_dir_all(&dir)?;

    assert_eq!(bundle.status, "failed");
    assert!(
        bundle
            .failures
            .iter()
            .any(|failure| failure.contains("report path"))
    );
    assert!(result.is_ok(), "{result:?}");
    Ok(())
}

#[test]
fn evidence_bundle_redacts_shareable_freeform_fields() -> Result<()> {
    let dir = temp_path("evidence-bundle-redacts");
    fs::create_dir_all(&dir)?;
    let run_path = dir.join("run.json");
    let ledger_path = dir.join("ledger.jsonl");
    let report_path = dir.join("report.md");
    let generated_at: DateTime<Utc> = "2026-05-11T12:00:00Z".parse()?;
    let source_id = format!("src-{}", short_hash(dir.display().to_string()));
    let state = ResearchRunState {
            query: "check (https://example.com/doc?token=punctuatedsecret), api_key = spacedsecret bearer ghp_FAKEstandaloneToken1234567890".to_string(),
            profile: ResearchProfile::Standard,
            topic: TopicKind::General,
            status: RunStatus::Open,
            created_at: generated_at,
            updated_at: generated_at,
            budgets: standard_budget(),
            spent: ProviderBudgets::default(),
            debits: vec![RunDebit {
                provider: ProviderKind::Context7,
                count: 1,
                note: Some("lookup api_key = spacedsecret Authorization: ghp_FAKEstandaloneToken1234567890".to_string()),
                created_at: generated_at,
            }],
            provider_errors: vec![ProviderError {
                provider: ProviderKind::Context7,
                message: "Context7 returned status=500; fetch failed: <https://example.com/doc?api_key=punctuatedsecret>; Authorization: Bearer ghp_FAKEstandaloneToken1234567890 body=raw-provider-body token=supersecret"
                    .to_string(),
                created_at: generated_at,
            }],
            source_ids: vec![source_id.clone()],
        };
    write_run_state(&run_path, &state)?;
    append_ledger_record(
        &ledger_path,
        &LedgerRecord::Source(SourceRecord {
            id: source_id.clone(),
            provider: "direct".to_string(),
            url: "https://example.com/doc".to_string(),
            title: Some("example docs".to_string()),
            route: Some("direct".to_string()),
            fetched_at: generated_at,
        }),
    )?;
    append_ledger_record(
        &ledger_path,
        &LedgerRecord::Claim(ClaimRecord {
            id: "claim-cited".to_string(),
            text: "The claim is cited.".to_string(),
            confidence: 0.9,
            sources: vec![source_id],
            note: None,
            created_at: generated_at,
        }),
    )?;
    fs::write(&report_path, "# Research Report\n")?;
    let args = BundleArgs {
        run: run_path,
        ledger: ledger_path,
        report: report_path,
        out: None,
        markdown_out: None,
        generated_at: Some(generated_at),
        strict: false,
    };
    let (bundle, markdown) = build_evidence_bundle(&args, generated_at)?;
    let json = serde_json::to_string(&bundle)?;
    fs::remove_dir_all(&dir)?;

    for leaked in [
        "supersecret",
        "punctuatedsecret",
        "spacedsecret",
        "raw-provider-body",
        "ghp_FAKEstandaloneToken1234567890",
    ] {
        assert!(!json.contains(leaked), "JSON leaked {leaked}: {json}");
        assert!(
            !markdown.contains(leaked),
            "Markdown leaked {leaked}: {markdown}"
        );
    }
    assert!(json.contains("[redacted]"));
    assert!(markdown.contains("[redacted]"));
    Ok(())
}

#[test]
fn eval_rejects_malformed_scalar_expectations() {
    let task = EvalTask {
        id: "bad-route".to_string(),
        kind: "route-classification".to_string(),
        description: "scalar validation".to_string(),
        input: json!({
            "url": "https://github.com/example/repo/blob/main/README.md",
            "body": "# README"
        }),
        expected: json!({
            "route": ["github"]
        }),
    };
    let outcome = evaluate_test_task(&task);

    assert!(
        outcome
            .failures
            .iter()
            .any(|failure| failure.contains("route"))
    );
}

#[test]
fn budget_eval_rejects_malformed_budget_expectations() {
    let task = EvalTask {
        id: "bad-budgets".to_string(),
        kind: "budget-plan".to_string(),
        description: "budget validation".to_string(),
        input: json!({
            "query": "verify dependency behavior",
            "profile": "deep",
            "topic": "dependency"
        }),
        expected: json!({
            "budgets": []
        }),
    };
    let outcome = evaluate_test_task(&task);

    assert!(
        outcome
            .failures
            .iter()
            .any(|failure| failure.contains("budgets"))
    );
}

#[test]
fn budget_eval_uses_loaded_config() {
    let mut config = ResearchConfig::default();
    config.profiles.deep.github_calls = 13;
    let task = EvalTask {
        id: "configured-budget".to_string(),
        kind: "budget-plan".to_string(),
        description: "configured budget validation".to_string(),
        input: json!({
            "query": "verify dependency behavior",
            "profile": "deep",
            "topic": "dependency"
        }),
        expected: json!({
            "budgets": {
                "github_calls": 13
            }
        }),
    };
    let outcome = evaluate_eval_task(&task, &config);

    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

#[test]
fn default_config_serializes_and_parses() -> Result<()> {
    let text = default_config_toml()?;
    let parsed: ResearchConfig = toml::from_str(&text)?;

    assert_eq!(parsed.profiles.deep.github_calls, 8);
    assert_eq!(parsed.privacy.private_external_default, "deny");
    assert_eq!(parsed.providers.firecrawl.default_max_age_ms, 172_800_000);
    Ok(())
}

#[test]
fn run_budget_debit_exhausts_provider() -> Result<()> {
    let dir = temp_path("run-budget");
    fs::create_dir_all(&dir)?;
    let run = dir.join("run.json");
    let state = ResearchRunState {
        query: "smoke".to_string(),
        profile: ResearchProfile::Quick,
        topic: TopicKind::Github,
        status: RunStatus::Open,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        budgets: quick_budget(),
        spent: ProviderBudgets::default(),
        debits: Vec::new(),
        provider_errors: Vec::new(),
        source_ids: Vec::new(),
    };
    write_run_state(&run, &state)?;

    let state = debit_run_budget(&run, ProviderKind::Github, 1, Some("test"))?;
    assert_eq!(provider_remaining(&state, ProviderKind::Github), 0);
    assert!(debit_run_budget(&run, ProviderKind::Github, 1, Some("test")).is_err());
    fs::remove_dir_all(&dir)?;
    Ok(())
}

#[test]
fn source_attachment_updates_active_run_state() -> Result<()> {
    let dir = temp_path("run-source");
    fs::create_dir_all(&dir)?;
    let run = dir.join("run.json");
    let state = ResearchRunState {
        query: "smoke".to_string(),
        profile: ResearchProfile::Quick,
        topic: TopicKind::Github,
        status: RunStatus::Open,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        budgets: quick_budget(),
        spent: ProviderBudgets::default(),
        debits: Vec::new(),
        provider_errors: Vec::new(),
        source_ids: Vec::new(),
    };
    write_run_state(&run, &state)?;
    let budget = BudgetArgs {
        run: Some(run.clone()),
        no_budget: true,
    };

    attach_source_to_run(&budget, "src123")?;
    attach_source_to_run(&budget, "src123")?;

    let state = read_run_state(&run)?;
    assert_eq!(state.source_ids, vec!["src123"]);
    fs::remove_dir_all(&dir)?;
    Ok(())
}

#[test]
fn provider_budget_errors_are_recorded_in_run_state() -> Result<()> {
    let dir = temp_path("run-provider-errors");
    fs::create_dir_all(&dir)?;
    let run = dir.join("run.json");
    let state = ResearchRunState {
        query: "smoke".to_string(),
        profile: ResearchProfile::Quick,
        topic: TopicKind::Github,
        status: RunStatus::Open,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        budgets: quick_budget(),
        spent: ProviderBudgets {
            github_calls: 1,
            ..ProviderBudgets::default()
        },
        debits: Vec::new(),
        provider_errors: Vec::new(),
        source_ids: Vec::new(),
    };
    write_run_state(&run, &state)?;
    let budget = BudgetArgs {
        run: Some(run.clone()),
        no_budget: false,
    };

    assert!(maybe_debit(&budget, ProviderKind::Github, 1, Some("test")).is_err());

    let state = read_run_state(&run)?;
    assert_eq!(state.provider_errors.len(), 1);
    assert_eq!(state.provider_errors[0].provider, ProviderKind::Github);
    fs::remove_dir_all(&dir)?;
    Ok(())
}

#[test]
fn run_init_refuses_to_overwrite_existing_state() -> Result<()> {
    let dir = temp_path("run-init-existing");
    fs::create_dir_all(&dir)?;
    let run = dir.join("run.json");
    fs::write(&run, b"{\"existing\":true}")?;

    let result = handle_run(
        RunCommand::Init {
            query: "smoke".to_string(),
            profile: ResearchProfile::Quick,
            topic: TopicKind::General,
            out: run.clone(),
        },
        &ResearchConfig::default(),
        false,
    );

    assert!(result.is_err());
    assert_eq!(fs::read_to_string(&run)?, "{\"existing\":true}");
    fs::remove_dir_all(&dir)?;
    Ok(())
}

#[test]
fn close_run_state_preserves_existing_run_history() -> Result<()> {
    let dir = temp_path("run-close");
    fs::create_dir_all(&dir)?;
    let run = dir.join("run.json");
    let state = ResearchRunState {
        query: "smoke".to_string(),
        profile: ResearchProfile::Quick,
        topic: TopicKind::Github,
        status: RunStatus::Open,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        budgets: quick_budget(),
        spent: ProviderBudgets::default(),
        debits: vec![RunDebit {
            provider: ProviderKind::Github,
            count: 1,
            note: Some("test".to_string()),
            created_at: Utc::now(),
        }],
        provider_errors: vec![ProviderError {
            provider: ProviderKind::Github,
            message: "rate limited".to_string(),
            created_at: Utc::now(),
        }],
        source_ids: vec!["src123".to_string()],
    };
    write_run_state(&run, &state)?;

    let state = close_run_state(&run)?;

    assert_eq!(state.status, RunStatus::Closed);
    assert_eq!(state.debits.len(), 1);
    assert_eq!(state.provider_errors.len(), 1);
    assert_eq!(state.source_ids, vec!["src123"]);
    fs::remove_dir_all(&dir)?;
    Ok(())
}

#[test]
fn provider_call_counts_reflect_hydration_requests() {
    assert_eq!(github_issue_call_count(false), 1);
    assert_eq!(github_issue_call_count(true), 2);
    assert_eq!(github_pr_call_count(false, false, false), 1);
    assert_eq!(github_pr_call_count(true, true, true), 5);
}

#[test]
fn github_pagination_metadata_marks_truncated_link_headers() {
    let mut headers = HeaderMap::new();
    headers.insert(
            LINK,
            HeaderValue::from_static(
                "<https://api.github.com/repositories/1/pulls/2/files?page=2>; rel=\"next\", <https://api.github.com/repositories/1/pulls/2/files?page=4>; rel=\"last\"",
            ),
        );

    let metadata = github_pagination_metadata(&headers);

    assert_eq!(metadata["truncated"], true);
    assert!(metadata["link"].as_str().unwrap().contains("rel=\"next\""));
}

#[test]
fn firecrawl_cache_default_uses_config_until_flag_overrides() {
    let mut config = ResearchConfig::default();

    assert!(effective_firecrawl_store_in_cache(false, &config));
    assert!(!effective_firecrawl_store_in_cache(true, &config));

    config.providers.firecrawl.store_in_cache_default = false;
    assert!(!effective_firecrawl_store_in_cache(false, &config));
    assert!(!effective_firecrawl_store_in_cache(true, &config));
}

#[test]
fn privacy_classifier_blocks_private_external_targets() {
    assert_eq!(
        classify_privacy("https://example.com/docs"),
        PrivacyClass::Public
    );
    assert_eq!(
        classify_privacy("http://localhost:3000/dashboard"),
        PrivacyClass::PrivateOrAuthenticated
    );
    assert_eq!(
        classify_privacy("https://example.com/file?token=secret"),
        PrivacyClass::PrivateOrAuthenticated
    );
    assert_eq!(
        classify_privacy("https://github/org/repo"),
        PrivacyClass::PrivateOrAuthenticated
    );
    assert_eq!(
        classify_privacy("https://user:token@example.com/docs"),
        PrivacyClass::PrivateOrAuthenticated
    );
    assert_eq!(
        classify_privacy("https://10.0.0.12/internal"),
        PrivacyClass::PrivateOrAuthenticated
    );
    assert_eq!(
        classify_privacy("https://[fd00::1]/internal"),
        PrivacyClass::PrivateOrAuthenticated
    );
    assert_eq!(
        classify_privacy("https://[::ffff:127.0.0.1]/internal"),
        PrivacyClass::PrivateOrAuthenticated
    );
    assert!(
        enforce_external_privacy(
            PrivacyClass::PrivateOrAuthenticated,
            false,
            &ResearchConfig::default(),
            "firecrawl"
        )
        .is_err()
    );
}

#[test]
fn cache_init_normalizes_legacy_unverified_privacy_classification() -> Result<()> {
    let dir = temp_path("cache-privacy-normalize");
    let paths = ResearchPaths {
        cache_dir: dir.clone(),
        database: dir.join("research.sqlite"),
        blobs_dir: dir.join("blobs"),
    };
    fs::create_dir_all(&paths.cache_dir)?;
    let conn = Connection::open(&paths.database)?;
    conn.execute_batch(
        "
        create table sources (
            id text primary key,
            url text not null,
            provider text not null,
            fetched_at text not null,
            content_hash text,
            status integer,
            route text,
            metadata_json text,
            canonical_url text,
            title text,
            freshness_status text not null default 'unverified',
            privacy_classification text not null default 'unverified',
            raw_body_stored integer not null default 0
        );
        insert into sources (
            id, url, provider, fetched_at, metadata_json, privacy_classification
        ) values (
            'legacy-source',
            'https://example.com/doc',
            'direct',
            '2026-05-11T00:00:00Z',
            '{}',
            'unverified'
        );
        ",
    )?;
    drop(conn);

    init_db(&paths)?;
    let record = cached_source(&paths, "legacy-source")?.expect("source should exist");
    fs::remove_dir_all(paths.cache_dir)?;

    assert_eq!(record.privacy_classification, "ambiguous");
    Ok(())
}

#[test]
fn route_memory_records_and_lists_domain_preferences() -> Result<()> {
    let dir = temp_path("route-memory");
    let paths = ResearchPaths {
        database: dir.join("research.sqlite"),
        blobs_dir: dir.join("blobs"),
        cache_dir: dir,
    };
    init_db(&paths)?;
    record_route_memory(
        &paths,
        "https://docs.example.com/app",
        Route::AgentBrowser,
        true,
        Some(200),
        "rendered route worked",
    )?;
    record_route_memory(
        &paths,
        "https://docs.example.com/app",
        Route::Firecrawl,
        false,
        Some(500),
        "crawl route failed",
    )?;

    let hit = route_memory_for_url(&paths, "https://docs.example.com/other")?
        .expect("route memory should exist");
    assert_eq!(hit.domain, "docs.example.com");
    assert_eq!(hit.preferred_route, "agent-browser");
    assert_eq!(hit.successes, 1);
    assert_eq!(hit.failures, 1);
    fs::remove_dir_all(paths.cache_dir)?;
    Ok(())
}

#[test]
fn metadata_redaction_uses_privacy_config() {
    let config = ResearchConfig::default();
    assert_eq!(
        metadata_text("find api_key=secret usage", &config),
        "[redacted]"
    );

    let config = ResearchConfig {
        privacy: PrivacyConfig {
            redact_query_secrets: false,
            ..PrivacyConfig::default()
        },
        ..ResearchConfig::default()
    };
    assert_eq!(
        metadata_text("find api_key=secret usage", &config),
        "find api_key=secret usage"
    );
}

#[test]
fn source_cache_redacts_secret_query_params() -> Result<()> {
    let dir = temp_path("cache-redaction");
    let paths = ResearchPaths {
        cache_dir: dir.clone(),
        database: dir.join("research.sqlite"),
        blobs_dir: dir.join("blobs"),
    };
    init_db(&paths)?;

    let source_id = record_source_cache(
        &paths,
        SourceCacheInsert {
            url: "https://example.com/doc?token=secret&page=2",
            provider: "direct",
            status: Some(200),
            content_hash: None,
            route: Some("direct"),
            title: None,
            canonical_url: Some("https://example.com/doc?api_key=secret"),
            freshness_status: "current",
            privacy_classification: "private-or-authenticated",
            raw_body_stored: false,
            metadata: json!({ "next_url": "https://example.com/next?password=secret" }),
            redact_query_secrets: true,
        },
    )?;
    let record = cached_source(&paths, &source_id)?.expect("source should exist");

    assert_eq!(
        record.url,
        "https://example.com/doc?token=%5Bredacted%5D&page=2"
    );
    assert_eq!(
        record.canonical_url.as_deref(),
        Some("https://example.com/doc?api_key=%5Bredacted%5D")
    );
    assert_eq!(
        record.metadata["next_url"],
        "https://example.com/next?password=%5Bredacted%5D"
    );
    fs::remove_dir_all(paths.cache_dir)?;
    Ok(())
}

#[test]
fn github_urls_route_to_github_api_hydration() {
    let report = classify_body(
        "https://github.com/openai/codex/blob/main/README.md",
        Some("text/html"),
        None,
        "<html>large rendered github page</html>",
    );

    assert_eq!(report.route, Route::Github);
    assert!(report.reason.contains("GitHub APIs"));
}

#[test]
fn app_shell_with_low_text_routes_to_browser() {
    let report = classify_body(
        "https://docs.example.com/app",
        Some("text/html"),
        None,
        r#"<html><body><div id="__next"></div><script src="/app.js"></script></body></html>"#,
    );

    assert_eq!(report.route, Route::AgentBrowser);
    assert!(
        report
            .app_shell_markers
            .iter()
            .any(|marker| marker == "id=\"__next\"")
    );
}

#[test]
fn text_like_content_routes_to_direct_fetch() {
    let report = classify_body(
        "https://example.com/llms.txt",
        Some("text/plain"),
        Some(256),
        "plain text docs",
    );

    assert_eq!(report.route, Route::Direct);
}

#[test]
fn ledger_records_round_trip() -> Result<()> {
    let dir = std::env::temp_dir().join(format!(
        "codex-research-ledger-test-{}",
        short_hash(format!("{}-{}", std::process::id(), Utc::now()))
    ));
    fs::create_dir_all(&dir)?;
    let ledger = dir.join("ledger.jsonl");

    let source = LedgerRecord::Source(SourceRecord {
        id: "src123".to_string(),
        provider: "github".to_string(),
        url: "https://github.com/openai/codex".to_string(),
        title: Some("openai/codex".to_string()),
        route: Some("github".to_string()),
        fetched_at: Utc::now(),
    });
    let claim = LedgerRecord::Claim(ClaimRecord {
        id: "claim123".to_string(),
        text: "GitHub source hydration works.".to_string(),
        confidence: 0.9,
        sources: vec!["src123".to_string()],
        note: None,
        created_at: Utc::now(),
    });

    append_ledger_record(&ledger, &source)?;
    append_ledger_record(&ledger, &claim)?;
    let records = read_ledger_records(&ledger)?;
    fs::remove_dir_all(&dir)?;

    assert_eq!(records.len(), 2);
    assert!(matches!(records[0], LedgerRecord::Source(_)));
    assert!(matches!(records[1], LedgerRecord::Claim(_)));
    Ok(())
}
