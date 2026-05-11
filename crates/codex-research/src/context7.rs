use crate::*;

pub(crate) async fn handle_context7(
    command: Context7Command,
    config: &ResearchConfig,
    json_out: bool,
) -> Result<()> {
    let api_key = required_env("CONTEXT7_API_KEY")?;
    let client = http_client()?;
    let (value, source_url, metadata, budget) = match command {
        Context7Command::Search {
            library,
            query,
            version,
            budget,
        } => {
            maybe_debit(&budget, ProviderKind::Context7, 1, Some("context7 search"))?;
            let metadata_query = metadata_text(&query, config);
            let version_pin_hint = version.as_ref().map(|version| {
                json!({
                    "slash": format!("/owner/repo/{version}"),
                    "at": format!("/owner/repo@{version}")
                })
            });
            let value = track_provider_result(
                &budget,
                ProviderKind::Context7,
                context7_send(
                    client
                        .get("https://context7.com/api/v2/libs/search")
                        .bearer_auth(&api_key)
                        .query(&[("libraryName", library.clone()), ("query", query.clone())]),
                ),
            )
            .await?;
            (
                value,
                "https://context7.com/api/v2/libs/search".to_string(),
                json!({
                    "operation": "search",
                    "library": library,
                    "query": metadata_query,
                    "version": version,
                    "version_pin_hint": version_pin_hint
                }),
                budget,
            )
        }
        Context7Command::Context {
            library_id,
            query,
            fast,
            budget,
        } => {
            maybe_debit(&budget, ProviderKind::Context7, 1, Some("context7 context"))?;
            let metadata_query = metadata_text(&query, config);
            let value = track_provider_result(
                &budget,
                ProviderKind::Context7,
                context7_send(
                    client
                        .get("https://context7.com/api/v2/context")
                        .bearer_auth(&api_key)
                        .query(&[
                            ("libraryId", library_id.clone()),
                            ("query", query.clone()),
                            ("type", "json".to_string()),
                            ("fast", fast.to_string()),
                        ]),
                ),
            )
            .await?;
            (
                value,
                format!("https://context7.com{library_id}"),
                json!({ "operation": "context", "library_id": library_id, "query": metadata_query, "fast": fast }),
                budget,
            )
        }
        Context7Command::Refresh {
            library_name,
            branch,
            budget,
        } => {
            maybe_debit(&budget, ProviderKind::Context7, 1, Some("context7 refresh"))?;
            let mut body = json!({ "libraryName": library_name });
            if let Some(branch) = branch.clone() {
                body["branch"] = json!(branch);
            }
            let value = track_provider_result(
                &budget,
                ProviderKind::Context7,
                context7_send(
                    client
                        .post("https://context7.com/api/v1/refresh")
                        .bearer_auth(&api_key)
                        .json(&body),
                ),
            )
            .await?;
            (
                value,
                format!("https://context7.com{library_name}"),
                json!({ "operation": "refresh", "library_name": library_name, "branch": branch }),
                budget,
            )
        }
    };
    let paths = research_paths()?;
    init_db(&paths)?;
    let source_id = record_source_cache(
        &paths,
        SourceCacheInsert {
            url: &source_url,
            provider: "context7",
            status: Some(200),
            content_hash: None,
            route: Some("context7"),
            title: None,
            canonical_url: Some(&source_url),
            freshness_status: "unverified",
            privacy_classification: "public",
            raw_body_stored: false,
            metadata: merge_metadata(
                metadata,
                json!({ "cache_ttl_hours": config.providers.context7.cache_ttl_hours }),
            ),
            redact_query_secrets: config.privacy.redact_query_secrets,
        },
    )?;
    attach_source_to_run(&budget, &source_id)?;
    let value = json!({ "source_id": source_id, "provider": "context7", "data": value });

    if json_out {
        print_json(&value)
    } else {
        println!("{}", serde_json::to_string_pretty(&value)?);
        Ok(())
    }
}
