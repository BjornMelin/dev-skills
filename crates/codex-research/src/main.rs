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
use rusqlite::{Connection, OpenFlags, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use url::Url;

mod bundle;
mod cache;
mod cache_store;
mod cli;
mod config;
mod context7;
mod doctor;
mod eval;
mod fetch;
mod firecrawl;
mod github;
mod ledger;
mod ledger_store;
mod models;
mod plan;
mod privacy;
mod provider_http;
mod routing;
mod run;
mod run_state;
mod settings;
mod utils;

pub(crate) use bundle::*;
pub(crate) use cache::*;
pub(crate) use cache_store::*;
pub(crate) use cli::*;
pub(crate) use config::*;
pub(crate) use context7::*;
pub(crate) use doctor::*;
pub(crate) use eval::*;
pub(crate) use fetch::*;
pub(crate) use firecrawl::*;
pub(crate) use github::*;
pub(crate) use ledger::*;
pub(crate) use ledger_store::*;
pub(crate) use models::*;
pub(crate) use plan::*;
pub(crate) use privacy::*;
pub(crate) use provider_http::*;
pub(crate) use routing::*;
pub(crate) use run::*;
pub(crate) use run_state::*;
pub(crate) use settings::*;
pub(crate) use utils::*;

const GITHUB_API_VERSION: &str = "2026-03-10";
const USER_AGENT_VALUE: &str = "codex-research/0.2";
const DEFAULT_EVAL_SUITE: &str = include_str!("../evals/research/core.json");
const EVIDENCE_BUNDLE_SCHEMA: &str = "codex-research.evidence-bundle.v1";

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
        Commands::Bundle(args) => build_evidence_bundle_command(args, cli.json),
        Commands::Cache { command } => handle_cache(command, cli.json),
        Commands::Config { command } => handle_config(command, loaded_config.path, cli.json),
        Commands::Run { command } => handle_run(command, &config, cli.json),
        Commands::Eval(args) => run_eval(args, &config, cli.json).await,
    }
}

#[cfg(test)]
mod tests;
