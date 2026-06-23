//! Filesystem walking and per-file orchestration.
//!
//! The scan runs in two passes so the New-Architecture config rule can know
//! whether the project actually uses Reanimated:
//! 1. Walk every supported source file, analyze it, and record whether any file
//!    imports `react-native-reanimated`.
//! 2. Analyze the config files found during the walk, passing the
//!    project-uses-Reanimated signal into the app-config rule.
//!
//! Both static (`app.json`, `app.config.json`) and dynamic
//! (`app.config.js`/`.ts`/`.cjs`/`.mjs`) Expo app configs are routed to the
//! app-config analyzer. The dynamic forms cannot be parsed as JSON and emit a
//! `config.unable-to-analyze` advisory rather than being treated as ordinary
//! source.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

use crate::analyze::analyze_source;
use crate::config::{analyze_app_config, analyze_babel_config};
use crate::source::source_type_for_extension;
use crate::types::{Category, Finding};

/// File extensions that expo-motion-audit will parse as source.
const INCLUDED_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"];

/// Directory names that are always skipped during the walk.
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    ".expo",
    "android",
    "ios",
    "target",
    "coverage",
];

/// Babel config file names handled by the config rules.
const BABEL_CONFIG_NAMES: &[&str] = &["babel.config.js", "babel.config.cjs"];

/// App config file names handled by the config rules. The static JSON forms
/// (`app.json`, `app.config.json`) are analyzed directly; the dynamic forms
/// (`app.config.js`/`.ts`/`.cjs`/`.mjs`) route to the same analyzer, where the
/// JSON parse fails and a `config.unable-to-analyze` advisory is emitted.
const APP_CONFIG_NAMES: &[&str] = &[
    "app.json",
    "app.config.json",
    "app.config.js",
    "app.config.ts",
    "app.config.cjs",
    "app.config.mjs",
];

/// Token used to detect whether a source file pulls in Reanimated, for the
/// app-config New-Architecture rule.
const REANIMATED_MODULE: &str = "react-native-reanimated";

/// Options controlling a scan.
#[derive(Clone, Debug)]
pub struct ScanOptions {
    /// Root directory to scan.
    pub root: PathBuf,
    /// Categories to include. Empty means all categories.
    pub categories: BTreeSet<Category>,
    /// Maximum number of candidate files to analyze before truncating.
    pub max_files: usize,
}

impl ScanOptions {
    /// Build options from a root, a category filter, and a file cap.
    #[must_use]
    pub fn new(root: PathBuf, categories: BTreeSet<Category>, max_files: usize) -> ScanOptions {
        ScanOptions {
            root,
            categories,
            max_files,
        }
    }
}

/// Result of a scan: findings plus walk statistics.
#[derive(Clone, Debug, Default)]
pub struct ScanOutcome {
    /// All findings (already category-filtered), sorted by file then position.
    pub findings: Vec<Finding>,
    /// Number of candidate source/config files analyzed.
    pub files_scanned: usize,
    /// Whether the walk hit `max_files` and stopped early.
    pub truncated: bool,
}

/// A config file discovered during the walk.
struct ConfigFile {
    relative: String,
    source: String,
    kind: ConfigKind,
}

#[derive(Clone, Copy)]
enum ConfigKind {
    Babel,
    App,
}

/// Walk `options.root` and analyze every matching source and config file.
///
/// Returns an error only for IO problems reaching the root; individual files
/// that fail to read are skipped rather than aborting the whole scan.
pub fn scan_root(options: &ScanOptions) -> Result<ScanOutcome> {
    let root = options.root.as_path();
    anyhow::ensure!(
        root.exists(),
        "scan root does not exist: {}",
        root.display()
    );

    let mut outcome = ScanOutcome::default();
    let include_all = options.categories.is_empty();
    let mut project_uses_reanimated = false;
    let mut config_files: Vec<ConfigFile> = Vec::new();

    let walker = WalkDir::new(root).into_iter().filter_entry(|entry| {
        // Skip well-known build/vendor/native directories by name.
        if entry.file_type().is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            return !SKIP_DIRS.contains(&name);
        }
        true
    });

    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        // Config files are collected for the second pass.
        if let Some(kind) = config_kind(file_name) {
            if outcome.files_scanned >= options.max_files {
                outcome.truncated = true;
                break;
            }
            let Ok(source) = std::fs::read_to_string(path) else {
                continue;
            };
            outcome.files_scanned += 1;
            config_files.push(ConfigFile {
                relative: relative_path(root, path),
                source,
                kind,
            });
            continue;
        }

        let Some(extension) = file_extension(path) else {
            continue;
        };
        if !INCLUDED_EXTENSIONS.contains(&extension.as_str()) {
            continue;
        }

        if outcome.files_scanned >= options.max_files {
            outcome.truncated = true;
            break;
        }

        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            // Unreadable / non-UTF-8 files are skipped, not fatal.
            Err(_) => continue,
        };
        outcome.files_scanned += 1;

        if source.contains(REANIMATED_MODULE) {
            project_uses_reanimated = true;
        }

        let relative = relative_path(root, path);
        let source_type = source_type_for_extension(&extension);
        let file_findings = analyze_source(&relative, &source, source_type);
        extend_filtered(&mut outcome.findings, file_findings, options, include_all);
    }

    // Second pass: config files (now that we know whether Reanimated is used).
    for config in config_files {
        let findings = match config.kind {
            ConfigKind::Babel => analyze_babel_config(&config.relative, &config.source),
            ConfigKind::App => {
                analyze_app_config(&config.relative, &config.source, project_uses_reanimated)
            }
        };
        extend_filtered(&mut outcome.findings, findings, options, include_all);
    }

    outcome.findings.sort_by(|left, right| {
        (left.file.as_str(), left.line, left.column, left.id.as_str()).cmp(&(
            right.file.as_str(),
            right.line,
            right.column,
            right.id.as_str(),
        ))
    });

    Ok(outcome)
}

/// Extend the accumulator with new findings, applying the category filter.
fn extend_filtered(
    accumulator: &mut Vec<Finding>,
    mut findings: Vec<Finding>,
    options: &ScanOptions,
    include_all: bool,
) {
    if !include_all {
        findings.retain(|finding| options.categories.contains(&finding.category));
    }
    accumulator.extend(findings);
}

/// Map a file name to a config kind, if it is a recognized config file.
fn config_kind(file_name: &str) -> Option<ConfigKind> {
    if BABEL_CONFIG_NAMES.contains(&file_name) {
        Some(ConfigKind::Babel)
    } else if APP_CONFIG_NAMES.contains(&file_name) {
        Some(ConfigKind::App)
    } else {
        None
    }
}

/// Lowercase file extension, if any.
fn file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
}

/// Render a path relative to root using forward slashes, falling back to the
/// full path display when the path is not under root.
fn relative_path(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let mut text = relative.to_string_lossy().replace('\\', "/");
    if text.is_empty() {
        text = path.to_string_lossy().replace('\\', "/");
    }
    text
}
