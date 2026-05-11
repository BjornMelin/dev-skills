use crate::*;

pub(crate) fn handle_ledger(command: LedgerCommand, json_out: bool) -> Result<()> {
    match command {
        LedgerCommand::Init { path } => {
            ensure_parent(&path)?;
            if !path.exists() {
                File::create(&path)?;
            }
            if json_out {
                print_json(&json!({ "ledger": path, "created": true }))
            } else {
                println!("ledger: {}", path.display());
                Ok(())
            }
        }
        LedgerCommand::AddSource(args) => {
            ensure_parent(&args.ledger)?;
            let (id, provider, url, title, route) = if let Some(source_id) = args.from_cache {
                if args.provider.is_some() || args.url.is_some() || args.route.is_some() {
                    bail!(
                        "--from-cache cannot be combined with --provider, --url, or --route; use --title only to override the cached title"
                    );
                }
                let paths = research_paths()?;
                init_db(&paths)?;
                let cached = cached_source(&paths, &source_id)?
                    .with_context(|| format!("cached source not found: {source_id}"))?;
                (
                    cached.id,
                    cached.provider,
                    cached.url,
                    args.title.or(cached.title),
                    cached.route,
                )
            } else {
                let provider = args
                    .provider
                    .context("--provider is required unless --from-cache is used")?;
                let url = args
                    .url
                    .context("--url is required unless --from-cache is used")?;
                (
                    short_hash(format!("{}:{}:{}", provider, url, Utc::now())),
                    provider,
                    url,
                    args.title,
                    args.route,
                )
            };
            let record = LedgerRecord::Source(SourceRecord {
                id: id.clone(),
                provider,
                url,
                title,
                route,
                fetched_at: Utc::now(),
            });
            append_ledger_record(&args.ledger, &record)?;
            if json_out {
                print_json(&json!({ "source_id": id }))
            } else {
                println!("{id}");
                Ok(())
            }
        }
        LedgerCommand::AddClaim(args) => {
            ensure_parent(&args.ledger)?;
            let id = short_hash(format!("{}:{:?}:{}", args.text, args.sources, Utc::now()));
            let record = LedgerRecord::Claim(ClaimRecord {
                id: id.clone(),
                text: args.text,
                confidence: args.confidence,
                sources: args.sources,
                note: args.note,
                created_at: Utc::now(),
            });
            append_ledger_record(&args.ledger, &record)?;
            if json_out {
                print_json(&json!({ "claim_id": id }))
            } else {
                println!("{id}");
                Ok(())
            }
        }
        LedgerCommand::Inspect { path } => {
            let records = read_ledger_records(&path)?;
            let sources = records
                .iter()
                .filter(|r| matches!(r, LedgerRecord::Source(_)))
                .count();
            let claims = records
                .iter()
                .filter(|r| matches!(r, LedgerRecord::Claim(_)))
                .count();
            if json_out {
                print_json(&json!({
                    "ledger": path,
                    "sources": sources,
                    "claims": claims,
                    "records": records.len()
                }))
            } else {
                println!("sources: {sources}");
                println!("claims: {claims}");
                println!("records: {}", records.len());
                Ok(())
            }
        }
    }
}

pub(crate) fn render_report(args: ReportArgs, json_out: bool) -> Result<()> {
    let records = read_ledger_records(&args.ledger)?;
    let mut sources = Vec::new();
    let mut claims = Vec::new();
    for record in records {
        match record {
            LedgerRecord::Source(source) => sources.push(source),
            LedgerRecord::Claim(claim) => claims.push(claim),
        }
    }

    let mut output = String::new();
    output.push_str("# Research Report\n\n");
    output.push_str("## Claims\n\n");
    if claims.is_empty() {
        output.push_str("No claims recorded.\n\n");
    }
    for claim in &claims {
        output.push_str(&format!(
            "- `{}` confidence {:.2}: {}\n",
            claim.id, claim.confidence, claim.text
        ));
        if !claim.sources.is_empty() {
            output.push_str(&format!("  sources: {}\n", claim.sources.join(", ")));
        }
        if let Some(note) = &claim.note {
            output.push_str(&format!("  note: {note}\n"));
        }
    }
    output.push_str("\n## Sources\n\n");
    if sources.is_empty() {
        output.push_str("No sources recorded.\n");
    }
    for source in &sources {
        let title = source.title.as_deref().unwrap_or(&source.url);
        output.push_str(&format!(
            "- `{}` [{}]({}) via {}\n",
            source.id, title, source.url, source.provider
        ));
    }

    if let Some(out) = args.out {
        ensure_parent(&out)?;
        fs::write(&out, output.as_bytes())?;
        if json_out {
            print_json(&json!({ "report": out }))
        } else {
            println!("report: {}", out.display());
            Ok(())
        }
    } else if json_out {
        print_json(&json!({ "markdown": output }))
    } else {
        print!("{output}");
        Ok(())
    }
}
