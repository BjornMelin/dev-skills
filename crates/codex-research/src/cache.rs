use crate::*;

pub(crate) fn handle_cache(command: CacheCommand, json_out: bool) -> Result<()> {
    let paths = research_paths()?;
    match command {
        CacheCommand::Init => {
            init_db(&paths)?;
            if json_out {
                print_json(&json!({
                    "cache_dir": paths.cache_dir,
                    "database": paths.database,
                    "blobs_dir": paths.blobs_dir
                }))
            } else {
                println!("initialized {}", paths.database.display());
                Ok(())
            }
        }
        CacheCommand::Stats => {
            init_db(&paths)?;
            let conn = Connection::open(&paths.database)?;
            let sources: i64 =
                conn.query_row("select count(*) from sources", [], |row| row.get(0))?;
            let routes: i64 =
                conn.query_row("select count(*) from route_memory", [], |row| row.get(0))?;
            let blobs = count_blobs(&paths.blobs_dir)?;
            if json_out {
                print_json(&json!({
                    "database": paths.database,
                    "sources": sources,
                    "route_memory": routes,
                    "blobs": blobs
                }))
            } else {
                println!("sources: {sources}");
                println!("route_memory: {routes}");
                println!("blobs: {blobs}");
                Ok(())
            }
        }
        CacheCommand::Sources { provider, limit } => {
            init_db(&paths)?;
            let sources = list_cached_sources(&paths, provider.as_deref(), limit)?;
            if json_out {
                print_json(&json!({ "sources": sources }))
            } else {
                for source in sources {
                    println!(
                        "{} {} {} {}",
                        source.id, source.provider, source.fetched_at, source.url
                    );
                }
                Ok(())
            }
        }
        CacheCommand::Source { source_id } => {
            init_db(&paths)?;
            let source = cached_source(&paths, &source_id)?
                .with_context(|| format!("cached source not found: {source_id}"))?;
            if json_out {
                print_json(&source)
            } else {
                println!("{}", serde_json::to_string_pretty(&source)?);
                Ok(())
            }
        }
        CacheCommand::RouteMemory { domain } => {
            init_db(&paths)?;
            let rows = list_route_memory(&paths, domain.as_deref())?;
            if json_out {
                print_json(&json!({ "route_memory": rows }))
            } else {
                for row in rows {
                    println!(
                        "{} -> {} (successes={} failures={})",
                        row.domain, row.preferred_route, row.successes, row.failures
                    );
                }
                Ok(())
            }
        }
        CacheCommand::Prune {
            older_than_days,
            dry_run,
        } => {
            init_db(&paths)?;
            let pruned = prune_cache(&paths, older_than_days, dry_run)?;
            if json_out {
                print_json(&json!({ "dry_run": dry_run, "sources": pruned }))
            } else {
                println!("sources: {pruned}");
                Ok(())
            }
        }
    }
}
