use crate::*;

#[derive(Debug, Deserialize)]
pub(crate) struct EvalSuite {
    pub(crate) suite: String,
    pub(crate) description: Option<String>,
    pub(crate) tasks: Vec<EvalTask>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EvalTask {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) description: String,
    pub(crate) input: Value,
    pub(crate) expected: Value,
}

#[derive(Default)]
pub(crate) struct EvalAssertions {
    pub(crate) failures: Vec<String>,
    pub(crate) warnings: Vec<String>,
    pub(crate) details: BTreeMap<String, Value>,
}

#[derive(Serialize)]
pub(crate) struct EvalTaskSummary {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) description: String,
}

#[derive(Serialize)]
pub(crate) struct EvalTaskOutcome {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) description: String,
    pub(crate) status: String,
    pub(crate) failures: Vec<String>,
    pub(crate) warnings: Vec<String>,
    pub(crate) details: BTreeMap<String, Value>,
}

pub(crate) async fn run_eval(
    args: EvalArgs,
    config: &ResearchConfig,
    json_out: bool,
) -> Result<()> {
    let suite = load_eval_suite(args.suite.as_deref())?;
    let selected = select_eval_tasks(&suite, &args.task)?;

    if args.list {
        let tasks = selected
            .iter()
            .map(|task| EvalTaskSummary {
                id: task.id.clone(),
                kind: task.kind.clone(),
                description: task.description.clone(),
            })
            .collect::<Vec<_>>();
        let result = json!({
            "suite": suite.suite,
            "description": suite.description,
            "tasks": tasks,
        });
        if json_out {
            print_json(&result)?;
        } else {
            for task in tasks {
                println!("{} [{}] {}", task.id, task.kind, task.description);
            }
        }
        return Ok(());
    }

    let mut outcomes = Vec::new();
    for task in selected {
        let assertions = evaluate_eval_task(task, config);
        let failed =
            !assertions.failures.is_empty() || (args.strict && !assertions.warnings.is_empty());
        outcomes.push(EvalTaskOutcome {
            id: task.id.clone(),
            kind: task.kind.clone(),
            description: task.description.clone(),
            status: if failed { "failed" } else { "passed" }.to_string(),
            failures: assertions.failures,
            warnings: assertions.warnings,
            details: assertions.details,
        });
    }

    let mut live = Vec::new();
    if args.live {
        let _client = http_client()?;
        if std::env::var_os("CONTEXT7_API_KEY").is_some() {
            live.push(json!({ "provider": "context7", "status": "configured" }));
        }
        if std::env::var_os("FIRECRAWL_API_KEY").is_some() {
            live.push(json!({ "provider": "firecrawl", "status": "configured" }));
        }
        let github = github_token().is_some();
        live.push(json!({ "provider": "github", "status": if github { "configured" } else { "public-only" } }));
    }

    let failed = outcomes.iter().any(|outcome| outcome.status == "failed");
    let passed = outcomes
        .iter()
        .filter(|outcome| outcome.status == "passed")
        .count();
    let failed_count = outcomes.len() - passed;
    let result = json!({
        "suite": suite.suite,
        "description": suite.description,
        "offline": {
            "passed": passed,
            "failed": failed_count,
            "strict": args.strict,
            "tasks": outcomes
        },
        "live": live
    });
    if json_out {
        print_json(&result)?;
        if failed {
            bail!("offline eval failures");
        }
        Ok(())
    } else {
        println!("offline: {passed}/{} tasks passed", passed + failed_count);
        if failed {
            println!(
                "{}",
                serde_json::to_string_pretty(&result["offline"]["tasks"])?
            );
            bail!("offline eval failures");
        }
        if args.live {
            println!("{}", serde_json::to_string_pretty(&result["live"])?);
        }
        Ok(())
    }
}

