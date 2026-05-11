use crate::*;

pub(crate) fn build_plan(
    query: &str,
    profile: ResearchProfile,
    topic: TopicKind,
    config: &ResearchConfig,
) -> ResearchPlan {
    let budgets = profile_budget(config, profile);
    let route_order = match topic {
        TopicKind::Docs => vec![
            Route::Context7,
            Route::CodexWeb,
            Route::Github,
            Route::Direct,
            Route::AgentBrowser,
            Route::Firecrawl,
            Route::Exa,
        ],
        TopicKind::Github => vec![Route::Github, Route::CodexWeb, Route::Direct, Route::Exa],
        TopicKind::Dependency => vec![
            Route::Context7,
            Route::Opensrc,
            Route::Github,
            Route::CodexWeb,
            Route::Exa,
        ],
        TopicKind::Openai => vec![
            Route::CodexWeb,
            Route::Context7,
            Route::Github,
            Route::Direct,
        ],
        TopicKind::Rendered => vec![
            Route::Direct,
            Route::AgentBrowser,
            Route::Firecrawl,
            Route::CodexWeb,
        ],
        TopicKind::General => vec![
            Route::CodexWeb,
            Route::Context7,
            Route::Github,
            Route::Direct,
            Route::Exa,
            Route::AgentBrowser,
            Route::Firecrawl,
        ],
    };

    ResearchPlan {
        query: query.to_string(),
        profile,
        budgets,
        route_order,
        rules: vec![
            "Treat search results as leads until hydrated into source records.".to_string(),
            "Prefer primary sources: official docs, source repos, release notes, API references, and maintained issue threads.".to_string(),
            "Use Codex-native web tools for narrow current facts and official docs checks; the CLI records instructions and evidence, not web.run calls.".to_string(),
            "Use Exa for broad semantic discovery, repository inspiration, and filtered multi-source exploration.".to_string(),
            "Use Firecrawl under classified policy: public/cacheable by default, maxAge=0 for latest-critical pages, no private content unless explicitly allowed.".to_string(),
        ],
    }
}

pub(crate) async fn probe_url(
    client: &reqwest::Client,
    url: &str,
    max_bytes: usize,
) -> Result<ProbeReport> {
    let mut status = None;
    let mut content_type = None;
    let mut content_length = None;

    if let Ok(resp) = client.head(url).send().await {
        status = Some(resp.status().as_u16());
        content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        content_length = resp
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());
    }

    let fetched = direct_fetch(client, url, max_bytes).await;
    match fetched {
        Ok(fetched) => {
            let mut report = classify_body(
                url,
                fetched.content_type.as_deref(),
                content_length,
                &fetched.body,
            );
            report.status = Some(fetched.status);
            if report.content_type.is_none() {
                report.content_type = content_type;
            }
            apply_route_memory(url, &mut report)?;
            Ok(report)
        }
        Err(_) => {
            let mut report = classify_body(url, content_type.as_deref(), content_length, "");
            report.status = status;
            apply_route_memory(url, &mut report)?;
            Ok(report)
        }
    }
}

#[derive(Serialize)]
pub(crate) struct FetchedBody {
    pub(crate) url: String,
    pub(crate) status: u16,
    pub(crate) content_type: Option<String>,
    pub(crate) bytes: usize,
    pub(crate) body: String,
    #[serde(skip_serializing)]
    pub(crate) raw_body: Vec<u8>,
}

#[derive(Serialize)]
pub(crate) struct FetchedOutput {
    pub(crate) source_id: Option<String>,
    #[serde(flatten)]
    pub(crate) fetched: FetchedBody,
}

pub(crate) async fn direct_fetch(
    client: &reqwest::Client,
    url: &str,
    max_bytes: usize,
) -> Result<FetchedBody> {
    let range = format!("bytes=0-{}", max_bytes.saturating_sub(1));
    let resp = client
        .get(url)
        .header(RANGE, range)
        .send()
        .await
        .with_context(|| format!("fetch failed: {url}"))?;
    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = resp.bytes().await?;
    let slice = if bytes.len() > max_bytes {
        &bytes[..max_bytes]
    } else {
        &bytes
    };
    let body = String::from_utf8_lossy(slice).to_string();
    let raw_body = slice.to_vec();
    Ok(FetchedBody {
        url: url.to_string(),
        status,
        content_type,
        bytes: slice.len(),
        body,
        raw_body,
    })
}

pub(crate) fn classify_body(
    url: &str,
    content_type: Option<&str>,
    content_length: Option<u64>,
    body: &str,
) -> ProbeReport {
    if is_github_url(url) {
        return ProbeReport {
            url: url.to_string(),
            status: None,
            content_type: content_type.map(str::to_string),
            content_length,
            text_chars: body.len(),
            script_markers: 0,
            app_shell_markers: vec!["github".to_string()],
            route: Route::Github,
            reason: "GitHub URL should be hydrated through GitHub APIs when possible.".to_string(),
            route_memory: Vec::new(),
        };
    }

    let lower = body.to_ascii_lowercase();
    let stripped = strip_tags(body);
    let text_chars = stripped.chars().filter(|c| !c.is_whitespace()).count();
    let script_markers = lower.matches("<script").count();
    let mut app_shell_markers = Vec::new();
    for marker in [
        "__next_data__",
        "id=\"__next\"",
        "window.__nuxt__",
        "id=\"root\"",
        "vite",
        "enable javascript",
        "requires javascript",
    ] {
        if lower.contains(marker) {
            app_shell_markers.push(marker.to_string());
        }
    }

    let ctype = content_type.unwrap_or_default().to_ascii_lowercase();
    let (route, reason) = if ctype.contains("pdf") {
        (
            Route::Firecrawl,
            "PDF-like content should use a parser or Firecrawl.".to_string(),
        )
    } else if ctype.contains("json") || ctype.contains("text/plain") || ctype.contains("markdown") {
        (
            Route::Direct,
            "Text-like content is suitable for direct fetch.".to_string(),
        )
    } else if lower.contains("cloudflare") && text_chars < 500 {
        (
            Route::Firecrawl,
            "Likely bot/block page; use Firecrawl if public and allowed.".to_string(),
        )
    } else if !app_shell_markers.is_empty() && text_chars < 1_200 {
        (
            Route::AgentBrowser,
            "HTML looks like an app shell with low text density.".to_string(),
        )
    } else if script_markers > 25 && text_chars < 2_000 {
        (
            Route::AgentBrowser,
            "High script count and low text density; render before trusting extracted text."
                .to_string(),
        )
    } else {
        (
            Route::Direct,
            "Direct fetch likely has enough extractable text.".to_string(),
        )
    };

    ProbeReport {
        url: url.to_string(),
        status: None,
        content_type: content_type.map(str::to_string),
        content_length,
        text_chars,
        script_markers,
        app_shell_markers,
        route,
        reason,
        route_memory: Vec::new(),
    }
}
