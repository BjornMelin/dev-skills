use crate::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ResearchProfile {
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
pub(crate) enum TopicKind {
    General,
    Docs,
    Github,
    Dependency,
    Openai,
    Rendered,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Route {
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
pub(crate) enum ProviderKind {
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
pub(crate) enum PrivacyClass {
    Public,
    SensitivePublic,
    PrivateOrAuthenticated,
    Ambiguous,
}

#[derive(Serialize)]
pub(crate) struct DoctorReport {
    pub(crate) cache_dir: PathBuf,
    pub(crate) database: PathBuf,
    pub(crate) blobs_dir: PathBuf,
    pub(crate) env: BTreeMap<&'static str, bool>,
    pub(crate) tools: BTreeMap<&'static str, Option<String>>,
    pub(crate) notes: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct ResearchPlan {
    pub(crate) query: String,
    pub(crate) profile: ResearchProfile,
    pub(crate) budgets: ProviderBudgets,
    pub(crate) route_order: Vec<Route>,
    pub(crate) rules: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ProviderBudgets {
    pub(crate) codex_web_queries: u32,
    pub(crate) context7_calls: u32,
    pub(crate) github_calls: u32,
    pub(crate) exa_calls: u32,
    pub(crate) direct_fetches: u32,
    pub(crate) browser_fetches: u32,
    pub(crate) firecrawl_calls: u32,
}

#[derive(Serialize)]
pub(crate) struct ProbeReport {
    pub(crate) url: String,
    pub(crate) status: Option<u16>,
    pub(crate) content_type: Option<String>,
    pub(crate) content_length: Option<u64>,
    pub(crate) text_chars: usize,
    pub(crate) script_markers: usize,
    pub(crate) app_shell_markers: Vec<String>,
    pub(crate) route: Route,
    pub(crate) reason: String,
    pub(crate) route_memory: Vec<RouteMemoryHit>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RouteMemoryHit {
    pub(crate) domain: String,
    pub(crate) preferred_route: String,
    pub(crate) successes: u32,
    pub(crate) failures: u32,
    pub(crate) updated_at: String,
}
