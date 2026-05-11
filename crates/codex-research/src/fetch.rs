use crate::*;

pub(crate) async fn handle_fetch(
    command: FetchCommand,
    config: &ResearchConfig,
    json_out: bool,
) -> Result<()> {
    match command {
        FetchCommand::Probe {
            url,
            max_bytes,
            budget,
        } => {
            maybe_debit(&budget, ProviderKind::Direct, 1, Some("fetch probe"))?;
            let client = http_client()?;
            let report = track_provider_result(
                &budget,
                ProviderKind::Direct,
                probe_url(&client, &url, max_bytes),
            )
            .await?;
            if json_out {
                print_json(&report)
            } else {
                println!("route: {}", route_name(report.route));
                println!("reason: {}", report.reason);
                println!("status: {:?}", report.status);
                println!("content_type: {:?}", report.content_type);
                println!("text_chars: {}", report.text_chars);
                if !report.app_shell_markers.is_empty() {
                    println!("app_shell_markers: {}", report.app_shell_markers.join(", "));
                }
                Ok(())
            }
        }
        FetchCommand::Get {
            url,
            max_bytes,
            store,
            budget,
        } => {
            maybe_debit(&budget, ProviderKind::Direct, 1, Some("fetch get"))?;
            let client = http_client()?;
            let fetched = track_provider_result(
                &budget,
                ProviderKind::Direct,
                direct_fetch(&client, &url, max_bytes),
            )
            .await?;
            let paths = research_paths()?;
            init_db(&paths)?;
            let mut source_id = None;
            if store {
                let hash = store_blob(&paths, &fetched.raw_body)?;
                source_id = Some(record_source_cache(
                    &paths,
                    SourceCacheInsert {
                        url: &url,
                        provider: "direct",
                        status: Some(fetched.status),
                        content_hash: Some(&hash),
                        route: Some("direct"),
                        title: None,
                        canonical_url: None,
                        freshness_status: "current",
                        privacy_classification: privacy_class_name(classify_privacy(&url)),
                        raw_body_stored: true,
                        metadata: json!({ "bytes": fetched.bytes }),
                        redact_query_secrets: config.privacy.redact_query_secrets,
                    },
                )?);
                if let Some(source_id) = &source_id {
                    attach_source_to_run(&budget, source_id)?;
                }
            }
            record_route_memory(
                &paths,
                &url,
                Route::Direct,
                true,
                Some(fetched.status),
                "direct fetch succeeded",
            )?;
            if json_out {
                print_json(&FetchedOutput { source_id, fetched })
            } else {
                println!("{}", fetched.body);
                Ok(())
            }
        }
        FetchCommand::Firecrawl {
            url,
            fresh,
            no_store_in_cache,
            timeout_ms,
            privacy,
            allow_private_external,
            budget,
        } => {
            let privacy_class = privacy.unwrap_or_else(|| classify_privacy(&url));
            enforce_external_privacy(privacy_class, allow_private_external, config, "firecrawl")?;
            maybe_debit(
                &budget,
                ProviderKind::Firecrawl,
                1,
                Some("firecrawl scrape"),
            )?;
            let store_in_cache = effective_firecrawl_store_in_cache(no_store_in_cache, config);
            let scrape = track_provider_result(
                &budget,
                ProviderKind::Firecrawl,
                firecrawl_scrape(&url, fresh, store_in_cache, timeout_ms, config),
            )
            .await?;
            let paths = research_paths()?;
            init_db(&paths)?;
            let source_id = record_source_cache(
                &paths,
                SourceCacheInsert {
                    url: &url,
                    provider: "firecrawl",
                    status: Some(scrape.status),
                    content_hash: None,
                    route: Some("firecrawl"),
                    title: None,
                    canonical_url: Some(&url),
                    freshness_status: if fresh { "current" } else { "unverified" },
                    privacy_classification: privacy_class_name(privacy_class),
                    raw_body_stored: false,
                    metadata: json!({
                        "fresh": fresh,
                        "store_in_cache": store_in_cache,
                        "timeout_ms": timeout_ms
                    }),
                    redact_query_secrets: config.privacy.redact_query_secrets,
                },
            )?;
            attach_source_to_run(&budget, &source_id)?;
            record_route_memory(
                &paths,
                &url,
                Route::Firecrawl,
                true,
                Some(scrape.status),
                "firecrawl scrape succeeded",
            )?;
            let value =
                json!({ "source_id": source_id, "provider": "firecrawl", "data": scrape.value });
            print_json(&value)
        }
    }
}