pub(crate) fn load_eval_suite(path: Option<&Path>) -> Result<EvalSuite> {
    let text = match path {
        Some(path) => fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?,
        None => DEFAULT_EVAL_SUITE.to_string(),
    };
    let suite: EvalSuite =
        serde_json::from_str(&text).context("failed to parse eval suite JSON")?;
    if suite.tasks.is_empty() {
        bail!("eval suite has no tasks");
    }
    let mut seen = BTreeSet::new();
    for task in &suite.tasks {
        if !seen.insert(task.id.clone()) {
            bail!("duplicate eval task id `{}`", task.id);
        }
    }
    Ok(suite)
}

pub(crate) fn select_eval_tasks<'a>(
    suite: &'a EvalSuite,
    ids: &[String],
) -> Result<Vec<&'a EvalTask>> {
    if ids.is_empty() {
        return Ok(suite.tasks.iter().collect());
    }
    let requested = ids.iter().cloned().collect::<BTreeSet<_>>();
    let selected = suite
        .tasks
        .iter()
        .filter(|task| requested.contains(&task.id))
        .collect::<Vec<_>>();
    let found = selected
        .iter()
        .map(|task| task.id.clone())
        .collect::<BTreeSet<_>>();
    let missing = requested.difference(&found).cloned().collect::<Vec<_>>();
    if !missing.is_empty() {
        bail!("unknown eval task id(s): {}", missing.join(", "));
    }
    Ok(selected)
}

pub(crate) fn evaluate_eval_task(task: &EvalTask, config: &ResearchConfig) -> EvalAssertions {
    let mut assertions = EvalAssertions::default();
    let result = match task.kind.as_str() {
        "route-classification" => evaluate_route_eval(task, &mut assertions),
        "privacy-redaction" => evaluate_privacy_eval(task, &mut assertions, config),
        "budget-plan" => evaluate_budget_eval(task, &mut assertions, config),
        "evidence-contract" => evaluate_evidence_contract_eval(task, &mut assertions),
        "evidence-bundle" => evaluate_evidence_bundle_eval(task, &mut assertions),
        "report-contract" => evaluate_report_contract_eval(task, &mut assertions),
        other => {
            assertions
                .failures
                .push(format!("unsupported eval task kind `{other}`"));
            Ok(())
        }
    };
    if let Err(error) = result {
        assertions.failures.push(error.to_string());
    }
    assertions
}

pub(crate) fn evaluate_route_eval(task: &EvalTask, assertions: &mut EvalAssertions) -> Result<()> {
    let url = required_str(&task.input, "url")?;
    let body = optional_str(&task.input, "body")?.unwrap_or("");
    let content_type = optional_str(&task.input, "content_type")?;
    let report = classify_body(url, content_type, None, body);

    assertions
        .details
        .insert("route".to_string(), json!(route_name(report.route)));
    assertions
        .details
        .insert("reason".to_string(), json!(report.reason));

    if let Some(expected_route) = optional_str(&task.expected, "route")? {
        assert_text_eq(
            assertions,
            "route",
            expected_route,
            route_name(report.route),
        );
    }
    if let Some(expected_privacy) = optional_str(&task.expected, "privacy")? {
        let privacy = classify_privacy(url);
        assertions
            .details
            .insert("privacy".to_string(), json!(privacy_class_name(privacy)));
        assert_text_eq(
            assertions,
            "privacy",
            expected_privacy,
            privacy_class_name(privacy),
        );
    }
    Ok(())
}

