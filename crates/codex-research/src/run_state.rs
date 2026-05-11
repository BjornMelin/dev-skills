use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ResearchRunState {
    pub(crate) query: String,
    pub(crate) profile: ResearchProfile,
    pub(crate) topic: TopicKind,
    pub(crate) status: RunStatus,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) budgets: ProviderBudgets,
    pub(crate) spent: ProviderBudgets,
    pub(crate) debits: Vec<RunDebit>,
    pub(crate) provider_errors: Vec<ProviderError>,
    pub(crate) source_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RunStatus {
    Open,
    Closed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RunDebit {
    pub(crate) provider: ProviderKind,
    pub(crate) count: u32,
    pub(crate) note: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ProviderError {
    pub(crate) provider: ProviderKind,
    pub(crate) message: String,
    pub(crate) created_at: DateTime<Utc>,
}

pub(crate) fn github_issue_call_count(comments: bool) -> u32 {
    1 + u32::from(comments)
}

pub(crate) fn github_pr_call_count(files: bool, comments: bool, reviews: bool) -> u32 {
    1 + u32::from(files) + if comments { 2 } else { 0 } + u32::from(reviews)
}

pub(crate) fn read_run_state(path: &Path) -> Result<ResearchRunState> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read run state: {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("failed to parse run state: {}", path.display()))
}

#[cfg(test)]
pub(crate) fn write_run_state(path: &Path, state: &ResearchRunState) -> Result<()> {
    let _lock = acquire_run_lock(path)?;
    write_run_state_unlocked(path, state)
}

pub(crate) fn write_run_state_unlocked(path: &Path, state: &ResearchRunState) -> Result<()> {
    ensure_parent(path)?;
    let temp_path = path.with_file_name(format!(
        ".{}.tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("run.json"),
        short_hash(Utc::now().to_rfc3339())
    ));
    fs::write(&temp_path, serde_json::to_vec_pretty(state)?)?;
    fs::rename(&temp_path, path)?;
    Ok(())
}

pub(crate) struct RunLock {
    pub(crate) path: PathBuf,
}

impl Drop for RunLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub(crate) fn run_lock_path(path: &Path) -> PathBuf {
    path.with_file_name(format!(
        "{}.lock",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("run.json")
    ))
}

pub(crate) fn acquire_run_lock(path: &Path) -> Result<RunLock> {
    ensure_parent(path)?;
    let lock_path = run_lock_path(path);
    for _ in 0..100 {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                writeln!(file, "pid={} created_at={}", std::process::id(), Utc::now())?;
                return Ok(RunLock { path: lock_path });
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(err) => {
                return Err(err).with_context(|| {
                    format!("failed to create run lock: {}", lock_path.display())
                });
            }
        }
    }
    bail!("timed out waiting for run lock: {}", lock_path.display())
}

pub(crate) fn maybe_debit(
    budget: &BudgetArgs,
    provider: ProviderKind,
    count: u32,
    note: Option<&str>,
) -> Result<()> {
    if let Some(path) = &budget.run
        && !budget.no_budget
    {
        let result = debit_run_budget(path, provider, count, note);
        if let Err(error) = &result {
            let _ = append_provider_error(path, provider, &error.to_string());
        }
        result?;
    }
    Ok(())
}

pub(crate) async fn track_provider_result<T, F>(
    budget: &BudgetArgs,
    provider: ProviderKind,
    future: F,
) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let result = future.await;
    if let Err(error) = &result {
        let _ = append_provider_error_from_budget(budget, provider, &error.to_string());
    }
    result
}

pub(crate) fn debit_run_budget(
    path: &Path,
    provider: ProviderKind,
    count: u32,
    note: Option<&str>,
) -> Result<ResearchRunState> {
    let _lock = acquire_run_lock(path)?;
    let mut state = read_run_state(path)?;
    if state.status == RunStatus::Closed {
        bail!("research run is closed: {}", path.display());
    }
    let remaining = provider_remaining(&state, provider);
    if remaining < count {
        bail!(
            "budget exhausted for {}; remaining={} requested={}",
            provider_name(provider),
            remaining,
            count
        );
    }
    *budget_slot_mut(&mut state.spent, provider) += count;
    state.debits.push(RunDebit {
        provider,
        count,
        note: note.map(str::to_string),
        created_at: Utc::now(),
    });
    state.updated_at = Utc::now();
    write_run_state_unlocked(path, &state)?;
    Ok(state)
}

