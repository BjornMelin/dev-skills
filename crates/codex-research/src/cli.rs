use crate::*;

#[derive(Parser)]
#[command(name = "codex-research")]
#[command(about = "Evidence-first research helper for Codex skills and subagents")]
pub(crate) struct Cli {
    #[arg(
        long,
        global = true,
        help = "Emit machine-readable JSON when supported"
    )]
    pub(crate) json: bool,

    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Load codex-research TOML config from an explicit path"
    )]
    pub(crate) config: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Generate shell completions for local installation.
    Completions { shell: Shell },
    /// Generate a roff manpage for local installation.
    Manpage,
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
    /// Build a closeout evidence bundle from run, ledger, cache, and report state.
    Bundle(BundleArgs),
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
pub(crate) struct PlanArgs {
    pub(crate) query: String,
    #[arg(long, value_enum, default_value_t = ResearchProfile::Standard)]
    pub(crate) profile: ResearchProfile,
}

#[derive(Args)]
pub(crate) struct SearchArgs {
    pub(crate) query: String,
    #[arg(long, value_enum, default_value_t = ResearchProfile::Standard)]
    pub(crate) profile: ResearchProfile,
    #[arg(long, value_enum, default_value_t = TopicKind::General)]
    pub(crate) topic: TopicKind,
}

#[derive(Args, Clone, Debug, Default)]
pub(crate) struct BudgetArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Debit this research run before calling a provider"
    )]
    pub(crate) run: Option<PathBuf>,
    #[arg(long, help = "Skip run-budget debit even when --run is provided")]
    pub(crate) no_budget: bool,
}

#[derive(Subcommand)]
pub(crate) enum FetchCommand {
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
pub(crate) enum Context7Command {
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
pub(crate) enum GithubCommand {
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
pub(crate) enum LedgerCommand {
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
pub(crate) struct AddSourceArgs {
    #[arg(long, default_value = ".codex/research/ledger.jsonl")]
    pub(crate) ledger: PathBuf,
    #[arg(long = "from-cache", value_name = "SOURCE_ID")]
    pub(crate) from_cache: Option<String>,
    #[arg(long)]
    pub(crate) provider: Option<String>,
    #[arg(long)]
    pub(crate) url: Option<String>,
    #[arg(long)]
    pub(crate) title: Option<String>,
    #[arg(long)]
    pub(crate) route: Option<String>,
}

#[derive(Args)]
pub(crate) struct AddClaimArgs {
    #[arg(long, default_value = ".codex/research/ledger.jsonl")]
    pub(crate) ledger: PathBuf,
    #[arg(long)]
    pub(crate) text: String,
    #[arg(long, default_value_t = 0.75)]
    pub(crate) confidence: f32,
    #[arg(long = "source")]
    pub(crate) sources: Vec<String>,
    #[arg(long)]
    pub(crate) note: Option<String>,
}

#[derive(Args)]
pub(crate) struct ReportArgs {
    #[arg(long, default_value = ".codex/research/ledger.jsonl")]
    pub(crate) ledger: PathBuf,
    #[arg(long)]
    pub(crate) out: Option<PathBuf>,
}

#[derive(Args)]
pub(crate) struct BundleArgs {
    #[arg(long, value_name = "PATH", default_value = ".codex/research/run.json")]
    pub(crate) run: PathBuf,
    #[arg(
        long,
        value_name = "PATH",
        default_value = ".codex/research/ledger.jsonl"
    )]
    pub(crate) ledger: PathBuf,
    #[arg(long, value_name = "PATH", default_value = ".codex/research/report.md")]
    pub(crate) report: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub(crate) out: Option<PathBuf>,
    #[arg(long = "markdown-out", value_name = "PATH")]
    pub(crate) markdown_out: Option<PathBuf>,
    #[arg(long, value_name = "RFC3339")]
    pub(crate) generated_at: Option<DateTime<Utc>>,
    #[arg(
        long,
        help = "Exit nonzero when citation, provider, report, or ledger evidence is incomplete"
    )]
    pub(crate) strict: bool,
}

#[derive(Subcommand)]
pub(crate) enum CacheCommand {
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
pub(crate) enum ConfigCommand {
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
pub(crate) enum RunCommand {
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
pub(crate) struct EvalArgs {
    #[arg(long)]
    pub(crate) live: bool,
    #[arg(long, value_name = "PATH", help = "Load an eval suite JSON file")]
    pub(crate) suite: Option<PathBuf>,
    #[arg(long, value_name = "ID", help = "Run only the selected task ID")]
    pub(crate) task: Vec<String>,
    #[arg(long, help = "List eval tasks without running them")]
    pub(crate) list: bool,
    #[arg(long, help = "Treat eval warnings as failures")]
    pub(crate) strict: bool,
}
