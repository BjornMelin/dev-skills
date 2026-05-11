use crate::*;

pub(crate) fn apply_route_memory(url: &str, report: &mut ProbeReport) -> Result<()> {
    if report.route == Route::Github {
        return Ok(());
    }
    let paths = research_paths()?;
    init_db(&paths)?;
    if let Some(hit) = route_memory_for_url(&paths, url)? {
        if hit.successes > hit.failures
            && let Some(route) = route_from_name(&hit.preferred_route)
        {
            report.reason = format!(
                "{} Route memory for {} prefers {} (successes={} failures={}).",
                report.reason, hit.domain, hit.preferred_route, hit.successes, hit.failures
            );
            report.route = route;
        }
        report.route_memory.push(hit);
    }
    Ok(())
}

pub(crate) fn route_from_name(value: &str) -> Option<Route> {
    match value {
        "codex-web" => Some(Route::CodexWeb),
        "context7" => Some(Route::Context7),
        "github" => Some(Route::Github),
        "direct" => Some(Route::Direct),
        "agent-browser" => Some(Route::AgentBrowser),
        "firecrawl" => Some(Route::Firecrawl),
        "exa" => Some(Route::Exa),
        "opensrc" => Some(Route::Opensrc),
        _ => None,
    }
}

pub(crate) fn path_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

pub(crate) fn slash_path(value: &str) -> String {
    value
        .split('/')
        .map(path_segment)
        .collect::<Vec<_>>()
        .join("/")
}

pub(crate) fn github_api_source_url(url: &str) -> String {
    url.to_string()
}

pub(crate) fn github_repo_url(repo: &str, path: &str) -> String {
    format!("https://github.com/{repo}/{path}")
}

pub(crate) fn github_search_limitations(kind: &str) -> Value {
    match kind {
        "code" => json!([
            "requires at least one search term",
            "default branch only",
            "files larger than 384 KB are not searchable",
            "strict search rate limits apply",
            "hydrate promising hits before citing"
        ]),
        _ => json!([
            "search can return incomplete_results on timeout",
            "queries have length and boolean-operator limits",
            "hydrate promising hits before citing"
        ]),
    }
}

pub(crate) fn normalize_compare(mut value: Value) -> Value {
    if let Some(files) = value.get("files").and_then(Value::as_array) {
        let summary = files
            .iter()
            .map(|file| {
                json!({
                    "filename": file.get("filename"),
                    "status": file.get("status"),
                    "previous_filename": file.get("previous_filename"),
                    "additions": file.get("additions"),
                    "deletions": file.get("deletions"),
                    "changes": file.get("changes"),
                    "patch_present": file.get("patch").is_some()
                })
            })
            .collect::<Vec<_>>();
        value["file_summary"] = json!(summary);
    }
    value
}

pub(crate) fn merge_metadata(mut left: Value, right: Value) -> Value {
    if let (Some(left), Some(right)) = (left.as_object_mut(), right.as_object()) {
        for (key, value) in right {
            left.insert(key.clone(), value.clone());
        }
    }
    left
}

pub(crate) fn short_hash(value: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    format!("{:x}", hasher.finalize())[..12].to_string()
}

pub(crate) fn required_env(name: &str) -> Result<String> {
    let value = std::env::var(name).with_context(|| format!("{name} is required"))?;
    if value.trim().is_empty() {
        bail!("{name} is empty");
    }
    Ok(value)
}

pub(crate) fn route_name(route: Route) -> &'static str {
    match route {
        Route::CodexWeb => "codex-web",
        Route::Context7 => "context7",
        Route::Github => "github",
        Route::Direct => "direct",
        Route::AgentBrowser => "agent-browser",
        Route::Firecrawl => "firecrawl",
        Route::Exa => "exa",
        Route::Opensrc => "opensrc",
    }
}

pub(crate) fn route_list(routes: &[Route]) -> String {
    routes
        .iter()
        .map(|r| route_name(*r))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