pub(crate) fn evaluate_privacy_eval(
    task: &EvalTask,
    assertions: &mut EvalAssertions,
    config: &ResearchConfig,
) -> Result<()> {
    let url = optional_str(&task.input, "url")?;
    let metadata_input = optional_str(&task.input, "metadata_text")?;
    if url.is_none() && metadata_input.is_none() {
        bail!("privacy-redaction requires `url` or `metadata_text` input");
    }
    if url.is_none()
        && (optional_str(&task.expected, "privacy")?.is_some()
            || optional_str(&task.expected, "redacted_url")?.is_some())
    {
        bail!("privacy-redaction expectations `privacy` and `redacted_url` require `url` input");
    }
    if metadata_input.is_none() && optional_str(&task.expected, "metadata_text")?.is_some() {
        bail!("privacy-redaction expectation `metadata_text` requires `metadata_text` input");
    }

    if let Some(url) = url {
        let privacy = classify_privacy(url);
        let redacted = redact_url_query_secrets(url);
        assertions
            .details
            .insert("privacy".to_string(), json!(privacy_class_name(privacy)));
        assertions
            .details
            .insert("redacted_url".to_string(), json!(redacted));
        if let Some(expected_privacy) = optional_str(&task.expected, "privacy")? {
            assert_text_eq(
                assertions,
                "privacy",
                expected_privacy,
                privacy_class_name(privacy),
            );
        }
        if let Some(expected_redacted) = optional_str(&task.expected, "redacted_url")? {
            let actual = assertions
                .details
                .get("redacted_url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            assert_text_eq(assertions, "redacted_url", expected_redacted, &actual);
        }
    }
    if let Some(text) = metadata_input {
        let redacted = metadata_text(text, config);
        assertions
            .details
            .insert("metadata_text".to_string(), json!(redacted));
        if let Some(expected_text) = optional_str(&task.expected, "metadata_text")? {
            let actual = assertions
                .details
                .get("metadata_text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            assert_text_eq(assertions, "metadata_text", expected_text, &actual);
        }
    }
    Ok(())
}

pub(crate) fn evaluate_budget_eval(
    task: &EvalTask,
    assertions: &mut EvalAssertions,
    config: &ResearchConfig,
) -> Result<()> {
    let query = required_str(&task.input, "query")?;
    let profile =
        parse_research_profile(optional_str(&task.input, "profile")?.unwrap_or("standard"))?;
    let topic = parse_topic_kind(optional_str(&task.input, "topic")?.unwrap_or("general"))?;
    let plan = build_plan(query, profile, topic, config);
    let route_order = plan
        .route_order
        .iter()
        .map(|route| route_name(*route))
        .collect::<Vec<_>>();
    assertions
        .details
        .insert("route_order".to_string(), json!(route_order));
    assertions
        .details
        .insert("budgets".to_string(), json!(plan.budgets));

    if let Some(prefix) = optional_str_array(&task.expected, "route_order_prefix")? {
        let actual = route_order
            .iter()
            .take(prefix.len())
            .copied()
            .collect::<Vec<_>>();
        if actual != prefix {
            assertions.failures.push(format!(
                "route_order_prefix expected {:?}, got {:?}",
                prefix, actual
            ));
        }
    }
    if let Some(expected_budgets) = optional_object(&task.expected, "budgets")? {
        for (key, expected) in expected_budgets {
            let Some(expected) = expected.as_u64() else {
                assertions
                    .failures
                    .push(format!("budgets.{key} expected value must be an integer"));
                continue;
            };
            let actual = budget_value(&plan.budgets, key);
            match actual {
                Some(actual) if u64::from(actual) == expected => {}
                Some(actual) => assertions
                    .failures
                    .push(format!("budgets.{key} expected {expected}, got {actual}")),
                None => assertions
                    .failures
                    .push(format!("unknown budget field `{key}`")),
            }
        }
    }
    Ok(())
}

pub(crate) fn evaluate_evidence_contract_eval(
    task: &EvalTask,
    assertions: &mut EvalAssertions,
) -> Result<()> {
    let sources = required_array(&task.input, "sources")?;
    let claims = required_array(&task.input, "claims")?;
    let source_ids = sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            source
                .get("id")
                .and_then(Value::as_str)
                .with_context(|| format!("sources[{index}].id must be a string"))
                .map(str::to_string)
        })
        .collect::<Result<BTreeSet<_>>>()?;
    let min_sources = optional_u64(&task.expected, "min_sources")?.unwrap_or(1);
    let min_claims = optional_u64(&task.expected, "min_claims")?.unwrap_or(1);
    let max_uncited_claims = optional_u64(&task.expected, "max_uncited_claims")?.unwrap_or(0);
    let min_confidence = optional_f64(&task.expected, "min_confidence")?;

    let mut uncited_claims = 0_u64;
    let mut missing_sources = Vec::new();
    let mut low_confidence = Vec::new();
    for claim in claims {
        let claim_id = claim
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("<unnamed claim>");
        let claim_sources = claim
            .get("sources")
            .and_then(Value::as_array)
            .with_context(|| format!("{claim_id}.sources must be an array"))?
            .iter()
            .enumerate()
            .map(|(index, value)| {
                value
                    .as_str()
                    .with_context(|| format!("{claim_id}.sources[{index}] must be a string"))
                    .map(str::to_string)
            })
            .collect::<Result<Vec<_>>>()?;
        if claim_sources.is_empty() {
            uncited_claims += 1;
        }
        for source_id in claim_sources {
            if !source_ids.contains(&source_id) {
                missing_sources.push(format!("{claim_id}->{source_id}"));
            }
        }
        if let Some(min_confidence) = min_confidence {
            let confidence = claim
                .get("confidence")
                .and_then(Value::as_f64)
                .unwrap_or_default();
            if confidence < min_confidence {
                low_confidence.push(format!("{claim_id}:{confidence:.2}"));
            }
        }
    }

    assertions
        .details
        .insert("source_count".to_string(), json!(source_ids.len()));
    assertions
        .details
        .insert("claim_count".to_string(), json!(claims.len()));
    assertions
        .details
        .insert("uncited_claims".to_string(), json!(uncited_claims));

    if source_ids.len() < usize::try_from(min_sources).unwrap_or(usize::MAX) {
        assertions.failures.push(format!(
            "source_count expected at least {min_sources}, got {}",
            source_ids.len()
        ));
    }
    if claims.len() < usize::try_from(min_claims).unwrap_or(usize::MAX) {
        assertions.failures.push(format!(
            "claim_count expected at least {min_claims}, got {}",
            claims.len()
        ));
    }
    if uncited_claims > max_uncited_claims {
        assertions.failures.push(format!(
            "uncited_claims expected at most {max_uncited_claims}, got {uncited_claims}"
        ));
    }
    if !missing_sources.is_empty() {
        assertions.failures.push(format!(
            "claims reference missing source ids: {}",
            missing_sources.join(", ")
        ));
    }
    if !low_confidence.is_empty() {
        assertions.warnings.push(format!(
            "claims below confidence threshold: {}",
            low_confidence.join(", ")
        ));
    }
    Ok(())
}

