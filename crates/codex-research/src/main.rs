use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::future::Future;
use std::io::ErrorKind;
use std::io::{BufRead, BufReader, Write};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use directories::BaseDirs;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, LINK, RANGE, USER_AGENT};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use url::Url;

const GITHUB_API_VERSION: &str = "2026-03-10";
const USER_AGENT_VALUE: &str = "codex-research/0.2";
const DEFAULT_EVAL_SUITE: &str = include_str!("../evals/research/core.json");

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

    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Load codex-research TOML config from an explicit path"
    )]
    config: Option<PathBuf>,

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
    /// Manage codex-research TOML configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Manage per-research-run budgets and state.
    Run {
        #[command(subcommand)]
        command: RunCommand,
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

#[derive(Args, Clone, Debug, Default)]
struct BudgetArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Debit this research run before calling a provider"
    )]
    run: Option<PathBuf>,
    #[arg(long, help = "Skip run-budget debit even when --run is provided")]
    no_budget: bool,
}

#[derive(Subcommand)]
enum FetchCommand {
    /// Classify a URL and recommend direct/browser/Firecrawl routing.
    Probe {
        url: String,
        #[arg(long, default_value_t = 65_536)]
        max_bytes: usize,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// Fetch a URL with direct HTTP and optionally store it in the content-addressed cache.
    Get {
        url: String,
        #[arg(long, default_value_t = 512_000)]
        max_bytes: usize,
        #[arg(long)]
        store: bool,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// Scrape a URL through Firecrawl v2.
    Firecrawl {
        url: String,
        #[arg(long)]
        fresh: bool,
        #[arg(
            long = "no-store-in-cache",
            help = "Disable Firecrawl server-side cache storage for this request"
        )]
        no_store_in_cache: bool,
        #[arg(long, default_value_t = 60_000)]
        timeout_ms: u64,
        #[arg(long, value_enum)]
        privacy: Option<PrivacyClass>,
        #[arg(long)]
        allow_private_external: bool,
        #[command(flatten)]
        budget: BudgetArgs,
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
        #[arg(long)]
        version: Option<String>,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// Retrieve documentation context for a library ID.
    Context {
        #[arg(long)]
        library_id: String,
        #[arg(long)]
        query: String,
        #[arg(long)]
        fast: bool,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// Trigger a Context7 refresh.
    Refresh {
        #[arg(long)]
        library_name: String,
        #[arg(long)]
        branch: Option<String>,
        #[command(flatten)]
        budget: BudgetArgs,
    },
}

#[derive(Subcommand)]
enum GithubCommand {
    /// Search repositories.
    SearchRepos {
        query: String,
        #[arg(long)]
        per_page: Option<u8>,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// Search code. This endpoint has strict limits; use narrow queries.
    SearchCode {
        query: String,
        #[arg(long)]
        per_page: Option<u8>,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// Search issues and pull requests.
    SearchIssues {
        query: String,
        #[arg(long)]
        per_page: Option<u8>,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// List repository releases.
    Releases {
        repo: String,
        #[arg(long)]
        per_page: Option<u8>,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// Fetch one release by tag or the latest release.
    Release {
        repo: String,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        latest: bool,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// Compare two refs and include changed-file metadata.
    Compare {
        repo: String,
        base: String,
        head: String,
        #[arg(long)]
        per_page: Option<u8>,
        #[arg(long, default_value_t = 1)]
        page: u32,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// List repository tags.
    Tags {
        repo: String,
        #[arg(long)]
        per_page: Option<u8>,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// Hydrate one issue and optionally comments.
    Issue {
        repo: String,
        number: u32,
        #[arg(long)]
        comments: bool,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// Hydrate one pull request and optional files, comments, and reviews.
    Pr {
        repo: String,
        number: u32,
        #[arg(long)]
        files: bool,
        #[arg(long)]
        comments: bool,
        #[arg(long)]
        reviews: bool,
        #[command(flatten)]
        budget: BudgetArgs,
    },
    /// Fetch one repository file through the contents API.
    File {
        repo: String,
        path: String,
        #[arg(long, default_value = "HEAD")]
        r#ref: String,
        #[command(flatten)]
        budget: BudgetArgs,
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
    #[arg(long = "from-cache", value_name = "SOURCE_ID")]
    from_cache: Option<String>,
    #[arg(long)]
    provider: Option<String>,
    #[arg(long)]
    url: Option<String>,
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
    /// List cached source metadata.
    Sources {
        #[arg(long)]
        provider: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Show one cached source.
    Source { source_id: String },
    /// Inspect route memory.
    RouteMemory {
        #[arg(long)]
        domain: Option<String>,
    },
    /// Delete old cache rows while preserving blobs.
    Prune {
        #[arg(long = "older-than-days")]
        older_than_days: i64,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Write a starter config file.
    Init {
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
    /// Show the effective config.
    Show,
}

#[derive(Subcommand)]
enum RunCommand {
    /// Initialize a research run state file.
    Init {
        query: String,
        #[arg(long, value_enum, default_value_t = ResearchProfile::Standard)]
        profile: ResearchProfile,
        #[arg(long, value_enum, default_value_t = TopicKind::General)]
        topic: TopicKind,
        #[arg(long, value_name = "PATH", default_value = ".codex/research/run.json")]
        out: PathBuf,
    },
    /// Show run status.
    Status {
        #[arg(long, value_name = "PATH", default_value = ".codex/research/run.json")]
        run: PathBuf,
    },
    /// Debit provider budget from a run state file.
    Debit {
        #[arg(long, value_name = "PATH", default_value = ".codex/research/run.json")]
        run: PathBuf,
        #[arg(long, value_enum)]
        provider: ProviderKind,
        #[arg(long, default_value_t = 1)]
        count: u32,
        #[arg(long)]
        note: Option<String>,
    },
    /// Mark a run closed.
    Close {
        #[arg(long, value_name = "PATH", default_value = ".codex/research/run.json")]
        run: PathBuf,
    },
}

#[derive(Args)]
struct EvalArgs {
    #[arg(long)]
    live: bool,
    #[arg(long, value_name = "PATH", help = "Load an eval suite JSON file")]
    suite: Option<PathBuf>,
    #[arg(long, value_name = "ID", help = "Run only the selected task ID")]
    task: Vec<String>,
    #[arg(long, help = "List eval tasks without running them")]
    list: bool,
    #[arg(long, help = "Treat eval warnings as failures")]
    strict: bool,
}

#[derive(Debug, Deserialize)]
struct EvalSuite {
    suite: String,
    description: Option<String>,
    tasks: Vec<EvalTask>,
}

#[derive(Debug, Deserialize)]
struct EvalTask {
    id: String,
    kind: String,
    description: String,
    input: Value,
    expected: Value,
}

#[derive(Default)]
struct EvalAssertions {
    failures: Vec<String>,
    warnings: Vec<String>,
    details: BTreeMap<String, Value>,
}

#[derive(Serialize)]
struct EvalTaskSummary {
    id: String,
    kind: String,
    description: String,
}

#[derive(Serialize)]
struct EvalTaskOutcome {
    id: String,
    kind: String,
    description: String,
    status: String,
    failures: Vec<String>,
    warnings: Vec<String>,
    details: BTreeMap<String, Value>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, ValueEnum)]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, ValueEnum)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum ProviderKind {
    CodexWeb,
    Context7,
    Github,
    Exa,
    Direct,
    Browser,
    Firecrawl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum PrivacyClass {
    Public,
    SensitivePublic,
    PrivateOrAuthenticated,
    Ambiguous,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct ProviderBudgets {
    codex_web_queries: u32,
    context7_calls: u32,
    github_calls: u32,
    exa_calls: u32,
    direct_fetches: u32,
    browser_fetches: u32,
    firecrawl_calls: u32,
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
    route_memory: Vec<RouteMemoryHit>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RouteMemoryHit {
    domain: String,
    preferred_route: String,
    successes: u32,
    failures: u32,
    updated_at: String,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct ResearchConfig {
    #[serde(default)]
    profiles: ProfilesConfig,
    #[serde(default)]
    privacy: PrivacyConfig,
    #[serde(default)]
    providers: ProvidersConfig,
    #[serde(default)]
    cache: CacheConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProfilesConfig {
    #[serde(default = "quick_budget")]
    quick: ProviderBudgets,
    #[serde(default = "standard_budget")]
    standard: ProviderBudgets,
    #[serde(default = "deep_budget")]
    deep: ProviderBudgets,
    #[serde(default = "exhaustive_budget")]
    exhaustive: ProviderBudgets,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PrivacyConfig {
    #[serde(default = "deny_string")]
    private_external_default: String,
    #[serde(default = "deny_string")]
    ambiguous_external_default: String,
    #[serde(default)]
    allow_private_external: bool,
    #[serde(default = "default_true")]
    redact_query_secrets: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct ProvidersConfig {
    #[serde(default)]
    github: GithubProviderConfig,
    #[serde(default)]
    context7: Context7ProviderConfig,
    #[serde(default)]
    firecrawl: FirecrawlProviderConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GithubProviderConfig {
    #[serde(default = "default_github_per_page")]
    per_page_default: u8,
    #[serde(default = "default_github_per_page_max")]
    per_page_max: u8,
    #[serde(default = "default_backoff_retries")]
    backoff_retries: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Context7ProviderConfig {
    #[serde(default = "default_cache_ttl_hours")]
    cache_ttl_hours: u32,
    #[serde(default = "default_true")]
    prefer_version_pinned_ids: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FirecrawlProviderConfig {
    #[serde(default = "default_firecrawl_max_age_ms")]
    default_max_age_ms: u64,
    #[serde(default)]
    latest_critical_max_age_ms: u64,
    #[serde(default = "default_true")]
    store_in_cache_default: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CacheConfig {
    #[serde(default = "default_cache_ttl_hours")]
    source_metadata_ttl_hours: u32,
    #[serde(default)]
    store_raw_external_default: bool,
}

impl Default for ProfilesConfig {
    fn default() -> Self {
        Self {
            quick: quick_budget(),
            standard: standard_budget(),
            deep: deep_budget(),
            exhaustive: exhaustive_budget(),
        }
    }
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            private_external_default: deny_string(),
            ambiguous_external_default: deny_string(),
            allow_private_external: false,
            redact_query_secrets: true,
        }
    }
}

impl Default for GithubProviderConfig {
    fn default() -> Self {
        Self {
            per_page_default: default_github_per_page(),
            per_page_max: default_github_per_page_max(),
            backoff_retries: default_backoff_retries(),
        }
    }
}

impl Default for Context7ProviderConfig {
    fn default() -> Self {
        Self {
            cache_ttl_hours: default_cache_ttl_hours(),
            prefer_version_pinned_ids: true,
        }
    }
}

impl Default for FirecrawlProviderConfig {
    fn default() -> Self {
        Self {
            default_max_age_ms: default_firecrawl_max_age_ms(),
            latest_critical_max_age_ms: 0,
            store_in_cache_default: true,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            source_metadata_ttl_hours: default_cache_ttl_hours(),
            store_raw_external_default: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ResearchRunState {
    query: String,
    profile: ResearchProfile,
    topic: TopicKind,
    status: RunStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    budgets: ProviderBudgets,
    spent: ProviderBudgets,
    debits: Vec<RunDebit>,
    provider_errors: Vec<ProviderError>,
    source_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum RunStatus {
    Open,
    Closed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RunDebit {
    provider: ProviderKind,
    count: u32,
    note: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProviderError {
    provider: ProviderKind,
    message: String,
    created_at: DateTime<Utc>,
}

struct FirecrawlScrape {
    status: u16,
    value: Value,
}

struct GithubResponse {
    value: Value,
    rate_limit: Value,
    pagination: Value,
}

#[derive(Clone, Debug, Serialize)]
struct ConfigReport {
    path: Option<PathBuf>,
    config: ResearchConfig,
}

#[derive(Clone, Debug, Serialize)]
struct SourceCacheRecord {
    id: String,
    provider: String,
    route: Option<String>,
    url: String,
    canonical_url: Option<String>,
    title: Option<String>,
    fetched_at: String,
    freshness_status: String,
    privacy_classification: String,
    status: Option<u16>,
    content_hash: Option<String>,
    raw_body_stored: bool,
    metadata: Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let loaded_config = load_config(cli.config.as_deref())?;
    let config = loaded_config.config;
    match cli.command {
        Commands::Doctor => doctor(cli.json),
        Commands::Plan(args) => output_plan(args, &config, cli.json),
        Commands::Search(args) => output_search_plan(args, &config, cli.json),
        Commands::Fetch { command } => handle_fetch(command, &config, cli.json).await,
        Commands::Context7 { command } => handle_context7(command, &config, cli.json).await,
        Commands::Github { command } => handle_github(command, &config, cli.json).await,
        Commands::Ledger { command } => handle_ledger(command, cli.json),
        Commands::Report(args) => render_report(args, cli.json),
        Commands::Cache { command } => handle_cache(command, cli.json),
        Commands::Config { command } => handle_config(command, loaded_config.path, cli.json),
        Commands::Run { command } => handle_run(command, &config, cli.json),
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

fn output_plan(args: PlanArgs, config: &ResearchConfig, json_out: bool) -> Result<()> {
    let plan = build_plan(&args.query, args.profile, TopicKind::General, config);
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

fn output_search_plan(args: SearchArgs, config: &ResearchConfig, json_out: bool) -> Result<()> {
    let plan = build_plan(&args.query, args.profile, args.topic, config);
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

async fn handle_fetch(
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

async fn handle_context7(
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

async fn handle_github(
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

fn handle_config(
    command: ConfigCommand,
    loaded_path: Option<PathBuf>,
    json_out: bool,
) -> Result<()> {
    match command {
        ConfigCommand::Init { path, force } => {
            let path = path.unwrap_or_else(|| PathBuf::from(".codex/research/config.toml"));
            if path.exists() && !force {
                bail!(
                    "config exists; pass --force to overwrite: {}",
                    path.display()
                );
            }
            ensure_parent(&path)?;
            fs::write(&path, default_config_toml()?.as_bytes())?;
            if json_out {
                print_json(&json!({ "path": path, "written": true }))
            } else {
                println!("config: {}", path.display());
                Ok(())
            }
        }
        ConfigCommand::Show => {
            let loaded = load_config(loaded_path.as_deref())?;
            let report = ConfigReport {
                path: loaded.path,
                config: loaded.config,
            };
            if json_out {
                print_json(&report)
            } else {
                println!("{}", toml::to_string_pretty(&report.config)?);
                Ok(())
            }
        }
    }
}

fn handle_run(command: RunCommand, config: &ResearchConfig, json_out: bool) -> Result<()> {
    match command {
        RunCommand::Init {
            query,
            profile,
            topic,
            out,
        } => {
            if out.exists() {
                bail!(
                    "run file already exists at {}; move it aside or choose a different --out path",
                    out.display()
                );
            }
            let state = ResearchRunState {
                query: query.clone(),
                profile,
                topic,
                status: RunStatus::Open,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                budgets: profile_budget(config, profile),
                spent: ProviderBudgets::default(),
                debits: Vec::new(),
                provider_errors: Vec::new(),
                source_ids: Vec::new(),
            };
            ensure_parent(&out)?;
            fs::write(&out, serde_json::to_vec_pretty(&state)?)?;
            if json_out {
                print_json(&json!({ "run": out, "state": state }))
            } else {
                println!("run: {}", out.display());
                Ok(())
            }
        }
        RunCommand::Status { run } => {
            let state = read_run_state(&run)?;
            let remaining = remaining_budgets(&state);
            let source_count = state.source_ids.len();
            if json_out {
                print_json(
                    &json!({ "run": run, "state": state, "remaining": remaining, "source_count": source_count }),
                )
            } else {
                println!("status: {:?}", state.status);
                println!("profile: {}", state.profile);
                println!("source_count: {}", source_count);
                println!("remaining:");
                print_budgets(&remaining);
                Ok(())
            }
        }
        RunCommand::Debit {
            run,
            provider,
            count,
            note,
        } => {
            let state = debit_run_budget(&run, provider, count, note.as_deref())?;
            if json_out {
                print_json(
                    &json!({ "run": run, "state": state, "remaining": remaining_budgets(&state) }),
                )
            } else {
                println!("debited {} from {}", count, provider_name(provider));
                Ok(())
            }
        }
        RunCommand::Close { run } => {
            let state = close_run_state(&run)?;
            if json_out {
                print_json(&json!({ "run": run, "state": state }))
            } else {
                println!("closed: {}", run.display());
                Ok(())
            }
        }
    }
}

async fn run_eval(args: EvalArgs, json_out: bool) -> Result<()> {
    let suite = load_eval_suite(args.suite.as_deref())?;
    let selected = select_eval_tasks(&suite, &args.task)?;

    if args.list {
        let tasks = selected
            .iter()
            .map(|task| EvalTaskSummary {
                id: task.id.clone(),
                kind: task.kind.clone(),
                description: task.description.clone(),
            })
            .collect::<Vec<_>>();
        let result = json!({
            "suite": suite.suite,
            "description": suite.description,
            "tasks": tasks,
        });
        if json_out {
            print_json(&result)?;
        } else {
            for task in tasks {
                println!("{} [{}] {}", task.id, task.kind, task.description);
            }
        }
        return Ok(());
    }

    let mut outcomes = Vec::new();
    for task in selected {
        let assertions = evaluate_eval_task(task);
        let failed =
            !assertions.failures.is_empty() || (args.strict && !assertions.warnings.is_empty());
        outcomes.push(EvalTaskOutcome {
            id: task.id.clone(),
            kind: task.kind.clone(),
            description: task.description.clone(),
            status: if failed { "failed" } else { "passed" }.to_string(),
            failures: assertions.failures,
            warnings: assertions.warnings,
            details: assertions.details,
        });
    }

    let mut live = Vec::new();
    if args.live {
        let _client = http_client()?;
        if std::env::var_os("CONTEXT7_API_KEY").is_some() {
            live.push(json!({ "provider": "context7", "status": "configured" }));
        }
        if std::env::var_os("FIRECRAWL_API_KEY").is_some() {
            live.push(json!({ "provider": "firecrawl", "status": "configured" }));
        }
        let github = github_token().is_some();
        live.push(json!({ "provider": "github", "status": if github { "configured" } else { "public-only" } }));
    }

    let failed = outcomes.iter().any(|outcome| outcome.status == "failed");
    let passed = outcomes
        .iter()
        .filter(|outcome| outcome.status == "passed")
        .count();
    let failed_count = outcomes.len() - passed;
    let result = json!({
        "suite": suite.suite,
        "description": suite.description,
        "offline": {
            "passed": passed,
            "failed": failed_count,
            "strict": args.strict,
            "tasks": outcomes
        },
        "live": live
    });
    if json_out {
        print_json(&result)?;
        if failed {
            bail!("offline eval failures");
        }
        Ok(())
    } else {
        println!("offline: {passed}/{} tasks passed", passed + failed_count);
        if failed {
            println!(
                "{}",
                serde_json::to_string_pretty(&result["offline"]["tasks"])?
            );
            bail!("offline eval failures");
        }
        if args.live {
            println!("{}", serde_json::to_string_pretty(&result["live"])?);
        }
        Ok(())
    }
}

fn load_eval_suite(path: Option<&Path>) -> Result<EvalSuite> {
    let text = match path {
        Some(path) => fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?,
        None => DEFAULT_EVAL_SUITE.to_string(),
    };
    let suite: EvalSuite =
        serde_json::from_str(&text).context("failed to parse eval suite JSON")?;
    if suite.tasks.is_empty() {
        bail!("eval suite has no tasks");
    }
    let mut seen = BTreeSet::new();
    for task in &suite.tasks {
        if !seen.insert(task.id.clone()) {
            bail!("duplicate eval task id `{}`", task.id);
        }
    }
    Ok(suite)
}

fn select_eval_tasks<'a>(suite: &'a EvalSuite, ids: &[String]) -> Result<Vec<&'a EvalTask>> {
    if ids.is_empty() {
        return Ok(suite.tasks.iter().collect());
    }
    let requested = ids.iter().cloned().collect::<BTreeSet<_>>();
    let selected = suite
        .tasks
        .iter()
        .filter(|task| requested.contains(&task.id))
        .collect::<Vec<_>>();
    let found = selected
        .iter()
        .map(|task| task.id.clone())
        .collect::<BTreeSet<_>>();
    let missing = requested.difference(&found).cloned().collect::<Vec<_>>();
    if !missing.is_empty() {
        bail!("unknown eval task id(s): {}", missing.join(", "));
    }
    Ok(selected)
}

fn evaluate_eval_task(task: &EvalTask) -> EvalAssertions {
    let mut assertions = EvalAssertions::default();
    let result = match task.kind.as_str() {
        "route-classification" => evaluate_route_eval(task, &mut assertions),
        "privacy-redaction" => evaluate_privacy_eval(task, &mut assertions),
        "budget-plan" => evaluate_budget_eval(task, &mut assertions),
        "evidence-contract" => evaluate_evidence_contract_eval(task, &mut assertions),
        "report-contract" => evaluate_report_contract_eval(task, &mut assertions),
        other => {
            assertions
                .failures
                .push(format!("unsupported eval task kind `{other}`"));
            Ok(())
        }
    };
    if let Err(error) = result {
        assertions.failures.push(error.to_string());
    }
    assertions
}

fn evaluate_route_eval(task: &EvalTask, assertions: &mut EvalAssertions) -> Result<()> {
    let url = required_str(&task.input, "url")?;
    let body = optional_str(&task.input, "body")?.unwrap_or("");
    let content_type = optional_str(&task.input, "content_type")?;
    let report = classify_body(url, content_type, None, body);

    assertions
        .details
        .insert("route".to_string(), json!(route_name(report.route)));
    assertions
        .details
        .insert("reason".to_string(), json!(report.reason));

    if let Some(expected_route) = optional_str(&task.expected, "route")? {
        assert_text_eq(
            assertions,
            "route",
            expected_route,
            route_name(report.route),
        );
    }
    if let Some(expected_privacy) = optional_str(&task.expected, "privacy")? {
        let privacy = classify_privacy(url);
        assertions
            .details
            .insert("privacy".to_string(), json!(privacy_class_name(privacy)));
        assert_text_eq(
            assertions,
            "privacy",
            expected_privacy,
            privacy_class_name(privacy),
        );
    }
    Ok(())
}

fn evaluate_privacy_eval(task: &EvalTask, assertions: &mut EvalAssertions) -> Result<()> {
    let config = ResearchConfig::default();
    let url = optional_str(&task.input, "url")?;
    let metadata_input = optional_str(&task.input, "metadata_text")?;
    if url.is_none() && metadata_input.is_none() {
        bail!("privacy-redaction requires `url` or `metadata_text` input");
    }
    if url.is_none()
        && (optional_str(&task.expected, "privacy")?.is_some()
            || optional_str(&task.expected, "redacted_url")?.is_some())
    {
        bail!("privacy-redaction expectations `privacy` and `redacted_url` require `url` input");
    }
    if metadata_input.is_none() && optional_str(&task.expected, "metadata_text")?.is_some() {
        bail!("privacy-redaction expectation `metadata_text` requires `metadata_text` input");
    }

    if let Some(url) = url {
        let privacy = classify_privacy(url);
        let redacted = redact_url_query_secrets(url);
        assertions
            .details
            .insert("privacy".to_string(), json!(privacy_class_name(privacy)));
        assertions
            .details
            .insert("redacted_url".to_string(), json!(redacted));
        if let Some(expected_privacy) = optional_str(&task.expected, "privacy")? {
            assert_text_eq(
                assertions,
                "privacy",
                expected_privacy,
                privacy_class_name(privacy),
            );
        }
        if let Some(expected_redacted) = optional_str(&task.expected, "redacted_url")? {
            let actual = assertions
                .details
                .get("redacted_url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            assert_text_eq(assertions, "redacted_url", expected_redacted, &actual);
        }
    }
    if let Some(text) = metadata_input {
        let redacted = metadata_text(text, &config);
        assertions
            .details
            .insert("metadata_text".to_string(), json!(redacted));
        if let Some(expected_text) = optional_str(&task.expected, "metadata_text")? {
            let actual = assertions
                .details
                .get("metadata_text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            assert_text_eq(assertions, "metadata_text", expected_text, &actual);
        }
    }
    Ok(())
}

fn evaluate_budget_eval(task: &EvalTask, assertions: &mut EvalAssertions) -> Result<()> {
    let query = required_str(&task.input, "query")?;
    let profile =
        parse_research_profile(optional_str(&task.input, "profile")?.unwrap_or("standard"))?;
    let topic = parse_topic_kind(optional_str(&task.input, "topic")?.unwrap_or("general"))?;
    let plan = build_plan(query, profile, topic, &ResearchConfig::default());
    let route_order = plan
        .route_order
        .iter()
        .map(|route| route_name(*route))
        .collect::<Vec<_>>();
    assertions
        .details
        .insert("route_order".to_string(), json!(route_order));
    assertions
        .details
        .insert("budgets".to_string(), json!(plan.budgets));

    if let Some(prefix) = optional_str_array(&task.expected, "route_order_prefix")? {
        let actual = route_order
            .iter()
            .take(prefix.len())
            .copied()
            .collect::<Vec<_>>();
        if actual != prefix {
            assertions.failures.push(format!(
                "route_order_prefix expected {:?}, got {:?}",
                prefix, actual
            ));
        }
    }
    if let Some(expected_budgets) = optional_object(&task.expected, "budgets")? {
        for (key, expected) in expected_budgets {
            let Some(expected) = expected.as_u64() else {
                assertions
                    .failures
                    .push(format!("budgets.{key} expected value must be an integer"));
                continue;
            };
            let actual = budget_value(&plan.budgets, key);
            match actual {
                Some(actual) if u64::from(actual) == expected => {}
                Some(actual) => assertions
                    .failures
                    .push(format!("budgets.{key} expected {expected}, got {actual}")),
                None => assertions
                    .failures
                    .push(format!("unknown budget field `{key}`")),
            }
        }
    }
    Ok(())
}

fn evaluate_evidence_contract_eval(task: &EvalTask, assertions: &mut EvalAssertions) -> Result<()> {
    let sources = required_array(&task.input, "sources")?;
    let claims = required_array(&task.input, "claims")?;
    let source_ids = sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            source
                .get("id")
                .and_then(Value::as_str)
                .with_context(|| format!("sources[{index}].id must be a string"))
                .map(str::to_string)
        })
        .collect::<Result<BTreeSet<_>>>()?;
    let min_sources = optional_u64(&task.expected, "min_sources")?.unwrap_or(1);
    let min_claims = optional_u64(&task.expected, "min_claims")?.unwrap_or(1);
    let max_uncited_claims = optional_u64(&task.expected, "max_uncited_claims")?.unwrap_or(0);
    let min_confidence = optional_f64(&task.expected, "min_confidence")?;

    let mut uncited_claims = 0_u64;
    let mut missing_sources = Vec::new();
    let mut low_confidence = Vec::new();
    for claim in claims {
        let claim_id = claim
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("<unnamed claim>");
        let claim_sources = claim
            .get("sources")
            .and_then(Value::as_array)
            .with_context(|| format!("{claim_id}.sources must be an array"))?
            .iter()
            .enumerate()
            .map(|(index, value)| {
                value
                    .as_str()
                    .with_context(|| format!("{claim_id}.sources[{index}] must be a string"))
                    .map(str::to_string)
            })
            .collect::<Result<Vec<_>>>()?;
        if claim_sources.is_empty() {
            uncited_claims += 1;
        }
        for source_id in claim_sources {
            if !source_ids.contains(&source_id) {
                missing_sources.push(format!("{claim_id}->{source_id}"));
            }
        }
        if let Some(min_confidence) = min_confidence {
            let confidence = claim
                .get("confidence")
                .and_then(Value::as_f64)
                .unwrap_or_default();
            if confidence < min_confidence {
                low_confidence.push(format!("{claim_id}:{confidence:.2}"));
            }
        }
    }

    assertions
        .details
        .insert("source_count".to_string(), json!(source_ids.len()));
    assertions
        .details
        .insert("claim_count".to_string(), json!(claims.len()));
    assertions
        .details
        .insert("uncited_claims".to_string(), json!(uncited_claims));

    if source_ids.len() < usize::try_from(min_sources).unwrap_or(usize::MAX) {
        assertions.failures.push(format!(
            "source_count expected at least {min_sources}, got {}",
            source_ids.len()
        ));
    }
    if claims.len() < usize::try_from(min_claims).unwrap_or(usize::MAX) {
        assertions.failures.push(format!(
            "claim_count expected at least {min_claims}, got {}",
            claims.len()
        ));
    }
    if uncited_claims > max_uncited_claims {
        assertions.failures.push(format!(
            "uncited_claims expected at most {max_uncited_claims}, got {uncited_claims}"
        ));
    }
    if !missing_sources.is_empty() {
        assertions.failures.push(format!(
            "claims reference missing source ids: {}",
            missing_sources.join(", ")
        ));
    }
    if !low_confidence.is_empty() {
        assertions.warnings.push(format!(
            "claims below confidence threshold: {}",
            low_confidence.join(", ")
        ));
    }
    Ok(())
}

fn evaluate_report_contract_eval(task: &EvalTask, assertions: &mut EvalAssertions) -> Result<()> {
    let report = required_str(&task.input, "report")?;
    for section in optional_str_array(&task.expected, "required_sections")?.unwrap_or_default() {
        if !report_has_heading(report, section) {
            assertions
                .failures
                .push(format!("report missing required section `{section}`"));
        }
    }
    for phrase in optional_str_array(&task.expected, "forbidden_phrases")?.unwrap_or_default() {
        if report.contains(phrase) {
            assertions
                .failures
                .push(format!("report contains forbidden phrase `{phrase}`"));
        }
    }
    for source_id in
        optional_str_array(&task.expected, "required_source_mentions")?.unwrap_or_default()
    {
        if !report.contains(source_id) {
            assertions
                .failures
                .push(format!("report missing source mention `{source_id}`"));
        }
    }
    assertions
        .details
        .insert("chars".to_string(), json!(report.chars().count()));
    Ok(())
}

fn report_has_heading(report: &str, section: &str) -> bool {
    report.lines().any(|line| {
        let trimmed = line.trim();
        let Some(title) = trimmed.strip_prefix("## ") else {
            return false;
        };
        title.trim() == section
    })
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    optional_str(value, key)?.with_context(|| format!("missing string input `{key}`"))
}

fn optional_str<'a>(value: &'a Value, key: &str) -> Result<Option<&'a str>> {
    let Some(value) = value.get(key) else {
        return Ok(None);
    };
    value
        .as_str()
        .map(Some)
        .with_context(|| format!("`{key}` must be a string"))
}

fn required_array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .with_context(|| format!("missing array input `{key}`"))
}

fn optional_object<'a>(
    value: &'a Value,
    key: &str,
) -> Result<Option<&'a serde_json::Map<String, Value>>> {
    let Some(value) = value.get(key) else {
        return Ok(None);
    };
    value
        .as_object()
        .map(Some)
        .with_context(|| format!("`{key}` must be an object"))
}

fn optional_u64(value: &Value, key: &str) -> Result<Option<u64>> {
    let Some(value) = value.get(key) else {
        return Ok(None);
    };
    value
        .as_u64()
        .map(Some)
        .with_context(|| format!("`{key}` must be an unsigned integer"))
}

fn optional_f64(value: &Value, key: &str) -> Result<Option<f64>> {
    let Some(value) = value.get(key) else {
        return Ok(None);
    };
    value
        .as_f64()
        .map(Some)
        .with_context(|| format!("`{key}` must be a number"))
}

fn optional_str_array<'a>(value: &'a Value, key: &str) -> Result<Option<Vec<&'a str>>> {
    let Some(values) = value.get(key) else {
        return Ok(None);
    };
    let Some(values) = values.as_array() else {
        bail!("`{key}` must be an array of strings");
    };
    let mut out = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let Some(text) = value.as_str() else {
            bail!("`{key}[{index}]` must be a string");
        };
        out.push(text);
    }
    Ok(Some(out))
}

fn assert_text_eq(assertions: &mut EvalAssertions, name: &str, expected: &str, actual: &str) {
    if expected != actual {
        assertions
            .failures
            .push(format!("{name} expected `{expected}`, got `{actual}`"));
    }
}

fn parse_research_profile(value: &str) -> Result<ResearchProfile> {
    match value {
        "quick" => Ok(ResearchProfile::Quick),
        "standard" => Ok(ResearchProfile::Standard),
        "deep" => Ok(ResearchProfile::Deep),
        "exhaustive" => Ok(ResearchProfile::Exhaustive),
        _ => bail!("unknown research profile `{value}`"),
    }
}

fn parse_topic_kind(value: &str) -> Result<TopicKind> {
    match value {
        "general" => Ok(TopicKind::General),
        "docs" => Ok(TopicKind::Docs),
        "github" => Ok(TopicKind::Github),
        "dependency" => Ok(TopicKind::Dependency),
        "openai" => Ok(TopicKind::Openai),
        "rendered" => Ok(TopicKind::Rendered),
        _ => bail!("unknown topic `{value}`"),
    }
}

fn budget_value(budgets: &ProviderBudgets, key: &str) -> Option<u32> {
    match key {
        "codex_web_queries" => Some(budgets.codex_web_queries),
        "context7_calls" => Some(budgets.context7_calls),
        "github_calls" => Some(budgets.github_calls),
        "exa_calls" => Some(budgets.exa_calls),
        "direct_fetches" => Some(budgets.direct_fetches),
        "browser_fetches" => Some(budgets.browser_fetches),
        "firecrawl_calls" => Some(budgets.firecrawl_calls),
        _ => None,
    }
}

fn build_plan(
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
struct FetchedBody {
    url: String,
    status: u16,
    content_type: Option<String>,
    bytes: usize,
    body: String,
    #[serde(skip_serializing)]
    raw_body: Vec<u8>,
}

#[derive(Serialize)]
struct FetchedOutput {
    source_id: Option<String>,
    #[serde(flatten)]
    fetched: FetchedBody,
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

async fn firecrawl_scrape(
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

async fn github_get(
    client: &reqwest::Client,
    url: &str,
    params: &[(&str, String)],
    retries: u8,
) -> Result<Value> {
    Ok(github_get_response(client, url, params, retries)
        .await?
        .value)
}

async fn github_get_tracked(
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

async fn github_get_response(
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

fn github_rate_limit_metadata(headers: &HeaderMap) -> Value {
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

fn github_pagination_metadata(headers: &HeaderMap) -> Value {
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

async fn context7_send(request: reqwest::RequestBuilder) -> Result<Value> {
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

struct LoadedConfig {
    path: Option<PathBuf>,
    config: ResearchConfig,
}

fn load_config(explicit: Option<&Path>) -> Result<LoadedConfig> {
    let (path, required) = if let Some(path) = explicit {
        (Some(path.to_path_buf()), true)
    } else if let Some(path) = std::env::var_os("CODEX_RESEARCH_CONFIG") {
        (Some(PathBuf::from(path)), true)
    } else {
        (
            find_nearest_config().or_else(|| {
                BaseDirs::new()
                    .map(|base| base.config_dir().join("codex-research").join("config.toml"))
                    .filter(|path| path.exists())
            }),
            false,
        )
    };

    let config = if let Some(path) = &path {
        if path.exists() {
            let text = fs::read_to_string(path)
                .with_context(|| format!("failed to read config: {}", path.display()))?;
            toml::from_str(&text)
                .with_context(|| format!("failed to parse config: {}", path.display()))?
        } else if required {
            bail!("config path does not exist: {}", path.display());
        } else {
            ResearchConfig::default()
        }
    } else {
        ResearchConfig::default()
    };
    Ok(LoadedConfig { path, config })
}

fn find_nearest_config() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join(".codex").join("research").join("config.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn default_config_toml() -> Result<String> {
    Ok(toml::to_string_pretty(&ResearchConfig::default())?)
}

fn quick_budget() -> ProviderBudgets {
    ProviderBudgets {
        codex_web_queries: 2,
        context7_calls: 1,
        github_calls: 1,
        exa_calls: 0,
        direct_fetches: 2,
        browser_fetches: 0,
        firecrawl_calls: 0,
    }
}

fn standard_budget() -> ProviderBudgets {
    ProviderBudgets {
        codex_web_queries: 4,
        context7_calls: 3,
        github_calls: 4,
        exa_calls: 2,
        direct_fetches: 8,
        browser_fetches: 2,
        firecrawl_calls: 1,
    }
}

fn deep_budget() -> ProviderBudgets {
    ProviderBudgets {
        codex_web_queries: 8,
        context7_calls: 4,
        github_calls: 8,
        exa_calls: 4,
        direct_fetches: 12,
        browser_fetches: 4,
        firecrawl_calls: 6,
    }
}

fn exhaustive_budget() -> ProviderBudgets {
    ProviderBudgets {
        codex_web_queries: 12,
        context7_calls: 8,
        github_calls: 16,
        exa_calls: 8,
        direct_fetches: 24,
        browser_fetches: 8,
        firecrawl_calls: 12,
    }
}

fn profile_budget(config: &ResearchConfig, profile: ResearchProfile) -> ProviderBudgets {
    match profile {
        ResearchProfile::Quick => config.profiles.quick.clone(),
        ResearchProfile::Standard => config.profiles.standard.clone(),
        ResearchProfile::Deep => config.profiles.deep.clone(),
        ResearchProfile::Exhaustive => config.profiles.exhaustive.clone(),
    }
}

fn deny_string() -> String {
    "deny".to_string()
}

fn default_true() -> bool {
    true
}

fn default_github_per_page() -> u8 {
    10
}

fn default_github_per_page_max() -> u8 {
    100
}

fn default_backoff_retries() -> u8 {
    2
}

fn default_cache_ttl_hours() -> u32 {
    168
}

fn default_firecrawl_max_age_ms() -> u64 {
    172_800_000
}

fn effective_firecrawl_store_in_cache(no_store_in_cache: bool, config: &ResearchConfig) -> bool {
    config.providers.firecrawl.store_in_cache_default && !no_store_in_cache
}

fn github_issue_call_count(comments: bool) -> u32 {
    1 + u32::from(comments)
}

fn github_pr_call_count(files: bool, comments: bool, reviews: bool) -> u32 {
    1 + u32::from(files) + if comments { 2 } else { 0 } + u32::from(reviews)
}

fn read_run_state(path: &Path) -> Result<ResearchRunState> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read run state: {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("failed to parse run state: {}", path.display()))
}

#[cfg(test)]
fn write_run_state(path: &Path, state: &ResearchRunState) -> Result<()> {
    let _lock = acquire_run_lock(path)?;
    write_run_state_unlocked(path, state)
}

fn write_run_state_unlocked(path: &Path, state: &ResearchRunState) -> Result<()> {
    ensure_parent(path)?;
    let temp_path = path.with_file_name(format!(
        ".{}.tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("run.json"),
        short_hash(Utc::now().to_rfc3339())
    ));
    fs::write(&temp_path, serde_json::to_vec_pretty(state)?)?;
    fs::rename(&temp_path, path)?;
    Ok(())
}

struct RunLock {
    path: PathBuf,
}

impl Drop for RunLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn run_lock_path(path: &Path) -> PathBuf {
    path.with_file_name(format!(
        "{}.lock",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("run.json")
    ))
}

fn acquire_run_lock(path: &Path) -> Result<RunLock> {
    ensure_parent(path)?;
    let lock_path = run_lock_path(path);
    for _ in 0..100 {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                writeln!(file, "pid={} created_at={}", std::process::id(), Utc::now())?;
                return Ok(RunLock { path: lock_path });
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(err) => {
                return Err(err).with_context(|| {
                    format!("failed to create run lock: {}", lock_path.display())
                });
            }
        }
    }
    bail!("timed out waiting for run lock: {}", lock_path.display())
}

fn maybe_debit(
    budget: &BudgetArgs,
    provider: ProviderKind,
    count: u32,
    note: Option<&str>,
) -> Result<()> {
    if let Some(path) = &budget.run
        && !budget.no_budget
    {
        let result = debit_run_budget(path, provider, count, note);
        if let Err(error) = &result {
            let _ = append_provider_error(path, provider, &error.to_string());
        }
        result?;
    }
    Ok(())
}

async fn track_provider_result<T, F>(
    budget: &BudgetArgs,
    provider: ProviderKind,
    future: F,
) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let result = future.await;
    if let Err(error) = &result {
        let _ = append_provider_error_from_budget(budget, provider, &error.to_string());
    }
    result
}

fn debit_run_budget(
    path: &Path,
    provider: ProviderKind,
    count: u32,
    note: Option<&str>,
) -> Result<ResearchRunState> {
    let _lock = acquire_run_lock(path)?;
    let mut state = read_run_state(path)?;
    if state.status == RunStatus::Closed {
        bail!("research run is closed: {}", path.display());
    }
    let remaining = provider_remaining(&state, provider);
    if remaining < count {
        bail!(
            "budget exhausted for {}; remaining={} requested={}",
            provider_name(provider),
            remaining,
            count
        );
    }
    *budget_slot_mut(&mut state.spent, provider) += count;
    state.debits.push(RunDebit {
        provider,
        count,
        note: note.map(str::to_string),
        created_at: Utc::now(),
    });
    state.updated_at = Utc::now();
    write_run_state_unlocked(path, &state)?;
    Ok(state)
}

fn attach_source_to_run(budget: &BudgetArgs, source_id: &str) -> Result<()> {
    let Some(path) = &budget.run else {
        return Ok(());
    };
    let _lock = acquire_run_lock(path)?;
    let mut state = read_run_state(path)?;
    if state.status == RunStatus::Closed {
        bail!("research run is closed: {}", path.display());
    }
    if !state.source_ids.iter().any(|id| id == source_id) {
        state.source_ids.push(source_id.to_string());
        state.updated_at = Utc::now();
        write_run_state_unlocked(path, &state)?;
    }
    Ok(())
}

fn close_run_state(path: &Path) -> Result<ResearchRunState> {
    let _lock = acquire_run_lock(path)?;
    let mut state = read_run_state(path)?;
    state.status = RunStatus::Closed;
    state.updated_at = Utc::now();
    write_run_state_unlocked(path, &state)?;
    Ok(state)
}

fn append_provider_error_from_budget(
    budget: &BudgetArgs,
    provider: ProviderKind,
    message: &str,
) -> Result<()> {
    if let Some(path) = &budget.run
        && !budget.no_budget
    {
        append_provider_error(path, provider, message)?;
    }
    Ok(())
}

fn append_provider_error(path: &Path, provider: ProviderKind, message: &str) -> Result<()> {
    let _lock = acquire_run_lock(path)?;
    let mut state = read_run_state(path)?;
    if state.status == RunStatus::Closed {
        return Ok(());
    }
    state.provider_errors.push(ProviderError {
        provider,
        message: message.to_string(),
        created_at: Utc::now(),
    });
    state.updated_at = Utc::now();
    write_run_state_unlocked(path, &state)?;
    Ok(())
}

fn provider_remaining(state: &ResearchRunState, provider: ProviderKind) -> u32 {
    budget_slot(&state.budgets, provider).saturating_sub(budget_slot(&state.spent, provider))
}

fn remaining_budgets(state: &ResearchRunState) -> ProviderBudgets {
    ProviderBudgets {
        codex_web_queries: provider_remaining(state, ProviderKind::CodexWeb),
        context7_calls: provider_remaining(state, ProviderKind::Context7),
        github_calls: provider_remaining(state, ProviderKind::Github),
        exa_calls: provider_remaining(state, ProviderKind::Exa),
        direct_fetches: provider_remaining(state, ProviderKind::Direct),
        browser_fetches: provider_remaining(state, ProviderKind::Browser),
        firecrawl_calls: provider_remaining(state, ProviderKind::Firecrawl),
    }
}

fn budget_slot(budgets: &ProviderBudgets, provider: ProviderKind) -> u32 {
    match provider {
        ProviderKind::CodexWeb => budgets.codex_web_queries,
        ProviderKind::Context7 => budgets.context7_calls,
        ProviderKind::Github => budgets.github_calls,
        ProviderKind::Exa => budgets.exa_calls,
        ProviderKind::Direct => budgets.direct_fetches,
        ProviderKind::Browser => budgets.browser_fetches,
        ProviderKind::Firecrawl => budgets.firecrawl_calls,
    }
}

fn budget_slot_mut(budgets: &mut ProviderBudgets, provider: ProviderKind) -> &mut u32 {
    match provider {
        ProviderKind::CodexWeb => &mut budgets.codex_web_queries,
        ProviderKind::Context7 => &mut budgets.context7_calls,
        ProviderKind::Github => &mut budgets.github_calls,
        ProviderKind::Exa => &mut budgets.exa_calls,
        ProviderKind::Direct => &mut budgets.direct_fetches,
        ProviderKind::Browser => &mut budgets.browser_fetches,
        ProviderKind::Firecrawl => &mut budgets.firecrawl_calls,
    }
}

fn print_budgets(budgets: &ProviderBudgets) {
    println!("  codex-web: {}", budgets.codex_web_queries);
    println!("  context7: {}", budgets.context7_calls);
    println!("  github: {}", budgets.github_calls);
    println!("  exa: {}", budgets.exa_calls);
    println!("  direct: {}", budgets.direct_fetches);
    println!("  browser: {}", budgets.browser_fetches);
    println!("  firecrawl: {}", budgets.firecrawl_calls);
}

fn init_db(paths: &ResearchPaths) -> Result<()> {
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

struct SourceCacheInsert<'a> {
    url: &'a str,
    provider: &'a str,
    status: Option<u16>,
    content_hash: Option<&'a str>,
    route: Option<&'a str>,
    title: Option<&'a str>,
    canonical_url: Option<&'a str>,
    freshness_status: &'a str,
    privacy_classification: &'a str,
    raw_body_stored: bool,
    metadata: Value,
    redact_query_secrets: bool,
}

fn record_source_cache(paths: &ResearchPaths, source: SourceCacheInsert<'_>) -> Result<String> {
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

fn add_column_if_missing(
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

fn cached_source(paths: &ResearchPaths, source_id: &str) -> Result<Option<SourceCacheRecord>> {
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

fn list_cached_sources(
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

fn source_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SourceCacheRecord> {
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

fn record_route_memory(
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

fn route_memory_for_url(paths: &ResearchPaths, url: &str) -> Result<Option<RouteMemoryHit>> {
    let Some(domain) = url_domain(url) else {
        return Ok(None);
    };
    let rows = list_route_memory(paths, Some(&domain))?;
    Ok(rows.into_iter().next())
}

fn list_route_memory(paths: &ResearchPaths, domain: Option<&str>) -> Result<Vec<RouteMemoryHit>> {
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

fn route_memory_hit_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RouteMemoryHit> {
    Ok(RouteMemoryHit {
        domain: row.get(0)?,
        preferred_route: row.get(1)?,
        successes: row.get::<_, i64>(2)? as u32,
        failures: row.get::<_, i64>(3)? as u32,
        updated_at: row.get(4)?,
    })
}

fn prune_cache(paths: &ResearchPaths, older_than_days: i64, dry_run: bool) -> Result<i64> {
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

fn url_domain(value: &str) -> Option<String> {
    Url::parse(value)
        .ok()
        .and_then(|url| url.host_str().map(|host| host.to_ascii_lowercase()))
}

fn classify_privacy(value: &str) -> PrivacyClass {
    let Ok(url) = Url::parse(value) else {
        return PrivacyClass::Ambiguous;
    };
    if url.scheme() == "file" {
        return PrivacyClass::PrivateOrAuthenticated;
    }
    if !url.username().is_empty() || url.password().is_some() {
        return PrivacyClass::PrivateOrAuthenticated;
    }
    if let Some(host) = url.host()
        && let Some(ip) = host_ip_addr(host)
        && private_or_local_ip(ip)
    {
        return PrivacyClass::PrivateOrAuthenticated;
    }
    let Some(host) = url.host_str().map(|host| host.to_ascii_lowercase()) else {
        return PrivacyClass::Ambiguous;
    };
    if matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.ends_with(".corp")
        || (!host.contains('.') && host != "github")
    {
        return PrivacyClass::PrivateOrAuthenticated;
    }
    for (key, _) in url.query_pairs() {
        if secret_query_key(&key) {
            return PrivacyClass::PrivateOrAuthenticated;
        }
    }
    PrivacyClass::Public
}

fn enforce_external_privacy(
    privacy: PrivacyClass,
    explicit_allow_private_external: bool,
    config: &ResearchConfig,
    provider: &str,
) -> Result<()> {
    let allow_private_external =
        explicit_allow_private_external || config.privacy.allow_private_external;
    match privacy {
        PrivacyClass::Public | PrivacyClass::SensitivePublic => Ok(()),
        PrivacyClass::PrivateOrAuthenticated
            if allow_private_external
                || config
                    .privacy
                    .private_external_default
                    .eq_ignore_ascii_case("allow") =>
        {
            Ok(())
        }
        PrivacyClass::Ambiguous
            if allow_private_external
                || config
                    .privacy
                    .ambiguous_external_default
                    .eq_ignore_ascii_case("allow") =>
        {
            Ok(())
        }
        PrivacyClass::PrivateOrAuthenticated => {
            bail!(
                "{provider} refused private/authenticated input; pass --allow-private-external to override"
            )
        }
        PrivacyClass::Ambiguous => {
            bail!(
                "{provider} refused ambiguous input; pass --privacy public or --allow-private-external to override"
            )
        }
    }
}

fn host_ip_addr(host: url::Host<&str>) -> Option<IpAddr> {
    match host {
        url::Host::Ipv4(ip) => Some(IpAddr::V4(ip)),
        url::Host::Ipv6(ip) => Some(IpAddr::V6(ip)),
        url::Host::Domain(_) => None,
    }
}

fn private_or_local_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified()
        }
        IpAddr::V6(ip) => {
            if let Some(mapped) = ip.to_ipv4_mapped() {
                return private_or_local_ip(IpAddr::V4(mapped));
            }
            let first = ip.segments()[0];
            ip.is_loopback()
                || ip.is_unspecified()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
        }
    }
}

fn metadata_text(value: &str, config: &ResearchConfig) -> String {
    if config.privacy.redact_query_secrets && text_looks_secret_bearing(value) {
        "[redacted]".to_string()
    } else {
        value.to_string()
    }
}

fn text_looks_secret_bearing(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "token",
        "api_key",
        "apikey",
        "secret",
        "password",
        "authorization",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn redact_url_query_secrets(value: &str) -> String {
    let Ok(mut url) = Url::parse(value) else {
        return value.to_string();
    };
    if url.query().is_none() {
        return value.to_string();
    }
    let pairs = url
        .query_pairs()
        .map(|(key, value)| {
            let key = key.to_string();
            let value = if secret_query_key(&key) {
                "[redacted]".to_string()
            } else {
                value.to_string()
            };
            (key, value)
        })
        .collect::<Vec<_>>();
    {
        let mut query = url.query_pairs_mut();
        query.clear();
        for (key, value) in pairs {
            query.append_pair(&key, &value);
        }
    }
    url.to_string()
}

fn redact_metadata_urls(value: Value) -> Value {
    match value {
        Value::String(text) => Value::String(redact_url_query_secrets(&text)),
        Value::Array(values) => {
            Value::Array(values.into_iter().map(redact_metadata_urls).collect())
        }
        Value::Object(entries) => Value::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key, redact_metadata_urls(value)))
                .collect(),
        ),
        other => other,
    }
}

fn secret_query_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "token"
            | "access_token"
            | "auth"
            | "authorization"
            | "signature"
            | "sig"
            | "x-amz-signature"
            | "x-amz-credential"
            | "key"
            | "apikey"
            | "api_key"
            | "secret"
            | "password"
            | "sas"
    )
}

fn privacy_class_name(privacy: PrivacyClass) -> &'static str {
    match privacy {
        PrivacyClass::Public => "public",
        PrivacyClass::SensitivePublic => "sensitive-public",
        PrivacyClass::PrivateOrAuthenticated => "private-or-authenticated",
        PrivacyClass::Ambiguous => "ambiguous",
    }
}

fn provider_name(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::CodexWeb => "codex-web",
        ProviderKind::Context7 => "context7",
        ProviderKind::Github => "github",
        ProviderKind::Exa => "exa",
        ProviderKind::Direct => "direct",
        ProviderKind::Browser => "browser",
        ProviderKind::Firecrawl => "firecrawl",
    }
}

fn apply_route_memory(url: &str, report: &mut ProbeReport) -> Result<()> {
    if report.route == Route::Github {
        return Ok(());
    }
    let paths = research_paths()?;
    init_db(&paths)?;
    if let Some(hit) = route_memory_for_url(&paths, url)? {
        if hit.successes > hit.failures
            && let Some(route) = route_from_name(&hit.preferred_route)
        {
            report.reason = format!(
                "{} Route memory for {} prefers {} (successes={} failures={}).",
                report.reason, hit.domain, hit.preferred_route, hit.successes, hit.failures
            );
            report.route = route;
        }
        report.route_memory.push(hit);
    }
    Ok(())
}

fn route_from_name(value: &str) -> Option<Route> {
    match value {
        "codex-web" => Some(Route::CodexWeb),
        "context7" => Some(Route::Context7),
        "github" => Some(Route::Github),
        "direct" => Some(Route::Direct),
        "agent-browser" => Some(Route::AgentBrowser),
        "firecrawl" => Some(Route::Firecrawl),
        "exa" => Some(Route::Exa),
        "opensrc" => Some(Route::Opensrc),
        _ => None,
    }
}

fn path_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn slash_path(value: &str) -> String {
    value
        .split('/')
        .map(path_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn github_api_source_url(url: &str) -> String {
    url.to_string()
}

fn github_repo_url(repo: &str, path: &str) -> String {
    format!("https://github.com/{repo}/{path}")
}

fn github_search_limitations(kind: &str) -> Value {
    match kind {
        "code" => json!([
            "requires at least one search term",
            "default branch only",
            "files larger than 384 KB are not searchable",
            "strict search rate limits apply",
            "hydrate promising hits before citing"
        ]),
        _ => json!([
            "search can return incomplete_results on timeout",
            "queries have length and boolean-operator limits",
            "hydrate promising hits before citing"
        ]),
    }
}

fn normalize_compare(mut value: Value) -> Value {
    if let Some(files) = value.get("files").and_then(Value::as_array) {
        let summary = files
            .iter()
            .map(|file| {
                json!({
                    "filename": file.get("filename"),
                    "status": file.get("status"),
                    "previous_filename": file.get("previous_filename"),
                    "additions": file.get("additions"),
                    "deletions": file.get("deletions"),
                    "changes": file.get("changes"),
                    "patch_present": file.get("patch").is_some()
                })
            })
            .collect::<Vec<_>>();
        value["file_summary"] = json!(summary);
    }
    value
}

fn merge_metadata(mut left: Value, right: Value) -> Value {
    if let (Some(left), Some(right)) = (left.as_object_mut(), right.as_object()) {
        for (key, value) in right {
            left.insert(key.clone(), value.clone());
        }
    }
    left
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

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "codex-research-{name}-{}",
            short_hash(format!("{}-{}", std::process::id(), Utc::now()))
        ))
    }

    #[test]
    fn dependency_plan_prefers_docs_source_and_github_routes() {
        let plan = build_plan(
            "verify dependency behavior",
            ResearchProfile::Deep,
            TopicKind::Dependency,
            &ResearchConfig::default(),
        );

        assert_eq!(plan.profile.to_string(), "deep");
        assert_eq!(plan.budgets.context7_calls, 4);
        assert_eq!(plan.budgets.github_calls, 8);
        assert_eq!(plan.route_order[0], Route::Context7);
        assert_eq!(plan.route_order[1], Route::Opensrc);
        assert!(plan.route_order.contains(&Route::Github));
    }

    #[test]
    fn default_eval_suite_is_manifest_backed_and_passes_offline() -> Result<()> {
        let suite = load_eval_suite(None)?;
        assert_eq!(suite.suite, "research-core");
        assert!(suite.tasks.len() >= 5);

        for task in select_eval_tasks(&suite, &[])? {
            let outcome = evaluate_eval_task(task);
            assert!(
                outcome.failures.is_empty(),
                "{} failed: {:?}",
                task.id,
                outcome.failures
            );
        }
        Ok(())
    }

    #[test]
    fn eval_task_filter_reports_unknown_ids() -> Result<()> {
        let suite = load_eval_suite(None)?;
        let result = select_eval_tasks(&suite, &["missing-task".to_string()]);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn privacy_eval_rejects_empty_tasks() {
        let task = EvalTask {
            id: "empty-privacy".to_string(),
            kind: "privacy-redaction".to_string(),
            description: "empty".to_string(),
            input: json!({}),
            expected: json!({}),
        };
        let outcome = evaluate_eval_task(&task);

        assert!(outcome.failures[0].contains("requires `url` or `metadata_text`"));
    }

    #[test]
    fn report_contract_requires_explicit_headings() {
        let task = EvalTask {
            id: "missing-heading".to_string(),
            kind: "report-contract".to_string(),
            description: "heading check".to_string(),
            input: json!({
                "report": "- Provider limits are mentioned in prose only."
            }),
            expected: json!({
                "required_sections": ["Provider limits"]
            }),
        };
        let outcome = evaluate_eval_task(&task);

        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("missing required section"))
        );
    }

    #[test]
    fn eval_rejects_malformed_expected_string_arrays() {
        let task = EvalTask {
            id: "bad-array".to_string(),
            kind: "report-contract".to_string(),
            description: "array validation".to_string(),
            input: json!({
                "report": "## Claims\n- cited"
            }),
            expected: json!({
                "required_sections": ["Claims", 42]
            }),
        };
        let outcome = evaluate_eval_task(&task);

        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("required_sections[1]"))
        );
    }

    #[test]
    fn eval_rejects_malformed_confidence_thresholds() {
        let task = EvalTask {
            id: "bad-confidence".to_string(),
            kind: "evidence-contract".to_string(),
            description: "threshold validation".to_string(),
            input: json!({
                "sources": [{"id": "source-1"}],
                "claims": [{"id": "claim-1", "sources": ["source-1"], "confidence": 0.9}]
            }),
            expected: json!({
                "min_confidence": "high"
            }),
        };
        let outcome = evaluate_eval_task(&task);

        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("min_confidence"))
        );
    }

    #[test]
    fn evidence_contract_rejects_malformed_source_entries() {
        let task = EvalTask {
            id: "bad-source".to_string(),
            kind: "evidence-contract".to_string(),
            description: "source validation".to_string(),
            input: json!({
                "sources": [{"id": 42}],
                "claims": [{"id": "claim-1", "sources": ["source-1"]}]
            }),
            expected: json!({}),
        };
        let outcome = evaluate_eval_task(&task);

        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("sources[0].id"))
        );
    }

    #[test]
    fn evidence_contract_rejects_malformed_claim_citations() {
        let task = EvalTask {
            id: "bad-citation".to_string(),
            kind: "evidence-contract".to_string(),
            description: "citation validation".to_string(),
            input: json!({
                "sources": [{"id": "source-1"}],
                "claims": [{"id": "claim-1", "sources": ["source-1", 42]}]
            }),
            expected: json!({}),
        };
        let outcome = evaluate_eval_task(&task);

        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("claim-1.sources[1]"))
        );
    }

    #[test]
    fn eval_rejects_malformed_scalar_expectations() {
        let task = EvalTask {
            id: "bad-route".to_string(),
            kind: "route-classification".to_string(),
            description: "scalar validation".to_string(),
            input: json!({
                "url": "https://github.com/example/repo/blob/main/README.md",
                "body": "# README"
            }),
            expected: json!({
                "route": ["github"]
            }),
        };
        let outcome = evaluate_eval_task(&task);

        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("route"))
        );
    }

    #[test]
    fn budget_eval_rejects_malformed_budget_expectations() {
        let task = EvalTask {
            id: "bad-budgets".to_string(),
            kind: "budget-plan".to_string(),
            description: "budget validation".to_string(),
            input: json!({
                "query": "verify dependency behavior",
                "profile": "deep",
                "topic": "dependency"
            }),
            expected: json!({
                "budgets": []
            }),
        };
        let outcome = evaluate_eval_task(&task);

        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("budgets"))
        );
    }

    #[test]
    fn default_config_serializes_and_parses() -> Result<()> {
        let text = default_config_toml()?;
        let parsed: ResearchConfig = toml::from_str(&text)?;

        assert_eq!(parsed.profiles.deep.github_calls, 8);
        assert_eq!(parsed.privacy.private_external_default, "deny");
        assert_eq!(parsed.providers.firecrawl.default_max_age_ms, 172_800_000);
        Ok(())
    }

    #[test]
    fn run_budget_debit_exhausts_provider() -> Result<()> {
        let dir = temp_path("run-budget");
        fs::create_dir_all(&dir)?;
        let run = dir.join("run.json");
        let state = ResearchRunState {
            query: "smoke".to_string(),
            profile: ResearchProfile::Quick,
            topic: TopicKind::Github,
            status: RunStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            budgets: quick_budget(),
            spent: ProviderBudgets::default(),
            debits: Vec::new(),
            provider_errors: Vec::new(),
            source_ids: Vec::new(),
        };
        write_run_state(&run, &state)?;

        let state = debit_run_budget(&run, ProviderKind::Github, 1, Some("test"))?;
        assert_eq!(provider_remaining(&state, ProviderKind::Github), 0);
        assert!(debit_run_budget(&run, ProviderKind::Github, 1, Some("test")).is_err());
        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn source_attachment_updates_active_run_state() -> Result<()> {
        let dir = temp_path("run-source");
        fs::create_dir_all(&dir)?;
        let run = dir.join("run.json");
        let state = ResearchRunState {
            query: "smoke".to_string(),
            profile: ResearchProfile::Quick,
            topic: TopicKind::Github,
            status: RunStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            budgets: quick_budget(),
            spent: ProviderBudgets::default(),
            debits: Vec::new(),
            provider_errors: Vec::new(),
            source_ids: Vec::new(),
        };
        write_run_state(&run, &state)?;
        let budget = BudgetArgs {
            run: Some(run.clone()),
            no_budget: true,
        };

        attach_source_to_run(&budget, "src123")?;
        attach_source_to_run(&budget, "src123")?;

        let state = read_run_state(&run)?;
        assert_eq!(state.source_ids, vec!["src123"]);
        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn provider_budget_errors_are_recorded_in_run_state() -> Result<()> {
        let dir = temp_path("run-provider-errors");
        fs::create_dir_all(&dir)?;
        let run = dir.join("run.json");
        let state = ResearchRunState {
            query: "smoke".to_string(),
            profile: ResearchProfile::Quick,
            topic: TopicKind::Github,
            status: RunStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            budgets: quick_budget(),
            spent: ProviderBudgets {
                github_calls: 1,
                ..ProviderBudgets::default()
            },
            debits: Vec::new(),
            provider_errors: Vec::new(),
            source_ids: Vec::new(),
        };
        write_run_state(&run, &state)?;
        let budget = BudgetArgs {
            run: Some(run.clone()),
            no_budget: false,
        };

        assert!(maybe_debit(&budget, ProviderKind::Github, 1, Some("test")).is_err());

        let state = read_run_state(&run)?;
        assert_eq!(state.provider_errors.len(), 1);
        assert_eq!(state.provider_errors[0].provider, ProviderKind::Github);
        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn run_init_refuses_to_overwrite_existing_state() -> Result<()> {
        let dir = temp_path("run-init-existing");
        fs::create_dir_all(&dir)?;
        let run = dir.join("run.json");
        fs::write(&run, b"{\"existing\":true}")?;

        let result = handle_run(
            RunCommand::Init {
                query: "smoke".to_string(),
                profile: ResearchProfile::Quick,
                topic: TopicKind::General,
                out: run.clone(),
            },
            &ResearchConfig::default(),
            false,
        );

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&run)?, "{\"existing\":true}");
        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn close_run_state_preserves_existing_run_history() -> Result<()> {
        let dir = temp_path("run-close");
        fs::create_dir_all(&dir)?;
        let run = dir.join("run.json");
        let state = ResearchRunState {
            query: "smoke".to_string(),
            profile: ResearchProfile::Quick,
            topic: TopicKind::Github,
            status: RunStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            budgets: quick_budget(),
            spent: ProviderBudgets::default(),
            debits: vec![RunDebit {
                provider: ProviderKind::Github,
                count: 1,
                note: Some("test".to_string()),
                created_at: Utc::now(),
            }],
            provider_errors: vec![ProviderError {
                provider: ProviderKind::Github,
                message: "rate limited".to_string(),
                created_at: Utc::now(),
            }],
            source_ids: vec!["src123".to_string()],
        };
        write_run_state(&run, &state)?;

        let state = close_run_state(&run)?;

        assert_eq!(state.status, RunStatus::Closed);
        assert_eq!(state.debits.len(), 1);
        assert_eq!(state.provider_errors.len(), 1);
        assert_eq!(state.source_ids, vec!["src123"]);
        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn provider_call_counts_reflect_hydration_requests() {
        assert_eq!(github_issue_call_count(false), 1);
        assert_eq!(github_issue_call_count(true), 2);
        assert_eq!(github_pr_call_count(false, false, false), 1);
        assert_eq!(github_pr_call_count(true, true, true), 5);
    }

    #[test]
    fn github_pagination_metadata_marks_truncated_link_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            LINK,
            HeaderValue::from_static(
                "<https://api.github.com/repositories/1/pulls/2/files?page=2>; rel=\"next\", <https://api.github.com/repositories/1/pulls/2/files?page=4>; rel=\"last\"",
            ),
        );

        let metadata = github_pagination_metadata(&headers);

        assert_eq!(metadata["truncated"], true);
        assert!(metadata["link"].as_str().unwrap().contains("rel=\"next\""));
    }

    #[test]
    fn firecrawl_cache_default_uses_config_until_flag_overrides() {
        let mut config = ResearchConfig::default();

        assert!(effective_firecrawl_store_in_cache(false, &config));
        assert!(!effective_firecrawl_store_in_cache(true, &config));

        config.providers.firecrawl.store_in_cache_default = false;
        assert!(!effective_firecrawl_store_in_cache(false, &config));
        assert!(!effective_firecrawl_store_in_cache(true, &config));
    }

    #[test]
    fn privacy_classifier_blocks_private_external_targets() {
        assert_eq!(
            classify_privacy("https://example.com/docs"),
            PrivacyClass::Public
        );
        assert_eq!(
            classify_privacy("http://localhost:3000/dashboard"),
            PrivacyClass::PrivateOrAuthenticated
        );
        assert_eq!(
            classify_privacy("https://example.com/file?token=secret"),
            PrivacyClass::PrivateOrAuthenticated
        );
        assert_eq!(
            classify_privacy("https://user:token@example.com/docs"),
            PrivacyClass::PrivateOrAuthenticated
        );
        assert_eq!(
            classify_privacy("https://10.0.0.12/internal"),
            PrivacyClass::PrivateOrAuthenticated
        );
        assert_eq!(
            classify_privacy("https://[fd00::1]/internal"),
            PrivacyClass::PrivateOrAuthenticated
        );
        assert_eq!(
            classify_privacy("https://[::ffff:127.0.0.1]/internal"),
            PrivacyClass::PrivateOrAuthenticated
        );
        assert!(
            enforce_external_privacy(
                PrivacyClass::PrivateOrAuthenticated,
                false,
                &ResearchConfig::default(),
                "firecrawl"
            )
            .is_err()
        );
    }

    #[test]
    fn route_memory_records_and_lists_domain_preferences() -> Result<()> {
        let dir = temp_path("route-memory");
        let paths = ResearchPaths {
            database: dir.join("research.sqlite"),
            blobs_dir: dir.join("blobs"),
            cache_dir: dir,
        };
        init_db(&paths)?;
        record_route_memory(
            &paths,
            "https://docs.example.com/app",
            Route::AgentBrowser,
            true,
            Some(200),
            "rendered route worked",
        )?;
        record_route_memory(
            &paths,
            "https://docs.example.com/app",
            Route::Firecrawl,
            false,
            Some(500),
            "crawl route failed",
        )?;

        let hit = route_memory_for_url(&paths, "https://docs.example.com/other")?
            .expect("route memory should exist");
        assert_eq!(hit.domain, "docs.example.com");
        assert_eq!(hit.preferred_route, "agent-browser");
        assert_eq!(hit.successes, 1);
        assert_eq!(hit.failures, 1);
        fs::remove_dir_all(paths.cache_dir)?;
        Ok(())
    }

    #[test]
    fn metadata_redaction_uses_privacy_config() {
        let config = ResearchConfig::default();
        assert_eq!(
            metadata_text("find api_key=secret usage", &config),
            "[redacted]"
        );

        let config = ResearchConfig {
            privacy: PrivacyConfig {
                redact_query_secrets: false,
                ..PrivacyConfig::default()
            },
            ..ResearchConfig::default()
        };
        assert_eq!(
            metadata_text("find api_key=secret usage", &config),
            "find api_key=secret usage"
        );
    }

    #[test]
    fn source_cache_redacts_secret_query_params() -> Result<()> {
        let dir = temp_path("cache-redaction");
        let paths = ResearchPaths {
            cache_dir: dir.clone(),
            database: dir.join("research.sqlite"),
            blobs_dir: dir.join("blobs"),
        };
        init_db(&paths)?;

        let source_id = record_source_cache(
            &paths,
            SourceCacheInsert {
                url: "https://example.com/doc?token=secret&page=2",
                provider: "direct",
                status: Some(200),
                content_hash: None,
                route: Some("direct"),
                title: None,
                canonical_url: Some("https://example.com/doc?api_key=secret"),
                freshness_status: "current",
                privacy_classification: "private-or-authenticated",
                raw_body_stored: false,
                metadata: json!({ "next_url": "https://example.com/next?password=secret" }),
                redact_query_secrets: true,
            },
        )?;
        let record = cached_source(&paths, &source_id)?.expect("source should exist");

        assert_eq!(
            record.url,
            "https://example.com/doc?token=%5Bredacted%5D&page=2"
        );
        assert_eq!(
            record.canonical_url.as_deref(),
            Some("https://example.com/doc?api_key=%5Bredacted%5D")
        );
        assert_eq!(
            record.metadata["next_url"],
            "https://example.com/next?password=%5Bredacted%5D"
        );
        fs::remove_dir_all(paths.cache_dir)?;
        Ok(())
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
