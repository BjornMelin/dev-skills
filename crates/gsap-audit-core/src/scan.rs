//! Filesystem walking and per-file orchestration.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::analyze::analyze_source;
use crate::source::source_type_for_extension;
use crate::types::{Category, Finding};

/// File extensions that gsap-audit will parse.
const INCLUDED_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"];

/// Directory names that are always skipped during the walk.
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    ".next",
    "out",
    "target",
    "coverage",
    ".turbo",
];

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
    /// Number of candidate source files analyzed.
    pub files_scanned: usize,
    /// Whether the walk hit `max_files` and stopped early.
    pub truncated: bool,
}

/// Walk `options.root` and analyze every matching source file.
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

    let walker = WalkDir::new(root).into_iter().filter_entry(|entry| {
        // Skip well-known build/vendor directories by name.
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

        let relative = relative_path(root, path);
        let source_type = source_type_for_extension(&extension);
        let mut file_findings = analyze_source(&relative, &source, source_type);

        if !include_all {
            file_findings.retain(|finding| options.categories.contains(&finding.category));
        }
        outcome.findings.extend(file_findings);
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

/// Analyze a single file path (used by callers that already have a path).
///
/// Returns `Ok(None)` if the extension is not a supported source extension.
pub fn scan_file(root: &Path, path: &Path) -> Result<Option<Vec<Finding>>> {
    let Some(extension) = file_extension(path) else {
        return Ok(None);
    };
    if !INCLUDED_EXTENSIONS.contains(&extension.as_str()) {
        return Ok(None);
    }
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let relative = relative_path(root, path);
    let source_type = source_type_for_extension(&extension);
    Ok(Some(analyze_source(&relative, &source, source_type)))
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