pub(crate) fn evaluate_evidence_bundle_eval(
    task: &EvalTask,
    assertions: &mut EvalAssertions,
) -> Result<()> {
    let bundle = task
        .input
        .get("bundle")
        .and_then(Value::as_object)
        .context("evidence-bundle requires object input `bundle`")?;
    let schema = bundle
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert_text_eq(assertions, "schema", EVIDENCE_BUNDLE_SCHEMA, schema);

    let source_count = bundle
        .get("ledger")
        .and_then(|ledger| ledger.get("source_count"))
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let claim_count = bundle
        .get("ledger")
        .and_then(|ledger| ledger.get("claim_count"))
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let uncited_claims = bundle
        .get("citation_coverage")
        .and_then(|coverage| coverage.get("uncited_claims"))
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let status = bundle
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let provider_error_count = bundle
        .get("provider_errors")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default() as u64;
    let failure_count = bundle
        .get("failures")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default() as u64;
    let report_exists = bundle
        .get("report")
        .and_then(|report| report.get("exists"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    assertions
        .details
        .insert("source_count".to_string(), json!(source_count));
    assertions
        .details
        .insert("claim_count".to_string(), json!(claim_count));
    assertions
        .details
        .insert("uncited_claims".to_string(), json!(uncited_claims));
    assertions
        .details
        .insert("status".to_string(), json!(status));
    assertions
        .details
        .insert("provider_errors".to_string(), json!(provider_error_count));
    assertions
        .details
        .insert("failures".to_string(), json!(failure_count));
    assertions
        .details
        .insert("report_exists".to_string(), json!(report_exists));

    let min_sources = optional_u64(&task.expected, "min_sources")?.unwrap_or(1);
    let min_claims = optional_u64(&task.expected, "min_claims")?.unwrap_or(1);
    let max_uncited_claims = optional_u64(&task.expected, "max_uncited_claims")?.unwrap_or(0);
    let max_provider_errors = optional_u64(&task.expected, "max_provider_errors")?;
    let max_failures = optional_u64(&task.expected, "max_failures")?;
    if source_count < min_sources {
        assertions.failures.push(format!(
            "bundle source_count expected at least {min_sources}, got {source_count}"
        ));
    }
    if claim_count < min_claims {
        assertions.failures.push(format!(
            "bundle claim_count expected at least {min_claims}, got {claim_count}"
        ));
    }
    if uncited_claims > max_uncited_claims {
        assertions.failures.push(format!(
            "bundle uncited_claims expected at most {max_uncited_claims}, got {uncited_claims}"
        ));
    }
    if let Some(expected_status) = optional_str(&task.expected, "status")? {
        assert_text_eq(assertions, "status", expected_status, status);
    }
    if let Some(max_provider_errors) = max_provider_errors
        && provider_error_count > max_provider_errors
    {
        assertions.failures.push(format!(
            "bundle provider_errors expected at most {max_provider_errors}, got {provider_error_count}"
        ));
    }
    if let Some(max_failures) = max_failures
        && failure_count > max_failures
    {
        assertions.failures.push(format!(
            "bundle failures expected at most {max_failures}, got {failure_count}"
        ));
    }
    if optional_bool(&task.expected, "report_exists")?.unwrap_or(false) && !report_exists {
        assertions
            .failures
            .push("bundle report.exists expected true".to_string());
    }
    let freshness_statuses = bundle
        .get("source_freshness")
        .and_then(|freshness| freshness.get("by_status"))
        .and_then(Value::as_object);
    for status in optional_str_array(&task.expected, "required_source_freshness_statuses")?
        .unwrap_or_default()
    {
        if !freshness_statuses
            .map(|statuses| statuses.contains_key(status))
            .unwrap_or(false)
        {
            assertions.failures.push(format!(
                "bundle source_freshness.by_status missing `{status}`"
            ));
        }
    }
    for key in optional_str_array(&task.expected, "required_top_level_keys")?.unwrap_or_default() {
        if !bundle.contains_key(key) {
            assertions
                .failures
                .push(format!("bundle missing top-level key `{key}`"));
        }
    }
    Ok(())
}

pub(crate) fn evaluate_report_contract_eval(
    task: &EvalTask,
    assertions: &mut EvalAssertions,
) -> Result<()> {
    let report = required_str(&task.input, "report")?;
    for section in optional_str_array(&task.expected, "required_sections")?.unwrap_or_default() {
        if !report_has_heading(report, section) {
            assertions
                .failures
                .push(format!("report missing required section `{section}`"));
        }
    }
    for phrase in optional_str_array(&task.expected, "forbidden_phrases")?.unwrap_or_default() {
        if report.contains(phrase) {
            assertions
                .failures
                .push(format!("report contains forbidden phrase `{phrase}`"));
        }
    }
    for source_id in
        optional_str_array(&task.expected, "required_source_mentions")?.unwrap_or_default()
    {
        if !report.contains(source_id) {
            assertions
                .failures
                .push(format!("report missing source mention `{source_id}`"));
        }
    }
    assertions
        .details
        .insert("chars".to_string(), json!(report.chars().count()));
    Ok(())
}

pub(crate) fn report_has_heading(report: &str, section: &str) -> bool {
    report.lines().any(|line| {
        let trimmed = line.trim();
        let Some(title) = trimmed.strip_prefix("## ") else {
            return false;
        };
        title.trim() == section
    })
}

pub(crate) fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    optional_str(value, key)?.with_context(|| format!("missing string input `{key}`"))
}

pub(crate) fn optional_str<'a>(value: &'a Value, key: &str) -> Result<Option<&'a str>> {
    let Some(value) = value.get(key) else {
        return Ok(None);
    };
    value
        .as_str()
        .map(Some)
        .with_context(|| format!("`{key}` must be a string"))
}

