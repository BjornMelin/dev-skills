use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use directories::BaseDirs;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, RANGE, USER_AGENT};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use url::Url;

const GITHUB_API_VERSION: &str = "2026-03-10";
const USER_AGENT_VALUE: &str = "codex-research/0.1";

#[derive(Parser)]
#[command(name = "codex-research")]
#[command(about = "Evidence-first research helper for Codex skills and subagents")]
struct Cli {
    #[arg(
        long,
        global = true,
        help = "Emit machine-readable JSON when supported"
    )]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect local provider auth, external tools, and cache paths.
    Doctor,
    /// Produce a provider-aware research plan for a query.
    Plan(PlanArgs),
    /// Produce a search routing plan. Codex-native web tools are represented as instructions.
    Search(SearchArgs),
    /// Probe or fetch web pages through the predictive router.
    Fetch {
        #[command(subcommand)]
        command: FetchCommand,
    },
    /// Query Context7 directly through its REST API.
    Context7 {
        #[command(subcommand)]
        command: Context7Command,
    },
    /// Query GitHub through REST API or local gh authentication.
    Github {
        #[command(subcommand)]
        command: GithubCommand,
    },
    /// Manage JSONL claim ledgers.
    Ledger {
        #[command(subcommand)]
        command: LedgerCommand,
    },
    /// Render a Markdown report from a ledger.
    Report(ReportArgs),
    /// Initialize or inspect global cache state.
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },
    /// Run deterministic offline checks and optional live-provider smoke checks.
    Eval(EvalArgs),
}

#[derive(Args)]
struct PlanArgs {
    query: String,
    #[arg(long, value_enum, default_value_t = ResearchProfile::Standard)]
    profile: ResearchProfile,
}

#[derive(Args)]
struct SearchArgs {
    query: String,
    #[arg(long, value_enum, default_value_t = ResearchProfile::Standard)]
    profile: ResearchProfile,
    #[arg(long, value_enum, default_value_t = TopicKind::General)]
    topic: TopicKind,
}

#[derive(Subcommand)]
enum FetchCommand {
    /// Classify a URL and recommend direct/browser/Firecrawl routing.
    Probe {
        url: String,
        #[arg(long, default_value_t = 65_536)]
        max_bytes: usize,
    },
    /// Fetch a URL with direct HTTP and optionally store it in the content-addressed cache.
    Get {
        url: String,
        #[arg(long, default_value_t = 512_000)]
        max_bytes: usize,
        #[arg(long)]
        store: bool,
    },
    /// Scrape a URL through Firecrawl v2.
    Firecrawl {
        url: String,
        #[arg(long)]
        fresh: bool,
        #[arg(long, default_value_t = true)]
        store_in_cache: bool,
        #[arg(long, default_value_t = 60_000)]
        timeout_ms: u64,
    },
}

#[derive(Subcommand)]
enum Context7Command {
    /// Find a Context7 library ID.
    Search {
        #[arg(long)]
        library: String,
        #[arg(long)]
        query: String,
    },
    /// Retrieve documentation context for a library ID.
    Context {
        #[arg(long)]
        library_id: String,
        #[arg(long)]
        query: String,
        #[arg(long)]
        fast: bool,
    },
    /// Trigger a Context7 refresh.
    Refresh {
        #[arg(long)]
        library_name: String,
        #[arg(long)]
        branch: Option<String>,
    },
}

#[derive(Subcommand)]
enum GithubCommand {
    /// Search repositories.
    SearchRepos {
        query: String,
        #[arg(long, default_value_t = 10)]
        per_page: u8,
    },
    /// Search code. This endpoint has strict limits; use narrow queries.
    SearchCode {
        query: String,
        #[arg(long, default_value_t = 10)]
        per_page: u8,
    },
    /// Search issues and pull requests.
    SearchIssues {
        query: String,
        #[arg(long, default_value_t = 10)]
        per_page: u8,
    },
    /// List repository releases.
    Releases {
        repo: String,
        #[arg(long, default_value_t = 10)]
        per_page: u8,
    },
    /// Fetch one repository file through the contents API.
    File {
        repo: String,
        path: String,
        #[arg(long, default_value = "HEAD")]
        r#ref: String,
    },
}

