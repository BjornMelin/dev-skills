use std::fmt::Write as _;

use crate::types::{Finding, PlannedFix, Severity};

pub fn format_findings_text(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "OK: no Bun platform audit findings.\n".to_string();
    }

    let mut out = String::new();
    for finding in findings {
        let location = format!("{}:{}:{}", finding.file, finding.line, finding.column);
        let _ = writeln!(
            out,
            "{:<5} {} {}",
            finding.severity.as_upper(),
            finding.rule_id,
            location
        );
        let _ = writeln!(out, "  {}", finding.message);
        if let Some(why) = &finding.why {
            let _ = writeln!(out, "  Why: {why}");
        }
        if let Some(fix) = &finding.suggested_fix {
            let _ = writeln!(out, "  Fix: {fix}");
        }
        if let Some(snippet) = &finding.snippet {
            let _ = writeln!(out, "  {snippet}");
        }
    }
    out
}

pub fn format_findings_md(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "OK: no Bun platform audit findings.\n".to_string();
    }
    let mut out = String::from("# Bun Platform Audit Findings\n\n");
    for finding in findings {
        let location = format!("{}:{}:{}", finding.file, finding.line, finding.column);
        let _ = writeln!(
            out,
            "- **{}** `{}` ({}): {}",
            finding.severity.as_upper(),
            finding.rule_id,
            location,
            finding.message
        );
        if let Some(fix) = &finding.suggested_fix {
            let _ = writeln!(out, "  - Fix: {fix}");
        }
    }
    out
}

pub fn format_fixes_text(fixes: &[PlannedFix], applied: bool) -> String {
    if fixes.is_empty() {
        return if applied {
            "OK: no safe fixes were applicable.\n".to_string()
        } else {
            "OK: no safe fixes were planned.\n".to_string()
        };
    }

    let mut out = if applied {
        format!("Applied {} safe fix(es):\n", fixes.len())
    } else {
        format!("Planned {} safe fix(es):\n", fixes.len())
    };
    for fix in fixes {
        let _ = writeln!(out, "- [{:?}] {} {}", fix.kind, fix.rule_id, fix.file);
        let _ = writeln!(out, "  {}", fix.description);
    }
    out
}

pub fn should_fail(findings: &[Finding], fail_on: Severity) -> bool {
    findings
        .iter()
        .any(|finding| finding.severity.rank() >= fail_on.rank())
}
