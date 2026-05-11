use crate::*;

pub(crate) struct FirecrawlScrape {
    pub(crate) status: u16,
    pub(crate) value: Value,
}

pub(crate) async fn firecrawl_scrape(
    url: &str,
    fresh: bool,
    store_in_cache: bool,
    timeout_ms: u64,
    config: &ResearchConfig,
) -> Result<FirecrawlScrape> {
    let api_key = required_env("FIRECRAWL_API_KEY")?;
    let client = http_client()?;
    let max_age = if fresh {
        config.providers.firecrawl.latest_critical_max_age_ms
    } else {
        config.providers.firecrawl.default_max_age_ms
    };
    let body = json!({
        "url": url,
        "formats": ["markdown"],
        "onlyMainContent": true,
        "maxAge": max_age,
        "storeInCache": store_in_cache,
        "timeout": timeout_ms
    });
    let resp = client
        .post("https://api.firecrawl.dev/v2/scrape")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;
    if resp.status().as_u16() == 429 {
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        bail!("Firecrawl rate limited; Retry-After={retry_after}");
    }
    let status = resp.status().as_u16();
    let value = resp.error_for_status()?.json::<Value>().await?;
    Ok(FirecrawlScrape { status, value })
}