pub(crate) fn required_array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .with_context(|| format!("missing array input `{key}`"))
}

pub(crate) fn optional_object<'a>(
    value: &'a Value,
    key: &str,
) -> Result<Option<&'a serde_json::Map<String, Value>>> {
    let Some(value) = value.get(key) else {
        return Ok(None);
    };
    value
        .as_object()
        .map(Some)
        .with_context(|| format!("`{key}` must be an object"))
}

pub(crate) fn optional_u64(value: &Value, key: &str) -> Result<Option<u64>> {
    let Some(value) = value.get(key) else {
        return Ok(None);
    };
    value
        .as_u64()
        .map(Some)
        .with_context(|| format!("`{key}` must be an unsigned integer"))
}

pub(crate) fn optional_f64(value: &Value, key: &str) -> Result<Option<f64>> {
    let Some(value) = value.get(key) else {
        return Ok(None);
    };
    value
        .as_f64()
        .map(Some)
        .with_context(|| format!("`{key}` must be a number"))
}

pub(crate) fn optional_bool(value: &Value, key: &str) -> Result<Option<bool>> {
    let Some(value) = value.get(key) else {
        return Ok(None);
    };
    value
        .as_bool()
        .map(Some)
        .with_context(|| format!("`{key}` must be a boolean"))
}