#[derive(Subcommand)]
enum LedgerCommand {
    /// Create an empty ledger file if absent.
    Init {
        #[arg(long, default_value = ".codex/research/ledger.jsonl")]
        path: PathBuf,
    },
    /// Append a source record.
    AddSource(AddSourceArgs),
    /// Append a claim record.
    AddClaim(AddClaimArgs),
    /// Summarize ledger counts and IDs.
    Inspect {
        #[arg(long, default_value = ".codex/research/ledger.jsonl")]
        path: PathBuf,
    },
}

#[derive(Args)]
struct AddSourceArgs {
    #[arg(long, default_value = ".codex/research/ledger.jsonl")]
    ledger: PathBuf,
    #[arg(long)]
    provider: String,
    #[arg(long)]
    url: String,
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    route: Option<String>,
}

#[derive(Args)]
struct AddClaimArgs {
    #[arg(long, default_value = ".codex/research/ledger.jsonl")]
    ledger: PathBuf,
    #[arg(long)]
    text: String,
    #[arg(long, default_value_t = 0.75)]
    confidence: f32,
    #[arg(long = "source")]
    sources: Vec<String>,
    #[arg(long)]
    note: Option<String>,
}

#[derive(Args)]
struct ReportArgs {
    #[arg(long, default_value = ".codex/research/ledger.jsonl")]
    ledger: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Subcommand)]
enum CacheCommand {
    /// Create the SQLite database and blob directory.
    Init,
    /// Print cache table counts and paths.
    Stats,
}

#[derive(Args)]
struct EvalArgs {
    #[arg(long)]
    live: bool,
}

#[derive(Clone, Copy, Debug, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum ResearchProfile {
    Quick,
    Standard,
    Deep,
    Exhaustive,
}

