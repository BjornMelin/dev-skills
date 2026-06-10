use crate::types::{Finding, PlannedFix, Severity};

pub fn format_findings_text(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "OK: no Bun platform audit findings.\n".to_string();
    }

    let mut out = String::new();
    for finding in findings {
        let location = format!("{}:{}:{}", finding.file, finding.line, finding.column);
        out.push_str(&format!(
            "{:<5} {} {}\n",
            finding.severity.as_upper(),
            finding.rule_id,
            location
        ));
        out.push_str(&format!("  {}\n", finding.message));
        if let Some(why) = &finding.why {
            out.push_str(&format!("  Why: {why}\n"));
        }
        if let Some(fix) = &finding.suggested_fix {
            out.push_str(&format!("  Fix: {fix}\n"));
        }
        if let Some(snippet) = &finding.snippet {
            out.push_str(&format!("  {snippet}\n"));
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
        out.push_str(&format!(
            "- **{}** `{}` ({}): {}\n",
            finding.severity.as_upper(),
            finding.rule_id,
            location,
            finding.message
        ));
        if let Some(fix) = &finding.suggested_fix {
            out.push_str(&format!("  - Fix: {fix}\n"));
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
        out.push_str(&format!(
            "- [{:?}] {} {}\n",
            fix.kind, fix.rule_id, fix.file
        ));
        out.push_str(&format!("  {}\n", fix.description));
    }
    out
}

pub fn should_fail(findings: &[Finding], fail_on: Severity) -> bool {
    findings
        .iter()
        .any(|finding| finding.severity.rank() >= fail_on.rank())
}
