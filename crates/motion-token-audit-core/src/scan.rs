//! Filesystem walking and per-file orchestration.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

use crate::analyze::{
    MotionTokens, analyze_css, analyze_source, discover_css_tokens, discover_tokens,
    empty_coverage, merge_coverage,
};
use crate::rules::ids;
use crate::source::source_type_for_extension;
use crate::types::{Category, Confidence, Coverage, Finding, Severity};

const JS_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"];
const CSS_EXTENSIONS: &[&str] = &["css", "scss"];
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

#[derive(Clone, Debug)]
pub struct ScanOptions {
    pub root: PathBuf,
    pub categories: BTreeSet<Category>,
    pub max_files: usize,
}

impl ScanOptions {
    #[must_use]
    pub fn new(root: PathBuf, categories: BTreeSet<Category>, max_files: usize) -> ScanOptions {
        ScanOptions {
            root,
            categories,
            max_files,
        }
    }
}

#[derive(Clone, Debug)]
struct SourceFile {
    path: PathBuf,
    relative: String,
    extension: String,
}

#[derive(Clone, Debug, Default)]
pub struct ScanOutcome {
    pub findings: Vec<Finding>,
    pub coverage: Vec<Coverage>,
    pub files_scanned: usize,
    pub truncated: bool,
}

pub fn scan_root(options: &ScanOptions) -> Result<ScanOutcome> {
    let root = options.root.as_path();
    anyhow::ensure!(
        root.exists(),
        "scan root does not exist: {}",
        root.display()
    );

    let (files, truncated) = collect_files(root, options.max_files);
    let mut tokens = MotionTokens::default();
    for file in &files {
        let source = match std::fs::read_to_string(&file.path) {
            Ok(source) => source,
            Err(_) => continue,
        };
        if CSS_EXTENSIONS.contains(&file.extension.as_str()) {
            tokens.merge(discover_css_tokens(&source));
        } else {
            let source_type = source_type_for_extension(&file.extension);
            tokens.merge(discover_tokens(&source, source_type));
        }
    }

    let mut outcome = ScanOutcome {
        coverage: empty_coverage(),
        files_scanned: files.len(),
        truncated,
        ..ScanOutcome::default()
    };
    if tokens.is_empty() {
        outcome.findings.push(Finding {
            id: ids::SSOT_NO_TOKEN_MODULE.to_string(),
            category: Category::Ssot,
            severity: Severity::Low,
            confidence: Confidence::High,
            file: ".".to_string(),
            line: 1,
            column: 1,
            message: "No motion token module or CSS custom-property SSOT was found.".to_string(),
            suggestion: "Add shared motion duration/easing/spring tokens before auditing drift."
                .to_string(),
        });
    }

    for file in files {
        let source = match std::fs::read_to_string(&file.path) {
            Ok(source) => source,
            Err(_) => continue,
        };
        let analysis = if CSS_EXTENSIONS.contains(&file.extension.as_str()) {
            analyze_css(&file.relative, &source, &tokens)
        } else {
            analyze_source(
                &file.relative,
                &source,
                source_type_for_extension(&file.extension),
                &tokens,
            )
        };
        merge_coverage(&mut outcome.coverage, &analysis.coverage);
        outcome.findings.extend(analysis.findings);
    }

    if !options.categories.is_empty() {
        outcome
            .findings
            .retain(|finding| options.categories.contains(&finding.category));
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

fn collect_files(root: &Path, max_files: usize) -> (Vec<SourceFile>, bool) {
    let mut files = Vec::new();
    let mut truncated = false;
    let walker = WalkDir::new(root).into_iter().filter_entry(|entry| {
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
        if !JS_EXTENSIONS.contains(&extension.as_str())
            && !CSS_EXTENSIONS.contains(&extension.as_str())
        {
            continue;
        }
        if files.len() >= max_files {
            truncated = true;
            break;
        }
        files.push(SourceFile {
            path: path.to_path_buf(),
            relative: relative_path(root, path),
            extension,
        });
    }
    (files, truncated)
}

fn file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
}

fn relative_path(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let mut text = relative.to_string_lossy().replace('\\', "/");
    if text.is_empty() {
        text = path.to_string_lossy().replace('\\', "/");
    }
    text
}
