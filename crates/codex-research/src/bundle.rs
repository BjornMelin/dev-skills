use crate::*;

#[derive(Debug, Serialize)]
pub(crate) struct EvidenceBundle {
    pub(crate) schema: &'static str,
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) status: String,
    pub(crate) strict: bool,
    pub(crate) run: EvidenceBundleRun,
    pub(crate) budget: EvidenceBundleBudget,
    pub(crate) provider_errors: Vec<EvidenceBundleProviderError>,
    pub(crate) ledger: EvidenceBundleLedger,
    pub(crate) citation_coverage: CitationCoverage,
    pub(crate) source_freshness: SourceFreshnessSummary,
    pub(crate) report: EvidenceBundleReport,
    pub(crate) artifacts: Vec<String>,
    pub(crate) warnings: Vec<String>,
    pub(crate) failures: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct EvidenceBundleRun {
    pub(crate) path: String,
    pub(crate) query: String,
    pub(crate) profile: ResearchProfile,
    pub(crate) topic: TopicKind,
    pub(crate) status: RunStatus,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) cache_source_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct EvidenceBundleBudget {
    pub(crate) total: ProviderBudgets,
    pub(crate) spent: ProviderBudgets,
    pub(crate) remaining: ProviderBudgets,
    pub(crate) debits: Vec<EvidenceBundleDebit>,
    pub(crate) by_provider: Vec<ProviderBudgetLine>,
}

