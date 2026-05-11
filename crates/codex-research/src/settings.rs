use crate::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ResearchConfig {
    #[serde(default)]
    pub(crate) profiles: ProfilesConfig,
    #[serde(default)]
    pub(crate) privacy: PrivacyConfig,
    #[serde(default)]
    pub(crate) providers: ProvidersConfig,
    #[serde(default)]
    pub(crate) cache: CacheConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ProfilesConfig {
    #[serde(default = "quick_budget")]
    pub(crate) quick: ProviderBudgets,
    #[serde(default = "standard_budget")]
    pub(crate) standard: ProviderBudgets,
    #[serde(default = "deep_budget")]
    pub(crate) deep: ProviderBudgets,
    #[serde(default = "exhaustive_budget")]
    pub(crate) exhaustive: ProviderBudgets,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PrivacyConfig {
    #[serde(default = "deny_string")]
    pub(crate) private_external_default: String,
    #[serde(default = "deny_string")]
    pub(crate) ambiguous_external_default: String,
    #[serde(default)]
    pub(crate) allow_private_external: bool,
    #[serde(default = "default_true")]
    pub(crate) redact_query_secrets: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ProvidersConfig {
    #[serde(default)]
    pub(crate) github: GithubProviderConfig,
    #[serde(default)]
    pub(crate) context7: Context7ProviderConfig,
    #[serde(default)]
    pub(crate) firecrawl: FirecrawlProviderConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct GithubProviderConfig {
    #[serde(default = "default_github_per_page")]
    pub(crate) per_page_default: u8,
    #[serde(default = "default_github_per_page_max")]
    pub(crate) per_page_max: u8,
    #[serde(default = "default_backoff_retries")]
    pub(crate) backoff_retries: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Context7ProviderConfig {
    #[serde(default = "default_cache_ttl_hours")]
    pub(crate) cache_ttl_hours: u32,
    #[serde(default = "default_true")]
    pub(crate) prefer_version_pinned_ids: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct FirecrawlProviderConfig {
    #[serde(default = "default_firecrawl_max_age_ms")]
    pub(crate) default_max_age_ms: u64,
    #[serde(default)]
    pub(crate) latest_critical_max_age_ms: u64,
    #[serde(default = "default_true")]
    pub(crate) store_in_cache_default: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct CacheConfig {
    #[serde(default = "default_cache_ttl_hours")]
    pub(crate) source_metadata_ttl_hours: u32,
    #[serde(default)]
    pub(crate) store_raw_external_default: bool,
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

pub(crate) struct ResearchPaths {
    pub(crate) cache_dir: PathBuf,
    pub(crate) database: PathBuf,
    pub(crate) blobs_dir: PathBuf,
}

pub(crate) fn research_paths() -> Result<ResearchPaths> {
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

pub(crate) struct LoadedConfig {
    pub(crate) path: Option<PathBuf>,
    pub(crate) config: ResearchConfig,
}

pub(crate) fn load_config(explicit: Option<&Path>) -> Result<LoadedConfig> {
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

pub(crate) fn find_nearest_config() -> Option<PathBuf> {
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

pub(crate) fn default_config_toml() -> Result<String> {
    Ok(toml::to_string_pretty(&ResearchConfig::default())?)
}

pub(crate) fn quick_budget() -> ProviderBudgets {
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

pub(crate) fn standard_budget() -> ProviderBudgets {
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

pub(crate) fn deep_budget() -> ProviderBudgets {
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

pub(crate) fn exhaustive_budget() -> ProviderBudgets {
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

pub(crate) fn profile_budget(config: &ResearchConfig, profile: ResearchProfile) -> ProviderBudgets {
    match profile {
        ResearchProfile::Quick => config.profiles.quick.clone(),
        ResearchProfile::Standard => config.profiles.standard.clone(),
        ResearchProfile::Deep => config.profiles.deep.clone(),
        ResearchProfile::Exhaustive => config.profiles.exhaustive.clone(),
    }
}

pub(crate) fn deny_string() -> String {
    "deny".to_string()
}

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_github_per_page() -> u8 {
    10
}

pub(crate) fn default_github_per_page_max() -> u8 {
    100
}

pub(crate) fn default_backoff_retries() -> u8 {
    2
}

pub(crate) fn default_cache_ttl_hours() -> u32 {
    168
}

pub(crate) fn default_firecrawl_max_age_ms() -> u64 {
    172_800_000
}

pub(crate) fn effective_firecrawl_store_in_cache(
    no_store_in_cache: bool,
    config: &ResearchConfig,
) -> bool {
    config.providers.firecrawl.store_in_cache_default && !no_store_in_cache
}
