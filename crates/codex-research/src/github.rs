use crate::*;

pub(crate) struct GithubResponse {
    pub(crate) value: Value,
    pub(crate) rate_limit: Value,
    pub(crate) pagination: Value,
}

pub(crate) async fn handle_github(
    command: GithubCommand,
    config: &ResearchConfig,
    json_out: bool,
) -> Result<()> {
    let client = http_client()?;
    let per_page_default = config.providers.github.per_page_default;
    let per_page_max = config.providers.github.per_page_max;
    let retries = config.providers.github.backoff_retries;
    let (value, source_url, metadata, budget) = match command {
        GithubCommand::SearchRepos {
            query,
            per_page,
            budget,
        } => {
            maybe_debit(
                &budget,
                ProviderKind::Github,
                1,
                Some("github search repos"),
            )?;
            let metadata_query = metadata_text(&query, config);
            let per_page = per_page.unwrap_or(per_page_default).min(per_page_max);
            let url = "https://api.github.com/search/repositories";
            let response = track_provider_result(
                &budget,
                ProviderKind::Github,
                github_get_response(
                    &client,
                    url,
                    &[("q", query.clone()), ("per_page", per_page.to_string())],
                    retries,
                ),
            )
            .await?;
            let metadata = merge_metadata(
                json!({ "operation": "search-repos", "query": metadata_query, "per_page": per_page, "limitations": github_search_limitations("repositories") }),
                json!({ "rate_limit": response.rate_limit }),
            );
            (response.value, github_api_source_url(url), metadata, budget)
        }
        GithubCommand::SearchCode {
            query,
            per_page,
            budget,
        } => {
            maybe_debit(&budget, ProviderKind::Github, 1, Some("github search code"))?;
            let metadata_query = metadata_text(&query, config);
            let per_page = per_page.unwrap_or(per_page_default).min(per_page_max);
            let url = "https://api.github.com/search/code";
            let response = track_provider_result(
                &budget,
                ProviderKind::Github,
                github_get_response(
                    &client,
                    url,
                    &[("q", query.clone()), ("per_page", per_page.to_string())],
                    retries,
                ),
            )
            .await?;
            let metadata = merge_metadata(
                json!({ "operation": "search-code", "query": metadata_query, "per_page": per_page, "limitations": github_search_limitations("code") }),
                json!({ "rate_limit": response.rate_limit }),
            );
            (response.value, github_api_source_url(url), metadata, budget)
        }
        GithubCommand::SearchIssues {
            query,
            per_page,
            budget,
        } => {
            maybe_debit(
                &budget,
                ProviderKind::Github,
                1,
                Some("github search issues"),
            )?;
            let metadata_query = metadata_text(&query, config);
            let per_page = per_page.unwrap_or(per_page_default).min(per_page_max);
            let url = "https://api.github.com/search/issues";
            let response = track_provider_result(
                &budget,
                ProviderKind::Github,
                github_get_response(
                    &client,
                    url,
                    &[("q", query.clone()), ("per_page", per_page.to_string())],
                    retries,
                ),
            )
            .await?;
            let metadata = merge_metadata(
                json!({ "operation": "search-issues", "query": metadata_query, "per_page": per_page, "limitations": github_search_limitations("issues") }),
                json!({ "rate_limit": response.rate_limit }),
            );
            (response.value, github_api_source_url(url), metadata, budget)
        }
        GithubCommand::Releases {
            repo,
            per_page,
            budget,
        } => {
            maybe_debit(&budget, ProviderKind::Github, 1, Some("github releases"))?;
            let per_page = per_page.unwrap_or(per_page_default).min(per_page_max);
            let url = format!("https://api.github.com/repos/{repo}/releases");
            let value = github_get_tracked(
                &budget,
                &client,
                &url,
                &[("per_page", per_page.to_string())],
                retries,
            )
            .await?;
            (
                value,
                github_repo_url(&repo, "releases"),
                json!({ "operation": "releases", "repo": repo, "per_page": per_page }),
                budget,
            )
        }
        GithubCommand::Release {
            repo,
            tag,
            latest,
            budget,
        } => {
            maybe_debit(&budget, ProviderKind::Github, 1, Some("github release"))?;
            if latest == tag.is_some() {
                bail!("pass exactly one of --latest or --tag <tag>");
            }
            let (url, operation, source_url) = if latest {
                (
                    format!("https://api.github.com/repos/{repo}/releases/latest"),
                    "release-latest".to_string(),
                    github_repo_url(&repo, "releases/latest"),
                )
            } else {
                let tag = tag.clone().expect("validated tag presence");
                (
                    format!(
                        "https://api.github.com/repos/{repo}/releases/tags/{}",
                        path_segment(&tag)
                    ),
                    "release-by-tag".to_string(),
                    github_repo_url(&repo, &format!("releases/tag/{tag}")),
                )
            };
            let value = github_get_tracked(&budget, &client, &url, &[], retries).await?;
            (
                value,
                source_url,
                json!({ "operation": operation, "repo": repo, "tag": tag, "latest": latest }),
                budget,
            )
        }
        GithubCommand::Compare {
            repo,
            base,
            head,
            per_page,
            page,
            budget,
        } => {
            maybe_debit(&budget, ProviderKind::Github, 1, Some("github compare"))?;
            let per_page = per_page.unwrap_or(per_page_default).min(per_page_max);
            let basehead = format!("{base}...{head}");
            let url = format!(
                "https://api.github.com/repos/{repo}/compare/{}",
                path_segment(&basehead)
            );
            let value = github_get_tracked(
                &budget,
                &client,
                &url,
                &[
                    ("per_page", per_page.to_string()),
                    ("page", page.to_string()),
                ],
                retries,
            )
            .await?;
            (
                normalize_compare(value),
                github_repo_url(&repo, &format!("compare/{basehead}")),
                json!({ "operation": "compare", "repo": repo, "base": base, "head": head, "per_page": per_page, "page": page }),
                budget,
            )
        }
        GithubCommand::Tags {
            repo,
            per_page,
            budget,
        } => {
            maybe_debit(&budget, ProviderKind::Github, 1, Some("github tags"))?;
            let per_page = per_page.unwrap_or(per_page_default).min(per_page_max);
            let url = format!("https://api.github.com/repos/{repo}/tags");
            let value = github_get_tracked(
                &budget,
                &client,
                &url,
                &[("per_page", per_page.to_string())],
                retries,
            )
            .await?;
            (
                value,
                github_repo_url(&repo, "tags"),
                json!({ "operation": "tags", "repo": repo, "per_page": per_page }),
                budget,
            )
        }
        GithubCommand::Issue {
            repo,
            number,
            comments,
            budget,
        } => {
            let call_count = github_issue_call_count(comments);
            maybe_debit(
                &budget,
                ProviderKind::Github,
                call_count,
                Some("github issue"),
            )?;
            let issue_url = format!("https://api.github.com/repos/{repo}/issues/{number}");
            let issue = github_get_tracked(&budget, &client, &issue_url, &[], retries).await?;
            let mut pagination = serde_json::Map::new();
            let comments_value = if comments {
                let url = format!("https://api.github.com/repos/{repo}/issues/{number}/comments");
                let response = track_provider_result(
                    &budget,
                    ProviderKind::Github,
                    github_get_response(&client, &url, &[("per_page", "100".to_string())], retries),
                )
                .await?;
                pagination.insert("comments".to_string(), response.pagination);
                response.value
            } else {
                json!([])
            };
            (
                json!({ "issue": issue, "comments": comments_value }),
                github_repo_url(&repo, &format!("issues/{number}")),
                json!({ "operation": "issue", "repo": repo, "number": number, "comments": comments, "github_calls": call_count, "pagination": pagination }),
                budget,
            )
        }
        GithubCommand::Pr {
            repo,
            number,
            files,
            comments,
            reviews,
            budget,
        } => {
            let call_count = github_pr_call_count(files, comments, reviews);
            maybe_debit(&budget, ProviderKind::Github, call_count, Some("github pr"))?;
            let pr_url = format!("https://api.github.com/repos/{repo}/pulls/{number}");
            let pr = github_get_tracked(&budget, &client, &pr_url, &[], retries).await?;
            let mut pagination = serde_json::Map::new();
            let files_value = if files {
                let url = format!("https://api.github.com/repos/{repo}/pulls/{number}/files");
                let response = track_provider_result(
                    &budget,
                    ProviderKind::Github,
                    github_get_response(&client, &url, &[("per_page", "100".to_string())], retries),
                )
                .await?;
                pagination.insert("files".to_string(), response.pagination);
                response.value
            } else {
                json!([])
            };
            let comments_value = if comments {
                let url = format!("https://api.github.com/repos/{repo}/issues/{number}/comments");
                let issue_response = track_provider_result(
                    &budget,
                    ProviderKind::Github,
                    github_get_response(&client, &url, &[("per_page", "100".to_string())], retries),
                )
                .await?;
                pagination.insert("issue_comments".to_string(), issue_response.pagination);
                let url = format!("https://api.github.com/repos/{repo}/pulls/{number}/comments");
                let review_response = track_provider_result(
                    &budget,
                    ProviderKind::Github,
                    github_get_response(&client, &url, &[("per_page", "100".to_string())], retries),
                )
                .await?;
                pagination.insert("review_comments".to_string(), review_response.pagination);
                json!({ "issue_comments": issue_response.value, "review_comments": review_response.value })
            } else {
                json!({ "issue_comments": [], "review_comments": [] })
            };
            let reviews_value = if reviews {
                let url = format!("https://api.github.com/repos/{repo}/pulls/{number}/reviews");
                let response = track_provider_result(
                    &budget,
                    ProviderKind::Github,
                    github_get_response(&client, &url, &[("per_page", "100".to_string())], retries),
                )
                .await?;
                pagination.insert("reviews".to_string(), response.pagination);
                response.value
            } else {
                json!([])
            };
            (
                json!({ "pull_request": pr, "files": files_value, "comments": comments_value, "reviews": reviews_value }),
                github_repo_url(&repo, &format!("pull/{number}")),
                json!({ "operation": "pr", "repo": repo, "number": number, "files": files, "comments": comments, "reviews": reviews, "github_calls": call_count, "pagination": pagination }),
                budget,
            )
        }
        GithubCommand::File {
            repo,
            path,
            r#ref,
            budget,
        } => {
            maybe_debit(&budget, ProviderKind::Github, 1, Some("github file"))?;
            let url = format!(
                "https://api.github.com/repos/{repo}/contents/{}",
                slash_path(&path)
            );
            let value =
                github_get_tracked(&budget, &client, &url, &[("ref", r#ref.clone())], retries)
                    .await?;
            (
                value,
                github_repo_url(&repo, &format!("blob/{ref_name}/{path}", ref_name = r#ref)),
                json!({ "operation": "file", "repo": repo, "path": path, "ref": r#ref }),
                budget,
            )
        }
    };
    let paths = research_paths()?;
    init_db(&paths)?;
    let output_metadata = metadata.clone();
    let source_id = record_source_cache(
        &paths,
        SourceCacheInsert {
            url: &source_url,
            provider: "github",
            status: Some(200),
            content_hash: None,
            route: Some("github"),
            title: None,
            canonical_url: Some(&source_url),
            freshness_status: "current",
            privacy_classification: privacy_class_name(classify_privacy(&source_url)),
            raw_body_stored: false,
            metadata,
            redact_query_secrets: config.privacy.redact_query_secrets,
        },
    )?;
    attach_source_to_run(&budget, &source_id)?;
    let value = json!({ "source_id": source_id, "provider": "github", "metadata": output_metadata, "data": value });

    if json_out {
        print_json(&value)
    } else {
        println!("{}", serde_json::to_string_pretty(&value)?);
        Ok(())
    }
}
