use crate::*;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SourceCacheRecord {
    pub(crate) id: String,
    pub(crate) provider: String,
    pub(crate) route: Option<String>,
    pub(crate) url: String,
    pub(crate) canonical_url: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) fetched_at: String,
    pub(crate) freshness_status: String,
    pub(crate) privacy_classification: String,
    pub(crate) status: Option<u16>,
    pub(crate) content_hash: Option<String>,
    pub(crate) raw_body_stored: bool,
    pub(crate) metadata: Value,
}

pub(crate) fn init_db(paths: &ResearchPaths) -> Result<()> {
    fs::create_dir_all(&paths.cache_dir)?;
    fs::create_dir_all(&paths.blobs_dir)?;
    let conn = Connection::open(&paths.database)?;
    conn.execute_batch(
        "
        create table if not exists schema_migrations (
            version integer primary key,
            applied_at text not null
        );
        create table if not exists sources (
            id text primary key,
            url text not null,
            provider text not null,
            fetched_at text not null,
            content_hash text,
            status integer,
            route text,
            metadata_json text
        );
        create table if not exists route_memory (
            domain text primary key,
            preferred_route text not null,
            successes integer not null default 0,
            failures integer not null default 0,
            updated_at text not null
        );
        create table if not exists claims (
            id text primary key,
            text text not null,
            confidence real not null,
            source_ids_json text not null,
            created_at text not null,
            status text not null default 'open'
        );
        ",
    )?;
    add_column_if_missing(&conn, "sources", "canonical_url", "text")?;
    add_column_if_missing(&conn, "sources", "title", "text")?;
    add_column_if_missing(
        &conn,
        "sources",
        "freshness_status",
        "text not null default 'unverified'",
    )?;
    add_column_if_missing(
        &conn,
        "sources",
        "privacy_classification",
        "text not null default 'unverified'",
    )?;
    add_column_if_missing(
        &conn,
        "sources",
        "raw_body_stored",
        "integer not null default 0",
    )?;
    add_column_if_missing(&conn, "route_memory", "last_reason", "text")?;
    add_column_if_missing(&conn, "route_memory", "last_status", "integer")?;
    conn.execute(
        "insert or ignore into schema_migrations (version, applied_at) values (?1, ?2)",
        params![1_i64, Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub(crate) fn store_blob(paths: &ResearchPaths, bytes: &[u8]) -> Result<String> {
    fs::create_dir_all(&paths.blobs_dir)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = format!("{:x}", hasher.finalize());
    let shard = paths.blobs_dir.join(&hash[0..2]);
    fs::create_dir_all(&shard)?;
    let path = shard.join(&hash);
    if !path.exists() {
        fs::write(path, bytes)?;
    }
    Ok(hash)
}

pub(crate) struct SourceCacheInsert<'a> {
    pub(crate) url: &'a str,
    pub(crate) provider: &'a str,
    pub(crate) status: Option<u16>,
    pub(crate) content_hash: Option<&'a str>,
    pub(crate) route: Option<&'a str>,
    pub(crate) title: Option<&'a str>,
    pub(crate) canonical_url: Option<&'a str>,
    pub(crate) freshness_status: &'a str,
    pub(crate) privacy_classification: &'a str,
    pub(crate) raw_body_stored: bool,
    pub(crate) metadata: Value,
    pub(crate) redact_query_secrets: bool,
}

pub(crate) fn record_source_cache(
    paths: &ResearchPaths,
    source: SourceCacheInsert<'_>,
) -> Result<String> {
    let conn = Connection::open(&paths.database)?;
    let content_hash = source.content_hash.unwrap_or("");
    let url = if source.redact_query_secrets {
        redact_url_query_secrets(source.url)
    } else {
        source.url.to_string()
    };
    let canonical_url = source.canonical_url.map(|url| {
        if source.redact_query_secrets {
            redact_url_query_secrets(url)
        } else {
            url.to_string()
        }
    });
    let metadata = if source.redact_query_secrets {
        redact_metadata_urls(source.metadata)
    } else {
        source.metadata
    };
    let id = short_hash(format!(
        "{}:{}:{}:{}",
        source.provider,
        url,
        content_hash,
        serde_json::to_string(&metadata)?
    ));
    conn.execute(
        "insert or replace into sources
         (id, url, provider, fetched_at, content_hash, status, route, metadata_json,
          canonical_url, title, freshness_status, privacy_classification, raw_body_stored)
         values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            id,
            url,
            source.provider,
            Utc::now().to_rfc3339(),
            source.content_hash,
            source.status,
            source.route,
            serde_json::to_string(&metadata)?,
            canonical_url,
            source.title,
            source.freshness_status,
            source.privacy_classification,
            if source.raw_body_stored { 1_i64 } else { 0_i64 }
        ],
    )?;
    Ok(id)
}

pub(crate) fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<()> {
    let mut stmt = conn.prepare(&format!("pragma table_info({table})"))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    if !columns.contains(column) {
        conn.execute(
            &format!("alter table {table} add column {column} {definition}"),
            [],
        )?;
    }
    Ok(())
}

pub(crate) fn cached_source(
    paths: &ResearchPaths,
    source_id: &str,
) -> Result<Option<SourceCacheRecord>> {
    let conn = Connection::open(&paths.database)?;
    let mut stmt = conn.prepare(
        "select id, provider, route, url, canonical_url, title, fetched_at,
                freshness_status, privacy_classification, status, content_hash,
                raw_body_stored, metadata_json
         from sources where id = ?1",
    )?;
    let mut rows = stmt.query(params![source_id])?;
    if let Some(row) = rows.next()? {
        return Ok(Some(source_from_row(row)?));
    }
    Ok(None)
}

pub(crate) fn list_cached_sources(
    paths: &ResearchPaths,
    provider: Option<&str>,
    limit: u32,
) -> Result<Vec<SourceCacheRecord>> {
    let conn = Connection::open(&paths.database)?;
    let sql = if provider.is_some() {
        "select id, provider, route, url, canonical_url, title, fetched_at,
                freshness_status, privacy_classification, status, content_hash,
                raw_body_stored, metadata_json
         from sources where provider = ?1 order by fetched_at desc limit ?2"
            .to_string()
    } else {
        "select id, provider, route, url, canonical_url, title, fetched_at,
                freshness_status, privacy_classification, status, content_hash,
                raw_body_stored, metadata_json
         from sources order by fetched_at desc limit ?1"
            .to_string()
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = if let Some(provider) = provider {
        stmt.query_map(params![provider, limit], source_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?
    } else {
        stmt.query_map(params![limit], source_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?
    };
    Ok(rows)
}

pub(crate) fn source_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SourceCacheRecord> {
    let metadata_json: String = row.get(12)?;
    let metadata = serde_json::from_str(&metadata_json).unwrap_or_else(|_| json!({}));
    let status: Option<i64> = row.get(9)?;
    Ok(SourceCacheRecord {
        id: row.get(0)?,
        provider: row.get(1)?,
        route: row.get(2)?,
        url: row.get(3)?,
        canonical_url: row.get(4)?,
        title: row.get(5)?,
        fetched_at: row.get(6)?,
        freshness_status: row.get(7)?,
        privacy_classification: row.get(8)?,
        status: status.map(|value| value as u16),
        content_hash: row.get(10)?,
        raw_body_stored: row.get::<_, i64>(11)? != 0,
        metadata,
    })
}

pub(crate) fn record_route_memory(
    paths: &ResearchPaths,
    url: &str,
    route: Route,
    success: bool,
    status: Option<u16>,
    reason: &str,
) -> Result<()> {
    let Some(domain) = url_domain(url) else {
        return Ok(());
    };
    let conn = Connection::open(&paths.database)?;
    let route = route_name(route);
    conn.execute(
        "
        insert into route_memory
          (domain, preferred_route, successes, failures, updated_at, last_reason, last_status)
        values (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        on conflict(domain) do update set
          preferred_route = case
            when excluded.successes > 0 then excluded.preferred_route
            else route_memory.preferred_route
          end,
          successes = route_memory.successes + excluded.successes,
          failures = route_memory.failures + excluded.failures,
          updated_at = excluded.updated_at,
          last_reason = excluded.last_reason,
          last_status = excluded.last_status
        ",
        params![
            domain,
            route,
            if success { 1_i64 } else { 0_i64 },
            if success { 0_i64 } else { 1_i64 },
            Utc::now().to_rfc3339(),
            reason,
            status,
        ],
    )?;
    Ok(())
}

pub(crate) fn route_memory_for_url(
    paths: &ResearchPaths,
    url: &str,
) -> Result<Option<RouteMemoryHit>> {
    let Some(domain) = url_domain(url) else {
        return Ok(None);
    };
    let rows = list_route_memory(paths, Some(&domain))?;
    Ok(rows.into_iter().next())
}

pub(crate) fn list_route_memory(
    paths: &ResearchPaths,
    domain: Option<&str>,
) -> Result<Vec<RouteMemoryHit>> {
    let conn = Connection::open(&paths.database)?;
    if let Some(domain) = domain {
        let mut stmt = conn.prepare(
            "select domain, preferred_route, successes, failures, updated_at
             from route_memory where domain = ?1 order by updated_at desc",
        )?;
        return Ok(stmt
            .query_map(params![domain], route_memory_hit_from_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?);
    }
    let mut stmt = conn.prepare(
        "select domain, preferred_route, successes, failures, updated_at
         from route_memory order by updated_at desc",
    )?;
    Ok(stmt
        .query_map([], route_memory_hit_from_row)?
        .collect::<std::result::Result<Vec<_>, _>>()?)
}

pub(crate) fn route_memory_hit_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<RouteMemoryHit> {
    Ok(RouteMemoryHit {
        domain: row.get(0)?,
        preferred_route: row.get(1)?,
        successes: row.get::<_, i64>(2)? as u32,
        failures: row.get::<_, i64>(3)? as u32,
        updated_at: row.get(4)?,
    })
}

pub(crate) fn prune_cache(
    paths: &ResearchPaths,
    older_than_days: i64,
    dry_run: bool,
) -> Result<i64> {
    let cutoff = Utc::now() - chrono::Duration::days(older_than_days);
    let conn = Connection::open(&paths.database)?;
    let count: i64 = conn.query_row(
        "select count(*) from sources where fetched_at < ?1",
        params![cutoff.to_rfc3339()],
        |row| row.get(0),
    )?;
    if !dry_run {
        conn.execute(
            "delete from sources where fetched_at < ?1",
            params![cutoff.to_rfc3339()],
        )?;
    }
    Ok(count)
}

pub(crate) fn count_blobs(dir: &Path) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for shard in fs::read_dir(dir)? {
        let shard = shard?;
        if shard.file_type()?.is_dir() {
            count += fs::read_dir(shard.path())?
                .filter_map(std::result::Result::ok)
                .count();
        }
    }
    Ok(count)
}