impl std::fmt::Display for ResearchProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quick => write!(f, "quick"),
            Self::Standard => write!(f, "standard"),
            Self::Deep => write!(f, "deep"),
            Self::Exhaustive => write!(f, "exhaustive"),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum TopicKind {
    General,
    Docs,
    Github,
    Dependency,
    Openai,
    Rendered,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Route {
    CodexWeb,
    Context7,
    Github,
    Direct,
    AgentBrowser,
    Firecrawl,
    Exa,
    Opensrc,
}

#[derive(Serialize)]
struct DoctorReport {
    cache_dir: PathBuf,
    database: PathBuf,
    blobs_dir: PathBuf,
    env: BTreeMap<&'static str, bool>,
    tools: BTreeMap<&'static str, Option<String>>,
    notes: Vec<String>,
}

#[derive(Serialize)]
struct ResearchPlan {
    query: String,
    profile: ResearchProfile,
    budgets: ProviderBudgets,
    route_order: Vec<Route>,
    rules: Vec<String>,
}

#[derive(Serialize)]
struct ProviderBudgets {
    codex_web_queries: u8,
    context7_calls: u8,
    github_calls: u8,
    exa_calls: u8,
    direct_fetches: u8,
    browser_fetches: u8,
    firecrawl_calls: u8,
}

#[derive(Serialize)]
struct ProbeReport {
    url: String,
    status: Option<u16>,
    content_type: Option<String>,
    content_length: Option<u64>,
    text_chars: usize,
    script_markers: usize,
    app_shell_markers: Vec<String>,
    route: Route,
    reason: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
enum LedgerRecord {
    #[serde(rename = "source")]
    Source(SourceRecord),
    #[serde(rename = "claim")]
    Claim(ClaimRecord),
}

#[derive(Serialize, Deserialize)]
struct SourceRecord {
    id: String,
    provider: String,
    url: String,
    title: Option<String>,
    route: Option<String>,
    fetched_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct ClaimRecord {
    id: String,
    text: String,
    confidence: f32,
    sources: Vec<String>,
    note: Option<String>,
    created_at: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Doctor => doctor(cli.json),
        Commands::Plan(args) => output_plan(args, cli.json),
        Commands::Search(args) => output_search_plan(args, cli.json),
        Commands::Fetch { command } => handle_fetch(command, cli.json).await,
        Commands::Context7 { command } => handle_context7(command, cli.json).await,
        Commands::Github { command } => handle_github(command, cli.json).await,
        Commands::Ledger { command } => handle_ledger(command, cli.json),
        Commands::Report(args) => render_report(args, cli.json),
        Commands::Cache { command } => handle_cache(command, cli.json),
        Commands::Eval(args) => run_eval(args, cli.json).await,
    }
}

fn doctor(json_out: bool) -> Result<()> {
    let paths = research_paths()?;
    let mut env = BTreeMap::new();
    for key in [
        "CONTEXT7_API_KEY",
        "FIRECRAWL_API_KEY",
        "GITHUB_TOKEN",
        "GH_TOKEN",
        "EXA_API_KEY",
        "CODEX_RESEARCH_HOME",
    ] {
        env.insert(key, std::env::var_os(key).is_some());
    }

    let mut tools = BTreeMap::new();
    tools.insert("gh", command_version("gh", &["--version"]));
    tools.insert(
        "agent-browser",
        command_version("agent-browser", &["--version"]),
    );
    tools.insert("ctx7", command_version("ctx7", &["--version"]));
    tools.insert("opensrc", command_version("opensrc", &["--version"]));

    let notes = vec![
        "Codex-native web.search_query/open/find are session tools, not external CLI APIs.".to_string(),
        "Use Context7 REST API directly for library docs and refreshes.".to_string(),
        "Use Firecrawl only after classification or when the task explicitly needs rendered/crawl extraction.".to_string(),
    ];

    let report = DoctorReport {
        cache_dir: paths.cache_dir,
        database: paths.database,
        blobs_dir: paths.blobs_dir,
        env,
        tools,
        notes,
    };
    if json_out {
        print_json(&report)
    } else {
        println!("cache: {}", report.cache_dir.display());
        println!("database: {}", report.database.display());
        println!("blobs: {}", report.blobs_dir.display());
        println!("env:");
        for (key, present) in report.env {
            println!("  {key}: {}", if present { "present" } else { "missing" });
        }
        println!("tools:");
        for (name, version) in report.tools {
            let status = version
                .as_deref()
                .and_then(|v| v.lines().next())
                .unwrap_or("missing");
            println!("  {name}: {status}");
        }
        Ok(())
    }
}

fn output_plan(args: PlanArgs, json_out: bool) -> Result<()> {
    let plan = build_plan(&args.query, args.profile, TopicKind::General);
    if json_out {
        print_json(&plan)
    } else {
        println!("# Research Plan");
        println!();
        println!("query: {}", plan.query);
        println!("profile: {}", plan.profile);
        println!("route order: {}", route_list(&plan.route_order));
        println!("budgets:");
        println!("  codex web: {}", plan.budgets.codex_web_queries);
        println!("  context7: {}", plan.budgets.context7_calls);
        println!("  github: {}", plan.budgets.github_calls);
        println!("  exa: {}", plan.budgets.exa_calls);
        println!("  direct fetches: {}", plan.budgets.direct_fetches);
        println!("  browser fetches: {}", plan.budgets.browser_fetches);
        println!("  firecrawl: {}", plan.budgets.firecrawl_calls);
        println!("rules:");
        for rule in plan.rules {
            println!("- {rule}");
        }
        Ok(())
    }
}

fn output_search_plan(args: SearchArgs, json_out: bool) -> Result<()> {
    let plan = build_plan(&args.query, args.profile, args.topic);
    if json_out {
        print_json(&plan)
    } else {
        println!("Use these routes in order for `{}`:", args.query);
        for (idx, route) in plan.route_order.iter().enumerate() {
            println!("{}. {}", idx + 1, route_name(*route));
        }
        println!();
        println!("Codex web should handle narrow official/current checks first.");
        println!(
            "Exa is reserved for broad semantic discovery or filtered multi-source exploration."
        );
        Ok(())
    }
}

async fn handle_fetch(command: FetchCommand, json_out: bool) -> Result<()> {
    match command {
        FetchCommand::Probe { url, max_bytes } => {
            let client = http_client()?;
            let report = probe_url(&client, &url, max_bytes).await?;
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
        } => {
            let client = http_client()?;
            let fetched = direct_fetch(&client, &url, max_bytes).await?;
            if store {
                let paths = research_paths()?;
                init_db(&paths)?;
                let hash = store_blob(&paths, fetched.body.as_bytes())?;
                record_source_cache(&paths, &url, "direct", fetched.status, &hash, None)?;
            }
            if json_out {
                print_json(&fetched)
            } else {
                println!("{}", fetched.body);
                Ok(())
            }
        }
        FetchCommand::Firecrawl {
            url,
            fresh,
            store_in_cache,
            timeout_ms,
        } => {
            let value = firecrawl_scrape(&url, fresh, store_in_cache, timeout_ms).await?;
            print_json(&value)
        }
    }
}

async fn handle_context7(command: Context7Command, json_out: bool) -> Result<()> {
    let api_key = required_env("CONTEXT7_API_KEY")?;
    let client = http_client()?;
    let value = match command {
        Context7Command::Search { library, query } => {
            client
                .get("https://context7.com/api/v2/libs/search")
                .bearer_auth(api_key)
                .query(&[("libraryName", library), ("query", query)])
                .send()
                .await?
                .error_for_status()?
                .json::<Value>()
                .await?
        }
        Context7Command::Context {
            library_id,
            query,
            fast,
        } => {
            client
                .get("https://context7.com/api/v2/context")
                .bearer_auth(api_key)
                .query(&[
                    ("libraryId", library_id),
                    ("query", query),
                    ("type", "json".to_string()),
                    ("fast", fast.to_string()),
                ])
                .send()
                .await?
                .error_for_status()?
                .json::<Value>()
                .await?
        }
        Context7Command::Refresh {
            library_name,
            branch,
        } => {
            let mut body = json!({ "libraryName": library_name });
            if let Some(branch) = branch {
                body["branch"] = json!(branch);
            }
            client
                .post("https://context7.com/api/v1/refresh")
                .bearer_auth(api_key)
                .json(&body)
                .send()
                .await?
                .error_for_status()?
                .json::<Value>()
                .await?
        }
    };

    if json_out {
        print_json(&value)
    } else {
        println!("{}", serde_json::to_string_pretty(&value)?);
        Ok(())
    }
}

async fn handle_github(command: GithubCommand, json_out: bool) -> Result<()> {
    let client = http_client()?;
    let value = match command {
        GithubCommand::SearchRepos { query, per_page } => {
            github_get(
                &client,
                "https://api.github.com/search/repositories",
                &[("q", query), ("per_page", per_page.to_string())],
            )
            .await?
        }
        GithubCommand::SearchCode { query, per_page } => {
            github_get(
                &client,
                "https://api.github.com/search/code",
                &[("q", query), ("per_page", per_page.to_string())],
            )
            .await?
        }
        GithubCommand::SearchIssues { query, per_page } => {
            github_get(
                &client,
                "https://api.github.com/search/issues",
                &[("q", query), ("per_page", per_page.to_string())],
            )
            .await?
        }
        GithubCommand::Releases { repo, per_page } => {
            let url = format!("https://api.github.com/repos/{repo}/releases");
            github_get(&client, &url, &[("per_page", per_page.to_string())]).await?
        }
        GithubCommand::File { repo, path, r#ref } => {
            let url = format!("https://api.github.com/repos/{repo}/contents/{path}");
            github_get(&client, &url, &[("ref", r#ref)]).await?
        }
    };

    if json_out {
        print_json(&value)
    } else {
        println!("{}", serde_json::to_string_pretty(&value)?);
        Ok(())
    }
}

fn handle_ledger(command: LedgerCommand, json_out: bool) -> Result<()> {
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
            let id = short_hash(format!("{}:{}:{}", args.provider, args.url, Utc::now()));
            let record = LedgerRecord::Source(SourceRecord {
                id: id.clone(),
                provider: args.provider,
                url: args.url,
                title: args.title,
                route: args.route,
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

fn render_report(args: ReportArgs, json_out: bool) -> Result<()> {
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

fn handle_cache(command: CacheCommand, json_out: bool) -> Result<()> {
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
    }
}

async fn run_eval(args: EvalArgs, json_out: bool) -> Result<()> {
    let client = http_client()?;
    let samples = vec![
        (
            "https://docs.example.com",
            "<html><main><h1>Docs</h1><p>Use this API for reliable output with examples and guides.</p></main></html>",
            Route::Direct,
        ),
        (
            "https://app.example.com",
            "<html><body><div id=\"__next\"></div><script src=\"/_next/static/app.js\"></script></body></html>",
            Route::AgentBrowser,
        ),
        (
            "https://github.com/org/repo/blob/main/README.md",
            "github",
            Route::Github,
        ),
    ];
    let mut passed = 0;
    let mut failures = Vec::new();
    for (url, body, expected) in samples {
        let report = classify_body(url, Some("text/html"), None, body);
        if route_name(report.route) == route_name(expected) {
            passed += 1;
        } else {
            failures.push(json!({
                "url": url,
                "expected": route_name(expected),
                "actual": route_name(report.route)
            }));
        }
    }

    let mut live = Vec::new();
    if args.live {
        if std::env::var_os("CONTEXT7_API_KEY").is_some() {
            live.push(json!({ "provider": "context7", "status": "configured" }));
        }
        if std::env::var_os("FIRECRAWL_API_KEY").is_some() {
            live.push(json!({ "provider": "firecrawl", "status": "configured" }));
        }
        let github = github_token().is_some();
        live.push(json!({ "provider": "github", "status": if github { "configured" } else { "public-only" } }));
        let _ = client;
    }

    let failed = !failures.is_empty();
    let result = json!({
        "offline": {
            "passed": passed,
            "failed": failures.len(),
            "failures": failures
        },
        "live": live
    });
    if json_out {
        print_json(&result)
    } else {
        println!("offline passed: {passed}");
        if failed {
            println!(
                "{}",
                serde_json::to_string_pretty(&result["offline"]["failures"])?
            );
            bail!("offline eval failures");
        }
        if args.live {
            println!("{}", serde_json::to_string_pretty(&result["live"])?);
        }
        Ok(())
    }
}

fn build_plan(query: &str, profile: ResearchProfile, topic: TopicKind) -> ResearchPlan {
    let budgets = match profile {
        ResearchProfile::Quick => ProviderBudgets {
            codex_web_queries: 2,
            context7_calls: 1,
            github_calls: 1,
            exa_calls: 0,
            direct_fetches: 2,
            browser_fetches: 0,
            firecrawl_calls: 0,
        },
        ResearchProfile::Standard => ProviderBudgets {
            codex_web_queries: 4,
            context7_calls: 2,
            github_calls: 4,
            exa_calls: 2,
            direct_fetches: 6,
            browser_fetches: 2,
            firecrawl_calls: 2,
        },
        ResearchProfile::Deep => ProviderBudgets {
            codex_web_queries: 8,
            context7_calls: 4,
            github_calls: 8,
            exa_calls: 4,
            direct_fetches: 12,
            browser_fetches: 4,
            firecrawl_calls: 6,
        },
        ResearchProfile::Exhaustive => ProviderBudgets {
            codex_web_queries: 12,
            context7_calls: 8,
            github_calls: 16,
            exa_calls: 8,
            direct_fetches: 24,
            browser_fetches: 8,
            firecrawl_calls: 12,
        },
    };

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

async fn probe_url(client: &reqwest::Client, url: &str, max_bytes: usize) -> Result<ProbeReport> {
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
            Ok(report)
        }
        Err(_) => {
            let mut report = classify_body(url, content_type.as_deref(), content_length, "");
            report.status = status;
            Ok(report)
        }
    }
}

#[derive(Serialize)]
struct FetchedBody {
    url: String,
    status: u16,
    content_type: Option<String>,
    bytes: usize,
    body: String,
}

async fn direct_fetch(
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
    Ok(FetchedBody {
        url: url.to_string(),
        status,
        content_type,
        bytes: slice.len(),
        body,
    })
}

fn classify_body(
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
    }
}

async fn firecrawl_scrape(
    url: &str,
    fresh: bool,
    store_in_cache: bool,
    timeout_ms: u64,
) -> Result<Value> {
    let api_key = required_env("FIRECRAWL_API_KEY")?;
    let client = http_client()?;
    let max_age = if fresh { 0 } else { 86_400_000 };
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
    Ok(resp.error_for_status()?.json::<Value>().await?)
}

async fn github_get(
    client: &reqwest::Client,
    url: &str,
    params: &[(&str, String)],
) -> Result<Value> {
    let mut req = client.get(url);
    if let Some(token) = github_token() {
        req = req.bearer_auth(token);
    }
    let resp = req
        .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
        .query(params)
        .send()
        .await?;
    if resp.status().as_u16() == 403 {
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
    Ok(resp.error_for_status()?.json::<Value>().await?)
}

fn http_client() -> Result<reqwest::Client> {
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

fn github_token() -> Option<String> {
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

fn command_version(command: &str, args: &[&str]) -> Option<String> {
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

struct ResearchPaths {
    cache_dir: PathBuf,
    database: PathBuf,
    blobs_dir: PathBuf,
}

fn research_paths() -> Result<ResearchPaths> {
    let cache_dir = if let Some(value) = std::env::var_os("CODEX_RESEARCH_HOME") {
        PathBuf::from(value)
    } else {
        BaseDirs::new()
            .context("could not determine base directories")?
            .cache_dir()
            .join("codex-research")
    };
    Ok(ResearchPaths {
        database: cache_dir.join("research.sqlite"),
        blobs_dir: cache_dir.join("blobs"),
        cache_dir,
    })
}

fn init_db(paths: &ResearchPaths) -> Result<()> {
    fs::create_dir_all(&paths.cache_dir)?;
    fs::create_dir_all(&paths.blobs_dir)?;
    let conn = Connection::open(&paths.database)?;
    conn.execute_batch(
        "
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
    Ok(())
}

fn store_blob(paths: &ResearchPaths, bytes: &[u8]) -> Result<String> {
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

fn record_source_cache(
    paths: &ResearchPaths,
    url: &str,
    provider: &str,
    status: u16,
    content_hash: &str,
    route: Option<&str>,
) -> Result<()> {
    let conn = Connection::open(&paths.database)?;
    let id = short_hash(format!("{provider}:{url}:{content_hash}"));
    conn.execute(
        "insert or replace into sources
         (id, url, provider, fetched_at, content_hash, status, route, metadata_json)
         values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            url,
            provider,
            Utc::now().to_rfc3339(),
            content_hash,
            status,
            route,
            "{}"
        ],
    )?;
    Ok(())
}

fn count_blobs(dir: &Path) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for shard in fs::read_dir(dir)? {
        let shard = shard?;
        if shard.file_type()?.is_dir() {
            count += fs::read_dir(shard.path())?.filter_map(Result::ok).count();
        }
    }
    Ok(count)
}

fn append_ledger_record(path: &Path, record: &LedgerRecord) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, record)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn read_ledger_records(path: &Path) -> Result<Vec<LedgerRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(serde_json::from_str(&line)?);
    }
    Ok(records)
}

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn strip_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

fn is_github_url(value: &str) -> bool {
    Url::parse(value)
        .ok()
        .and_then(|url| url.host_str().map(|h| h.eq_ignore_ascii_case("github.com")))
        .unwrap_or(false)
}

fn short_hash(value: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    format!("{:x}", hasher.finalize())[..12].to_string()
}

fn required_env(name: &str) -> Result<String> {
    let value = std::env::var(name).with_context(|| format!("{name} is required"))?;
    if value.trim().is_empty() {
        bail!("{name} is empty");
    }
    Ok(value)
}

fn route_name(route: Route) -> &'static str {
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

fn route_list(routes: &[Route]) -> String {
    routes
        .iter()
        .map(|r| route_name(*r))
        .collect::<Vec<_>>()
        .join(", ")
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_plan_prefers_docs_source_and_github_routes() {
        let plan = build_plan(
            "verify dependency behavior",
            ResearchProfile::Deep,
            TopicKind::Dependency,
        );

        assert_eq!(plan.profile.to_string(), "deep");
        assert_eq!(plan.budgets.context7_calls, 4);
        assert_eq!(plan.budgets.github_calls, 8);
        assert_eq!(plan.route_order[0], Route::Context7);
        assert_eq!(plan.route_order[1], Route::Opensrc);
        assert!(plan.route_order.contains(&Route::Github));
    }

    #[test]
    fn github_urls_route_to_github_api_hydration() {
        let report = classify_body(
            "https://github.com/openai/codex/blob/main/README.md",
            Some("text/html"),
            None,
            "<html>large rendered github page</html>",
        );

        assert_eq!(report.route, Route::Github);
        assert!(report.reason.contains("GitHub APIs"));
    }

    #[test]
    fn app_shell_with_low_text_routes_to_browser() {
        let report = classify_body(
            "https://docs.example.com/app",
            Some("text/html"),
            None,
            r#"<html><body><div id="__next"></div><script src="/app.js"></script></body></html>"#,
        );

        assert_eq!(report.route, Route::AgentBrowser);
        assert!(
            report
                .app_shell_markers
                .iter()
                .any(|marker| marker == "id=\"__next\"")
        );
    }

    #[test]
    fn text_like_content_routes_to_direct_fetch() {
        let report = classify_body(
            "https://example.com/llms.txt",
            Some("text/plain"),
            Some(256),
            "plain text docs",
        );

        assert_eq!(report.route, Route::Direct);
    }

    #[test]
    fn ledger_records_round_trip() -> Result<()> {
        let dir = std::env::temp_dir().join(format!(
            "codex-research-ledger-test-{}",
            short_hash(format!("{}-{}", std::process::id(), Utc::now()))
        ));
        fs::create_dir_all(&dir)?;
        let ledger = dir.join("ledger.jsonl");

        let source = LedgerRecord::Source(SourceRecord {
            id: "src123".to_string(),
            provider: "github".to_string(),
            url: "https://github.com/openai/codex".to_string(),
            title: Some("openai/codex".to_string()),
            route: Some("github".to_string()),
            fetched_at: Utc::now(),
        });
        let claim = LedgerRecord::Claim(ClaimRecord {
            id: "claim123".to_string(),
            text: "GitHub source hydration works.".to_string(),
            confidence: 0.9,
            sources: vec!["src123".to_string()],
            note: None,
            created_at: Utc::now(),
        });

        append_ledger_record(&ledger, &source)?;
        append_ledger_record(&ledger, &claim)?;
        let records = read_ledger_records(&ledger)?;
        fs::remove_dir_all(&dir)?;

        assert_eq!(records.len(), 2);
        assert!(matches!(records[0], LedgerRecord::Source(_)));
        assert!(matches!(records[1], LedgerRecord::Claim(_)));
        Ok(())
    }
}