pub(crate) fn attach_source_to_run(budget: &BudgetArgs, source_id: &str) -> Result<()> {
    let Some(path) = &budget.run else {
        return Ok(());
    };
    let _lock = acquire_run_lock(path)?;
    let mut state = read_run_state(path)?;
    if state.status == RunStatus::Closed {
        bail!("research run is closed: {}", path.display());
    }
    if !state.source_ids.iter().any(|id| id == source_id) {
        state.source_ids.push(source_id.to_string());
        state.updated_at = Utc::now();
        write_run_state_unlocked(path, &state)?;
    }
    Ok(())
}

pub(crate) fn close_run_state(path: &Path) -> Result<ResearchRunState> {
    let _lock = acquire_run_lock(path)?;
    let mut state = read_run_state(path)?;
    state.status = RunStatus::Closed;
    state.updated_at = Utc::now();
    write_run_state_unlocked(path, &state)?;
    Ok(state)
}

pub(crate) fn append_provider_error_from_budget(
    budget: &BudgetArgs,
    provider: ProviderKind,
    message: &str,
) -> Result<()> {
    if let Some(path) = &budget.run
        && !budget.no_budget
    {
        append_provider_error(path, provider, message)?;
    }
    Ok(())
}

pub(crate) fn append_provider_error(
    path: &Path,
    provider: ProviderKind,
    message: &str,
) -> Result<()> {
    let _lock = acquire_run_lock(path)?;
    let mut state = read_run_state(path)?;
    if state.status == RunStatus::Closed {
        return Ok(());
    }
    state.provider_errors.push(ProviderError {
        provider,
        message: message.to_string(),
        created_at: Utc::now(),
    });
    state.updated_at = Utc::now();
    write_run_state_unlocked(path, &state)?;
    Ok(())
}

pub(crate) fn provider_remaining(state: &ResearchRunState, provider: ProviderKind) -> u32 {
    budget_slot(&state.budgets, provider).saturating_sub(budget_slot(&state.spent, provider))
}

pub(crate) fn remaining_budgets(state: &ResearchRunState) -> ProviderBudgets {
    ProviderBudgets {
        codex_web_queries: provider_remaining(state, ProviderKind::CodexWeb),
        context7_calls: provider_remaining(state, ProviderKind::Context7),
        github_calls: provider_remaining(state, ProviderKind::Github),
        exa_calls: provider_remaining(state, ProviderKind::Exa),
        direct_fetches: provider_remaining(state, ProviderKind::Direct),
        browser_fetches: provider_remaining(state, ProviderKind::Browser),
        firecrawl_calls: provider_remaining(state, ProviderKind::Firecrawl),
    }
}

pub(crate) fn budget_slot(budgets: &ProviderBudgets, provider: ProviderKind) -> u32 {
    match provider {
        ProviderKind::CodexWeb => budgets.codex_web_queries,
        ProviderKind::Context7 => budgets.context7_calls,
        ProviderKind::Github => budgets.github_calls,
        ProviderKind::Exa => budgets.exa_calls,
        ProviderKind::Direct => budgets.direct_fetches,
        ProviderKind::Browser => budgets.browser_fetches,
        ProviderKind::Firecrawl => budgets.firecrawl_calls,
    }
}

pub(crate) fn budget_slot_mut(budgets: &mut ProviderBudgets, provider: ProviderKind) -> &mut u32 {
    match provider {
        ProviderKind::CodexWeb => &mut budgets.codex_web_queries,
        ProviderKind::Context7 => &mut budgets.context7_calls,
        ProviderKind::Github => &mut budgets.github_calls,
        ProviderKind::Exa => &mut budgets.exa_calls,
        ProviderKind::Direct => &mut budgets.direct_fetches,
        ProviderKind::Browser => &mut budgets.browser_fetches,
        ProviderKind::Firecrawl => &mut budgets.firecrawl_calls,
    }
}

pub(crate) fn print_budgets(budgets: &ProviderBudgets) {
    println!("  codex-web: {}", budgets.codex_web_queries);
    println!("  context7: {}", budgets.context7_calls);
    println!("  github: {}", budgets.github_calls);
    println!("  exa: {}", budgets.exa_calls);
    println!("  direct: {}", budgets.direct_fetches);
    println!("  browser: {}", budgets.browser_fetches);
    println!("  firecrawl: {}", budgets.firecrawl_calls);
}
