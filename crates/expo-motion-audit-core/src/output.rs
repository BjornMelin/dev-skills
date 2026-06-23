//! Output formatting for findings and the rule catalog.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::Serialize;
use serde_json::{Value, json};

use crate::rules::CATALOG;
use crate::types::{Category, Finding, Severity};

/// Highest severity among a set of findings, if any.
#[must_use]
pub fn highest_severity(findings: &[Finding]) -> Option<Severity> {
    findings.iter().map(|finding| finding.severity).max()
}

/// Counts of findings by severity and by category, for the summary block.
#[derive(Debug, Default, Serialize)]
pub struct Summary {
    /// Total number of findings.
    pub total: usize,
    /// Count per severity (lowercase keys: low/medium/high).
    pub by_severity: BTreeMap<String, usize>,
    /// Count per category (kebab-case keys).
    pub by_category: BTreeMap<String, usize>,
}

/// Build a summary from findings.
#[must_use]
pub fn summarize(findings: &[Finding]) -> Summary {
    let mut summary = Summary {
        total: findings.len(),
        ..Summary::default()
    };
    for finding in findings {
        *summary
            .by_severity
            .entry(finding.severity.as_str().to_string())
            .or_insert(0) += 1;
        *summary
            .by_category
            .entry(finding.category.as_str().to_string())
            .or_insert(0) += 1;
    }
    summary
}

/// Render findings as the stable JSON output payload.
#[must_use]
pub fn format_json(tool: &str, version: &str, findings: &[Finding]) -> Value {
    let summary = summarize(findings);
    let findings_json: Vec<Value> = findings
        .iter()
        .map(|finding| {
            json!({
                "id": finding.id,
                "category": finding.category.as_str(),
                "severity": finding.severity.as_str(),
                "confidence": finding.confidence.as_str(),
                "file": finding.file,
                "line": finding.line,
                "column": finding.column,
                "message": finding.message,
                "suggestion": finding.suggestion,
            })
        })
        .collect();
    json!({
        "tool": tool,
        "version": version,
        "summary": {
            "total": summary.total,
            "by_severity": summary.by_severity,
            "by_category": summary.by_category,
        },
        "findings": findings_json,
    })
}

/// Render findings as markdown, grouped by file with a trailing summary.
#[must_use]
pub fn format_markdown(tool: &str, version: &str, findings: &[Finding]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {tool} report (v{version})");
    let _ = writeln!(out);

    if findings.is_empty() {
        let _ = writeln!(out, "No findings.");
        let _ = writeln!(out);
    } else {
        // Group by file, preserving sorted order.
        let mut current_file: Option<&str> = None;
        for finding in findings {
            if current_file != Some(finding.file.as_str()) {
                if current_file.is_some() {
                    let _ = writeln!(out);
                }
                let _ = writeln!(out, "## {}", finding.file);
                current_file = Some(finding.file.as_str());
            }
            let _ = writeln!(
                out,
                "- {}:{} [{}] {} — {}",
                finding.line, finding.column, finding.severity, finding.id, finding.message
            );
            let _ = writeln!(out, "  - suggestion: {}", finding.suggestion);
        }
        let _ = writeln!(out);
    }

    let summary = summarize(findings);
    let _ = writeln!(out, "## Summary");
    let _ = writeln!(out, "- total: {}", summary.total);
    let _ = writeln!(
        out,
        "- by severity: {}",
        format_counts(&summary.by_severity)
    );
    let _ = writeln!(
        out,
        "- by category: {}",
        format_counts(&summary.by_category)
    );
    out
}

/// Render the rule catalog as markdown (for `doctor`).
#[must_use]
pub fn format_catalog_markdown(tool: &str, version: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {tool} rule catalog (v{version})");
    let _ = writeln!(out);
    let _ = writeln!(out, "| id | category | severity |");
    let _ = writeln!(out, "| --- | --- | --- |");
    for rule in CATALOG {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            rule.id, rule.category, rule.severity
        );
    }
    out
}

/// Render the rule catalog as JSON (for `doctor`).
#[must_use]
pub fn format_catalog_json(tool: &str, version: &str) -> Value {
    let rules: Vec<Value> = CATALOG
        .iter()
        .map(|rule| {
            json!({
                "id": rule.id,
                "category": rule.category.as_str(),
                "severity": rule.severity.as_str(),
                "confidence": rule.confidence.as_str(),
                "summary": rule.summary,
            })
        })
        .collect();
    json!({
        "tool": tool,
        "version": version,
        "rules": rules,
    })
}

/// Format an ordered count map as `key=count` pairs.
fn format_counts(counts: &BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }
    counts
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Convenience: total count for a category in a set of findings.
#[must_use]
pub fn count_category(findings: &[Finding], category: Category) -> usize {
    findings
        .iter()
        .filter(|finding| finding.category == category)
        .count()
}