#[derive(Debug, Serialize)]
pub(crate) struct EvidenceBundleDebit {
    pub(crate) provider: ProviderKind,
    pub(crate) count: u32,
    pub(crate) note: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub(crate) struct EvidenceBundleProviderError {
    pub(crate) provider: ProviderKind,
    pub(crate) message: String,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProviderBudgetLine {
    pub(crate) provider: &'static str,
    pub(crate) budget: u32,
    pub(crate) spent: u32,
    pub(crate) remaining: u32,
}

#[derive(Debug, Serialize)]
pub(crate) struct EvidenceBundleLedger {
    pub(crate) path: String,
    pub(crate) source_count: usize,
    pub(crate) claim_count: usize,
    pub(crate) source_ids: Vec<String>,
    pub(crate) claim_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CitationCoverage {
    pub(crate) cited_claims: usize,
    pub(crate) uncited_claims: usize,
    pub(crate) uncited_claim_ids: Vec<String>,
    pub(crate) missing_source_refs: Vec<String>,
    pub(crate) coverage: f64,
}

#[derive(Debug, Serialize)]
pub(crate) struct SourceFreshnessSummary {
    pub(crate) by_status: BTreeMap<String, usize>,
    pub(crate) unknown_source_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct EvidenceBundleReport {
    pub(crate) path: String,
    pub(crate) exists: bool,
}

pub(crate) fn build_evidence_bundle_command(args: BundleArgs, json_out: bool) -> Result<()> {
    let generated_at = args.generated_at.unwrap_or_else(Utc::now);
    let (bundle, markdown) = build_evidence_bundle(&args, generated_at)?;
    if let Some(out) = &args.out {
        ensure_parent(out)?;
        fs::write(out, serde_json::to_vec_pretty(&bundle)?)?;
    }
    if let Some(markdown_out) = &args.markdown_out {
        ensure_parent(markdown_out)?;
        fs::write(markdown_out, markdown.as_bytes())?;
    }
    let failed = args.strict && !bundle.failures.is_empty();
    if json_out {
        print_json(&bundle)?;
    } else if args.markdown_out.is_some() {
        println!(
            "bundle: {}",
            args.markdown_out
                .as_ref()
                .expect("markdown path checked")
                .display()
        );
    } else {
        print!("{markdown}");
    }
    if failed {
        bail!("evidence bundle failed closeout checks");
    }
    Ok(())
}

pub(crate) fn build_evidence_bundle(
    args: &BundleArgs,
    generated_at: DateTime<Utc>,
) -> Result<(EvidenceBundle, String)> {
    let run = read_run_state(&args.run)?;
    let ledger_exists = args.ledger.exists();
    let records = read_ledger_records(&args.ledger)?;
    let mut sources = Vec::new();
    let mut claims = Vec::new();
    for record in records {
        match record {
            LedgerRecord::Source(source) => sources.push(source),
            LedgerRecord::Claim(claim) => claims.push(claim),
        }
    }

    let ledger_source_ids = sources
        .iter()
        .map(|source| source.id.clone())
        .collect::<Vec<_>>();
    let ledger_claim_ids = claims
        .iter()
        .map(|claim| claim.id.clone())
        .collect::<Vec<_>>();
    let source_id_set = ledger_source_ids.iter().cloned().collect::<BTreeSet<_>>();
    let citation_coverage = citation_coverage(&claims, &source_id_set);
    let source_freshness = source_freshness_summary(&sources)?;
    let remaining = remaining_budgets(&run);
    let mut warnings = Vec::new();
    let mut failures = Vec::new();

    if !ledger_exists {
        failures.push(format!(
            "ledger path does not exist: {}",
            args.ledger.display()
        ));
    }
    if claims.is_empty() {
        failures.push("ledger has no claims".to_string());
    }
    if sources.is_empty() {
        failures.push("ledger has no sources".to_string());
    }
    if !citation_coverage.uncited_claim_ids.is_empty() {
        failures.push(format!(
            "uncited claim(s): {}",
            citation_coverage.uncited_claim_ids.join(", ")
        ));
    }
    if !citation_coverage.missing_source_refs.is_empty() {
        failures.push(format!(
            "claim source reference(s) missing from ledger: {}",
            citation_coverage.missing_source_refs.join(", ")
        ));
    }
    if !run.provider_errors.is_empty() {
        failures.push(format!(
            "{} unresolved provider error(s)",
            run.provider_errors.len()
        ));
    }
    if !args.report.exists() {
        failures.push(format!(
            "report path does not exist: {}",
            args.report.display()
        ));
    }
    for source_id in &run.source_ids {
        if !source_id_set.contains(source_id) {
            warnings.push(format!(
                "run source id `{source_id}` is not represented in the ledger"
            ));
        }
    }
    if !source_freshness.unknown_source_ids.is_empty() {
        let message = format!(
            "{} source(s) have no cache freshness record",
            source_freshness.unknown_source_ids.len()
        );
        if args.strict {
            failures.push(message);
        } else {
            warnings.push(message);
        }
    }

    let mut artifacts = Vec::new();
    if let Some(out) = &args.out {
        artifacts.push(out.display().to_string());
    }
    if let Some(markdown_out) = &args.markdown_out {
        artifacts.push(markdown_out.display().to_string());
    }
    if args.report.exists() {
        artifacts.push(args.report.display().to_string());
    }

    let status = if failures.is_empty() {
        "passed"
    } else {
        "failed"
    }
    .to_string();
    let bundle = EvidenceBundle {
        schema: EVIDENCE_BUNDLE_SCHEMA,
        generated_at,
        status,
        strict: args.strict,
        run: EvidenceBundleRun {
            path: args.run.display().to_string(),
            query: bundle_safe_text(&run.query),
            profile: run.profile,
            topic: run.topic,
            status: run.status.clone(),
            created_at: run.created_at,
            updated_at: run.updated_at,
            cache_source_ids: run.source_ids.clone(),
        },
        budget: EvidenceBundleBudget {
            total: run.budgets.clone(),
            spent: run.spent.clone(),
            remaining: remaining.clone(),
            debits: run.debits.iter().map(bundle_safe_debit).collect::<Vec<_>>(),
            by_provider: provider_budget_lines(&run.budgets, &run.spent, &remaining),
        },
        provider_errors: run
            .provider_errors
            .iter()
            .map(bundle_safe_provider_error)
            .collect::<Vec<_>>(),
        ledger: EvidenceBundleLedger {
            path: args.ledger.display().to_string(),
            source_count: ledger_source_ids.len(),
            claim_count: ledger_claim_ids.len(),
            source_ids: ledger_source_ids,
            claim_ids: ledger_claim_ids,
        },
        citation_coverage,
        source_freshness,
        report: EvidenceBundleReport {
            path: args.report.display().to_string(),
            exists: args.report.exists(),
        },
        artifacts,
        warnings,
        failures,
    };
    let markdown = render_evidence_bundle_markdown(&bundle);
    Ok((bundle, markdown))
}

pub(crate) fn citation_coverage(
    claims: &[ClaimRecord],
    source_ids: &BTreeSet<String>,
) -> CitationCoverage {
    let mut cited_claims = 0;
    let mut uncited_claim_ids = Vec::new();
    let mut missing_source_refs = Vec::new();
    for claim in claims {
        if claim.sources.is_empty() {
            uncited_claim_ids.push(claim.id.clone());
        } else {
            cited_claims += 1;
        }
        for source_id in &claim.sources {
            if !source_ids.contains(source_id) {
                missing_source_refs.push(format!("{}->{source_id}", claim.id));
            }
        }
    }
    let coverage = if claims.is_empty() {
        0.0
    } else {
        cited_claims as f64 / claims.len() as f64
    };
    CitationCoverage {
        cited_claims,
        uncited_claims: uncited_claim_ids.len(),
        uncited_claim_ids,
        missing_source_refs,
        coverage,
    }
}

pub(crate) fn source_freshness_summary(sources: &[SourceRecord]) -> Result<SourceFreshnessSummary> {
    let paths = research_paths()?;
    let mut by_status = BTreeMap::new();
    let mut unknown_source_ids = Vec::new();
    let conn = if paths.database.exists() {
        Connection::open_with_flags(&paths.database, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()
    } else {
        None
    };
    for source in sources {
        if let Some(cached) = conn
            .as_ref()
            .and_then(|conn| cached_source_readonly(conn, &source.id).ok().flatten())
        {
            *by_status.entry(cached.freshness_status).or_insert(0) += 1;
        } else {
            *by_status.entry("unknown".to_string()).or_insert(0) += 1;
            unknown_source_ids.push(source.id.clone());
        }
    }
    Ok(SourceFreshnessSummary {
        by_status,
        unknown_source_ids,
    })
}

pub(crate) fn cached_source_readonly(
    conn: &Connection,
    source_id: &str,
) -> rusqlite::Result<Option<SourceCacheRecord>> {
    let mut stmt = conn.prepare(
        "select id, provider, route, url, canonical_url, title, fetched_at,
                freshness_status, privacy_classification, status, content_hash,
                raw_body_stored, metadata_json
         from sources where id = ?1",
    )?;
    let mut rows = stmt.query(params![source_id])?;
    if let Some(row) = rows.next()? {
        return Ok(Some(source_from_row(row)?));
    }
    Ok(None)
}

pub(crate) fn provider_budget_lines(
    total: &ProviderBudgets,
    spent: &ProviderBudgets,
    remaining: &ProviderBudgets,
) -> Vec<ProviderBudgetLine> {
    [
        (
            ProviderKind::CodexWeb,
            total.codex_web_queries,
            spent.codex_web_queries,
            remaining.codex_web_queries,
        ),
        (
            ProviderKind::Context7,
            total.context7_calls,
            spent.context7_calls,
            remaining.context7_calls,
        ),
        (
            ProviderKind::Github,
            total.github_calls,
            spent.github_calls,
            remaining.github_calls,
        ),
        (
            ProviderKind::Exa,
            total.exa_calls,
            spent.exa_calls,
            remaining.exa_calls,
        ),
        (
            ProviderKind::Direct,
            total.direct_fetches,
            spent.direct_fetches,
            remaining.direct_fetches,
        ),
        (
            ProviderKind::Browser,
            total.browser_fetches,
            spent.browser_fetches,
            remaining.browser_fetches,
        ),
        (
            ProviderKind::Firecrawl,
            total.firecrawl_calls,
            spent.firecrawl_calls,
            remaining.firecrawl_calls,
        ),
    ]
    .into_iter()
    .map(|(provider, budget, spent, remaining)| ProviderBudgetLine {
        provider: provider_name(provider),
        budget,
        spent,
        remaining,
    })
    .collect()
}

pub(crate) fn bundle_safe_debit(debit: &RunDebit) -> EvidenceBundleDebit {
    EvidenceBundleDebit {
        provider: debit.provider,
        count: debit.count,
        note: debit.note.as_deref().map(bundle_safe_text),
        created_at: debit.created_at,
    }
}

pub(crate) fn bundle_safe_provider_error(error: &ProviderError) -> EvidenceBundleProviderError {
    EvidenceBundleProviderError {
        provider: error.provider,
        message: bundle_safe_text(&error.message),
        created_at: error.created_at,
    }
}

pub(crate) fn bundle_safe_text(value: &str) -> String {
    redact_standalone_secret_tokens(&redact_secret_assignments(&redact_embedded_urls(
        &redact_provider_body(value),
    )))
}

pub(crate) fn redact_provider_body(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    let Some(index) = lower.find("body=") else {
        return value.to_string();
    };
    let mut redacted = value[..index].to_string();
    redacted.push_str("body=[redacted]");
    redacted
}

pub(crate) fn redact_embedded_urls(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(index) = find_url_start(rest) {
        output.push_str(&rest[..index]);
        let url_and_after = &rest[index..];
        let token_end = url_and_after
            .char_indices()
            .find(|(_, ch)| ch.is_whitespace())
            .map(|(index, _)| index)
            .unwrap_or(url_and_after.len());
        let (url_token, after_token) = url_and_after.split_at(token_end);
        let trailing_len = url_token
            .chars()
            .rev()
            .take_while(|ch| secret_token_boundary_punctuation(*ch))
            .map(char::len_utf8)
            .sum::<usize>();
        let core_end = url_token.len().saturating_sub(trailing_len);
        let (url_core, trailing) = url_token.split_at(core_end);
        output.push_str(&redact_url_query_secrets(url_core));
        output.push_str(trailing);
        rest = after_token;
    }
    output.push_str(rest);
    output
}

pub(crate) fn find_url_start(value: &str) -> Option<usize> {
    match (value.find("http://"), value.find("https://")) {
        (Some(http), Some(https)) => Some(http.min(https)),
        (Some(http), None) => Some(http),
        (None, Some(https)) => Some(https),
        (None, None) => None,
    }
}

pub(crate) fn redact_secret_assignments(value: &str) -> String {
    value
        .split_inclusive(char::is_whitespace)
        .map(|part| {
            let split_at = part
                .char_indices()
                .find(|(_, ch)| ch.is_whitespace())
                .map(|(index, _)| index)
                .unwrap_or(part.len());
            let (token, suffix) = part.split_at(split_at);
            format!("{}{}", redact_secret_assignment_token(token), suffix)
        })
        .collect()
}

pub(crate) fn redact_secret_assignment_token(token: &str) -> String {
    for separator in ['=', ':'] {
        if let Some(index) = token.find(separator) {
            let key = token[..index].trim_matches(|ch: char| {
                matches!(ch, '"' | '\'' | '`' | '{' | '[' | '(' | ',' | ';')
            });
            if secret_query_key(key) || key.eq_ignore_ascii_case("body") {
                let prefix = &token[..=index];
                let trailing = token[index + 1..]
                    .chars()
                    .rev()
                    .take_while(|ch| matches!(ch, '"' | '\'' | '`' | '}' | ']' | ')' | ',' | ';'))
                    .count();
                let suffix_start = token.len()
                    - token[index + 1..]
                        .chars()
                        .rev()
                        .take(trailing)
                        .map(char::len_utf8)
                        .sum::<usize>();
                return format!("{}[redacted]{}", prefix, &token[suffix_start..]);
            }
        }
    }
    token.to_string()
}

pub(crate) fn redact_standalone_secret_tokens(value: &str) -> String {
    let mut redaction_state = BundleRedactionState::None;
    let mut output = String::with_capacity(value.len());
    for part in value.split_inclusive(char::is_whitespace) {
        let split_at = part
            .char_indices()
            .find(|(_, ch)| ch.is_whitespace())
            .map(|(index, _)| index)
            .unwrap_or(part.len());
        let (token, suffix) = part.split_at(split_at);
        let (leading, core, trailing) = split_secret_token_parts(token);
        if matches!(redaction_state, BundleRedactionState::AwaitAssignmentValue) {
            if is_assignment_separator(core) {
                output.push_str(token);
                output.push_str(suffix);
                redaction_state = BundleRedactionState::RedactNext;
                continue;
            }
            redaction_state = BundleRedactionState::None;
        }
        if matches!(redaction_state, BundleRedactionState::RedactNext) {
            if is_bearer_marker(core) || is_assignment_separator(core) {
                output.push_str(token);
                output.push_str(suffix);
                redaction_state = BundleRedactionState::RedactNext;
                continue;
            }
            if !core.is_empty() {
                output.push_str(leading);
                output.push_str("[redacted]");
                output.push_str(trailing);
                output.push_str(suffix);
                redaction_state = BundleRedactionState::None;
                continue;
            }
        }
        if is_secret_context_marker(core) {
            output.push_str(token);
            output.push_str(suffix);
            redaction_state = if secret_context_needs_separator(core) {
                BundleRedactionState::AwaitAssignmentValue
            } else {
                BundleRedactionState::RedactNext
            };
            continue;
        }
        if secret_token_like(core) {
            output.push_str(leading);
            output.push_str("[redacted]");
            output.push_str(trailing);
        } else {
            output.push_str(token);
        }
        output.push_str(suffix);
        redaction_state = BundleRedactionState::None;
    }
    output
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BundleRedactionState {
    None,
    AwaitAssignmentValue,
    RedactNext,
}

pub(crate) fn split_secret_token_parts(token: &str) -> (&str, &str, &str) {
    let leading_end = token
        .char_indices()
        .find(|(_, ch)| !secret_token_boundary_punctuation(*ch))
        .map(|(index, _)| index)
        .unwrap_or(token.len());
    let trailing_len = token[leading_end..]
        .chars()
        .rev()
        .take_while(|ch| secret_token_boundary_punctuation(*ch))
        .map(char::len_utf8)
        .sum::<usize>();
    let core_end = token.len().saturating_sub(trailing_len);
    (
        &token[..leading_end],
        &token[leading_end..core_end],
        &token[core_end..],
    )
}

pub(crate) fn secret_token_boundary_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '"' | '\'' | '`' | '{' | '[' | '(' | '<' | '}' | ']' | ')' | '>' | ',' | ';' | '.'
    )
}

pub(crate) fn is_secret_context_marker(value: &str) -> bool {
    let marker = value.trim_end_matches(':').to_ascii_lowercase();
    matches!(
        marker.as_str(),
        "authorization"
            | "bearer"
            | "token"
            | "access_token"
            | "auth"
            | "api_key"
            | "apikey"
            | "secret"
            | "password"
    )
}

pub(crate) fn secret_context_needs_separator(value: &str) -> bool {
    let marker = value.trim_end_matches(':');
    !(value.ends_with(':')
        || marker.eq_ignore_ascii_case("authorization")
        || marker.eq_ignore_ascii_case("bearer"))
}

pub(crate) fn is_bearer_marker(value: &str) -> bool {
    value.trim_end_matches(':').eq_ignore_ascii_case("bearer")
}

pub(crate) fn is_assignment_separator(value: &str) -> bool {
    matches!(value, "=" | ":" | "=>")
}

pub(crate) fn secret_token_like(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let known_prefix = [
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "ghr_",
        "github_pat_",
        "sk-",
        "sk_live_",
        "sk_test_",
        "xoxb-",
        "xoxp-",
        "xoxa-",
        "xoxr-",
        "pk_live_",
        "pk_test_",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix));
    if known_prefix && value.len() >= 8 {
        return true;
    }
    if value.len() == 20
        && (value.starts_with("AKIA") || value.starts_with("ASIA"))
        && value
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
    {
        return true;
    }
    value.split('.').count() == 3 && value.starts_with("eyJ")
}

pub(crate) fn render_evidence_bundle_markdown(bundle: &EvidenceBundle) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Research Evidence Bundle\n\n");
    markdown.push_str(&format!("- Status: {}\n", bundle.status));
    markdown.push_str(&format!("- Generated: {}\n", bundle.generated_at));
    markdown.push_str(&format!("- Query: {}\n", bundle.run.query));
    markdown.push_str(&format!("- Run status: {:?}\n", bundle.run.status));
    markdown.push_str(&format!(
        "- Claims: {} total, {} cited, {} uncited\n",
        bundle.ledger.claim_count,
        bundle.citation_coverage.cited_claims,
        bundle.citation_coverage.uncited_claims
    ));
    markdown.push_str(&format!("- Sources: {}\n", bundle.ledger.source_count));
    markdown.push_str(&format!(
        "- Provider errors: {}\n",
        bundle.provider_errors.len()
    ));
    markdown.push_str(&format!(
        "- Report: {} ({})\n",
        bundle.report.path,
        if bundle.report.exists {
            "exists"
        } else {
            "missing"
        }
    ));
    markdown.push_str("\n## Provider Budget\n\n");
    markdown.push_str("| Provider | Budget | Spent | Remaining |\n");
    markdown.push_str("| --- | ---: | ---: | ---: |\n");
    for line in &bundle.budget.by_provider {
        markdown.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            line.provider, line.budget, line.spent, line.remaining
        ));
    }
    markdown.push_str("\n## Source Freshness\n\n");
    if bundle.source_freshness.by_status.is_empty() {
        markdown.push_str("- No sources recorded.\n");
    } else {
        for (status, count) in &bundle.source_freshness.by_status {
            markdown.push_str(&format!("- {status}: {count}\n"));
        }
    }
    markdown.push_str("\n## Failures\n\n");
    if bundle.failures.is_empty() {
        markdown.push_str("- None.\n");
    } else {
        for failure in &bundle.failures {
            markdown.push_str(&format!("- {failure}\n"));
        }
    }
    markdown.push_str("\n## Warnings\n\n");
    if bundle.warnings.is_empty() {
        markdown.push_str("- None.\n");
    } else {
        for warning in &bundle.warnings {
            markdown.push_str(&format!("- {warning}\n"));
        }
    }
    markdown
}