pub(crate) fn optional_str_array<'a>(value: &'a Value, key: &str) -> Result<Option<Vec<&'a str>>> {
    let Some(values) = value.get(key) else {
        return Ok(None);
    };
    let Some(values) = values.as_array() else {
        bail!("`{key}` must be an array of strings");
    };
    let mut out = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let Some(text) = value.as_str() else {
            bail!("`{key}[{index}]` must be a string");
        };
        out.push(text);
    }
    Ok(Some(out))
}

pub(crate) fn assert_text_eq(
    assertions: &mut EvalAssertions,
    name: &str,
    expected: &str,
    actual: &str,
) {
    if expected != actual {
        assertions
            .failures
            .push(format!("{name} expected `{expected}`, got `{actual}`"));
    }
}

pub(crate) fn parse_research_profile(value: &str) -> Result<ResearchProfile> {
    match value {
        "quick" => Ok(ResearchProfile::Quick),
        "standard" => Ok(ResearchProfile::Standard),
        "deep" => Ok(ResearchProfile::Deep),
        "exhaustive" => Ok(ResearchProfile::Exhaustive),
        _ => bail!("unknown research profile `{value}`"),
    }
}

pub(crate) fn parse_topic_kind(value: &str) -> Result<TopicKind> {
    match value {
        "general" => Ok(TopicKind::General),
        "docs" => Ok(TopicKind::Docs),
        "github" => Ok(TopicKind::Github),
        "dependency" => Ok(TopicKind::Dependency),
        "openai" => Ok(TopicKind::Openai),
        "rendered" => Ok(TopicKind::Rendered),
        _ => bail!("unknown topic `{value}`"),
    }
}

pub(crate) fn budget_value(budgets: &ProviderBudgets, key: &str) -> Option<u32> {
    match key {
        "codex_web_queries" => Some(budgets.codex_web_queries),
        "context7_calls" => Some(budgets.context7_calls),
        "github_calls" => Some(budgets.github_calls),
        "exa_calls" => Some(budgets.exa_calls),
        "direct_fetches" => Some(budgets.direct_fetches),
        "browser_fetches" => Some(budgets.browser_fetches),
        "firecrawl_calls" => Some(budgets.firecrawl_calls),
        _ => None,
    }
}
