use crate::*;

pub(crate) async fn github_get(
    client: &reqwest::Client,
    url: &str,
    params: &[(&str, String)],
    retries: u8,
) -> Result<Value> {
    Ok(github_get_response(client, url, params, retries)
        .await?
        .value)
}

pub(crate) async fn github_get_tracked(
    budget: &BudgetArgs,
    client: &reqwest::Client,
    url: &str,
    params: &[(&str, String)],
    retries: u8,
) -> Result<Value> {
    track_provider_result(
        budget,
        ProviderKind::Github,
        github_get(client, url, params, retries),
    )
    .await
}

pub(crate) async fn github_get_response(
    client: &reqwest::Client,
    url: &str,
    params: &[(&str, String)],
    retries: u8,
) -> Result<GithubResponse> {
    let token = github_token();
    for attempt in 0..=retries {
        let mut req = client
            .get(url)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .query(params);
        if let Some(token) = &token {
            req = req.bearer_auth(token);
        }
        let resp = req.send().await?;
        let status = resp.status();
        if status.as_u16() == 403 {
            let remaining = resp
                .headers()
                .get("x-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            let reset = resp
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            bail!("GitHub request forbidden or rate-limited; remaining={remaining} reset={reset}");
        }
        if matches!(status.as_u16(), 429 | 500 | 502 | 503 | 504) && attempt < retries {
            let wait_seconds = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(u64::from(attempt + 1));
            tokio::time::sleep(Duration::from_secs(wait_seconds)).await;
            continue;
        }
        let rate_limit = github_rate_limit_metadata(resp.headers());
        let pagination = github_pagination_metadata(resp.headers());
        let value = resp.error_for_status()?.json::<Value>().await?;
        return Ok(GithubResponse {
            value,
            rate_limit,
            pagination,
        });
    }
    unreachable!("github retry loop always returns or bails")
}

pub(crate) fn github_rate_limit_metadata(headers: &HeaderMap) -> Value {
    let mut out = serde_json::Map::new();
    for (header, key) in [
        ("x-ratelimit-limit", "limit"),
        ("x-ratelimit-remaining", "remaining"),
        ("x-ratelimit-reset", "reset"),
        ("x-ratelimit-used", "used"),
        ("x-ratelimit-resource", "resource"),
    ] {
        if let Some(value) = headers.get(header).and_then(|value| value.to_str().ok()) {
            out.insert(key.to_string(), json!(value));
        }
    }
    Value::Object(out)
}

pub(crate) fn github_pagination_metadata(headers: &HeaderMap) -> Value {
    let link = headers
        .get(LINK)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if link.is_empty() {
        return json!({ "truncated": false });
    }
    json!({
        "truncated": link.split(',').any(|part| part.contains("rel=\"next\"")),
        "link": link
    })
}

pub(crate) async fn context7_send(request: reqwest::RequestBuilder) -> Result<Value> {
    let resp = request.send().await?;
    let status = resp.status();
    if status.as_u16() == 429 {
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        bail!("Context7 rate limited; Retry-After={retry_after}");
    }
    if matches!(status.as_u16(), 202 | 301 | 503 | 504) {
        let status_code = status.as_u16();
        let body = resp.text().await.unwrap_or_default();
        bail!("Context7 returned status={status_code}; body={body}");
    }
    Ok(resp.error_for_status()?.json::<Value>().await?)
}

pub(crate) fn http_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/json, text/plain, text/html"),
    );
    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::limited(8))
        .timeout(Duration::from_secs(30))
        .build()?)
}

pub(crate) fn github_token() -> Option<String> {
    for key in ["GITHUB_TOKEN", "GH_TOKEN"] {
        if let Ok(value) = std::env::var(key)
            && !value.trim().is_empty()
        {
            return Some(value);
        }
    }
    let output = StdCommand::new("gh")
        .args(["auth", "token"])
        .output()
        .ok()?;
    if output.status.success() {
        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !token.is_empty() {
            return Some(token);
        }
    }
    None
}

pub(crate) fn command_version(command: &str, args: &[&str]) -> Option<String> {
    let output = StdCommand::new(command).args(args).output().ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let combined = if stdout.is_empty() { stderr } else { stdout };
        if combined.is_empty() {
            Some("present".to_string())
        } else {
            Some(combined)
        }
    } else {
        None
    }
}
