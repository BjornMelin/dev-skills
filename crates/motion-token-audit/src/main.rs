//! motion-token-audit: static auditor CLI for cross-stack motion token drift.

use std::collections::BTreeSet;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use audit_gate::{Baseline, GateFinding, GateSeverity, is_excluded, to_sarif};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use motion_token_audit_core::output::{format_catalog_json, format_catalog_markdown};
use motion_token_audit_core::{
    Category, ScanOptions, Severity, TOOL_NAME, TOOL_VERSION, format_json, format_markdown,
    scan_root,
};

#[derive(Parser, Debug)]
#[command(
    name = "motion-token-audit",
    version,
    about = "Statically audit cross-stack motion token drift.",
    long_about = "motion-token-audit discovers shared motion duration/easing/spring tokens, then checks CSS, Reanimated, GSAP, and Motion React code for hardcoded motion literals that drift from or bypass the token vocabulary.",
    propagate_version = true,
    after_long_help = "Examples:\n  motion-token-audit scan --root . --format markdown\n  motion-token-audit scan --root ./src --format json --categories tokens-css,tokens-reanimated\n  motion-token-audit doctor --format json\n  motion-token-audit completions zsh"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(
        about = "Scan a directory tree for motion token drift.",
        long_about = "Walk the given root, discover motion tokens, parse supported source files, and report drift/orphan findings. Exit code is 2 when any finding meets --min-severity (default medium), otherwise 0.",
        after_long_help = "Example:\n  motion-token-audit scan --root . --format json --categories tokens-css,tokens-reanimated"
    )]
    Scan {
        #[arg(
            long,
            value_name = "PATH",
            default_value = ".",
            help = "Directory to scan."
        )]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Markdown, help = "Output format.")]
        format: OutputFormat,
        #[arg(
            long,
            value_name = "CSV",
            help = "Comma-separated subset of: ssot,tokens-css,tokens-reanimated,tokens-gsap,tokens-react,tokens-r3f. Default = all."
        )]
        categories: Option<String>,
        #[arg(
            long,
            value_name = "PATH",
            help = "Write output to this file instead of stdout."
        )]
        output: Option<PathBuf>,
        #[arg(
            long = "max-files",
            value_name = "N",
            default_value_t = 5000,
            help = "Maximum number of files to analyze before truncating."
        )]
        max_files: usize,
        #[arg(
            long = "min-severity",
            value_enum,
            default_value_t = MinSeverity::Medium,
            help = "Lowest severity that makes the exit code non-zero."
        )]
        min_severity: MinSeverity,
        #[arg(
            long = "exclude",
            value_name = "GLOB",
            help = "Skip findings whose path matches this glob. Repeatable. Supports *, ** and ?."
        )]
        exclude: Vec<String>,
        #[arg(
            long = "baseline",
            value_name = "PATH",
            help = "Report only findings absent from this baseline file."
        )]
        baseline: Option<PathBuf>,
        #[arg(
            long = "write-baseline",
            value_name = "PATH",
            help = "Write the current findings to a baseline file and exit 0."
        )]
        write_baseline: Option<PathBuf>,
    },
    #[command(
        about = "Print the tool version and the full rule catalog.",
        long_about = "Print the tool name and version plus every rule (id, category, severity) as markdown or JSON.",
        after_long_help = "Example:\n  motion-token-audit doctor --format json"
    )]
    Doctor {
        #[arg(long, value_enum, default_value_t = OutputFormat::Markdown, help = "Output format.")]
        format: OutputFormat,
    },
    #[command(
        about = "Generate shell completions.",
        long_about = "Print a shell completion script for the requested shell.",
        after_long_help = "Example:\n  motion-token-audit completions zsh"
    )]
    Completions {
        #[arg(value_enum, help = "Shell to generate completions for.")]
        shell: Shell,
    },
}

/// `--min-severity` threshold. Mirrors the core `Severity` ordering; the core
/// type is not a `ValueEnum`, and making it one would put a CLI concern in a
/// library that has no other clap dependency.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, ValueEnum)]
enum MinSeverity {
    Low,
    Medium,
    High,
}

impl MinSeverity {
    /// Whether a finding at `severity` meets this threshold.
    fn is_met_by(self, severity: Severity) -> bool {
        let rank = |value: MinSeverity| match value {
            MinSeverity::Low => 0,
            MinSeverity::Medium => 1,
            MinSeverity::High => 2,
        };
        let found = match severity {
            Severity::Low => 0,
            Severity::Medium => 1,
            Severity::High => 2,
        };
        found >= rank(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Markdown,
    Json,
    /// SARIF 2.1.0, for rendering findings as annotations on a diff.
    Sarif,
}

fn main() {
    match run() {
        Ok(code) => process::exit(code),
        Err(error) => {
            eprintln!("{error:#}");
            process::exit(1);
        }
    }
}

fn run() -> Result<i32> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan {
            root,
            format,
            categories,
            output,
            max_files,
            min_severity,
            exclude,
            baseline,
            write_baseline,
        } => run_scan(ScanRequest {
            root,
            format,
            categories: categories.as_deref(),
            output,
            max_files,
            min_severity,
            exclude: &exclude,
            baseline: baseline.as_deref(),
            write_baseline: write_baseline.as_deref(),
        }),
        Commands::Doctor { format } => {
            let text = match format {
                OutputFormat::Markdown => format_catalog_markdown(TOOL_NAME, TOOL_VERSION),
                OutputFormat::Json => {
                    let value = format_catalog_json(TOOL_NAME, TOOL_VERSION);
                    serde_json::to_string_pretty(&value)?
                }
                // SARIF describes findings in a scan; a rule catalog is not a
                // result set, so refuse rather than emit an empty run.
                OutputFormat::Sarif => {
                    anyhow::bail!("--format sarif applies to `scan`, not `doctor`")
                }
            };
            print_line(&text)?;
            Ok(0)
        }
        Commands::Completions { shell } => {
            let mut command = Cli::command();
            generate(shell, &mut command, "motion-token-audit", &mut io::stdout());
            Ok(0)
        }
    }
}

/// Everything `scan` needs, grouped so the signature stays readable as the
/// gate layer grows.
struct ScanRequest<'a> {
    root: PathBuf,
    format: OutputFormat,
    categories: Option<&'a str>,
    output: Option<PathBuf>,
    max_files: usize,
    min_severity: MinSeverity,
    exclude: &'a [String],
    baseline: Option<&'a std::path::Path>,
    write_baseline: Option<&'a std::path::Path>,
}

fn run_scan(request: ScanRequest<'_>) -> Result<i32> {
    let ScanRequest {
        root,
        format,
        categories,
        output,
        max_files,
        min_severity,
        exclude,
        baseline,
        write_baseline,
    } = request;
    let categories = parse_categories(categories)?;
    let options = ScanOptions::new(root, categories, max_files);
    let mut outcome = scan_root(&options)?;

    // Exclusion runs before everything else: a vendored tree should not decide
    // the exit code, appear in the report, or land in a baseline.
    if !exclude.is_empty() {
        outcome
            .findings
            .retain(|finding| !is_excluded(&finding.file, exclude));
    }

    if let Some(path) = write_baseline {
        Baseline::from_findings(&to_gate_findings(&outcome.findings)).save(path)?;
        return Ok(0);
    }

    // A baseline suppresses known findings from the report as well as from the
    // exit code. Leaving them in a report that claims to be gated would make
    // "clean" mean two different things in the same output.
    if let Some(path) = baseline {
        let known = Baseline::load(path)?;
        let gate = to_gate_findings(&outcome.findings);
        let keep: Vec<bool> = gate.iter().map(|f| !known.contains(f)).collect();
        let mut index = 0;
        outcome.findings.retain(|_| {
            let keep_this = keep[index];
            index += 1;
            keep_this
        });
    }

    let rendered = match format {
        OutputFormat::Markdown => {
            let mut text = format_markdown(
                TOOL_NAME,
                TOOL_VERSION,
                &outcome.findings,
                &outcome.coverage,
            );
            if outcome.truncated {
                text.push_str(&format!(
                    "\nLimitation: file walk truncated at {} files; some files were not analyzed.\n",
                    outcome.files_scanned
                ));
            }
            text
        }
        OutputFormat::Sarif => serde_json::to_string_pretty(&to_sarif(
            TOOL_NAME,
            TOOL_VERSION,
            &to_gate_findings(&outcome.findings),
        ))?,
        OutputFormat::Json => {
            let mut value = format_json(
                TOOL_NAME,
                TOOL_VERSION,
                &outcome.findings,
                &outcome.coverage,
            );
            if let Some(object) = value.as_object_mut() {
                object.insert(
                    "files_scanned".to_string(),
                    serde_json::json!(outcome.files_scanned),
                );
                object.insert(
                    "truncated".to_string(),
                    serde_json::json!(outcome.truncated),
                );
            }
            serde_json::to_string_pretty(&value)?
        }
    };

    match output {
        Some(path) => {
            std::fs::write(&path, format!("{rendered}\n"))
                .with_context(|| format!("failed to write output to {}", path.display()))?;
        }
        None => print_line(&rendered)?,
    }

    // Exit-code contract: 2 when any finding meets --min-severity (default
    // medium), else 0. Configurable so a repo can ratchet: block on `high`
    // today and tighten to `low` once clean.
    let code = if outcome
        .findings
        .iter()
        .any(|finding| min_severity.is_met_by(finding.severity))
    {
        2
    } else {
        0
    };
    Ok(code)
}

/// Convert core findings into the shared gate shape.
///
/// The cores keep their own severity enums; this is the one place the mapping
/// lives, so the gate layer needs no dependency on any parser crate.
fn to_gate_findings(findings: &[motion_token_audit_core::types::Finding]) -> Vec<GateFinding> {
    findings
        .iter()
        .map(|finding| GateFinding {
            id: finding.id.clone(),
            severity: match finding.severity {
                Severity::Low => GateSeverity::Low,
                Severity::Medium => GateSeverity::Medium,
                Severity::High => GateSeverity::High,
            },
            file: finding.file.clone(),
            line: finding.line,
            message: finding.message.clone(),
        })
        .collect()
}

fn parse_categories(value: Option<&str>) -> Result<BTreeSet<Category>> {
    let mut set = BTreeSet::new();
    let Some(value) = value else {
        return Ok(set);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(set);
    }
    for token in trimmed.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let category =
            Category::parse(token).with_context(|| format!("unknown category `{token}`"))?;
        set.insert(category);
    }
    Ok(set)
}

fn print_line(text: &str) -> Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(text.as_bytes())?;
    if !text.ends_with('\n') {
        stdout.write_all(b"\n")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_categories_all_when_empty() {
        assert!(parse_categories(None).unwrap().is_empty());
        assert!(parse_categories(Some("")).unwrap().is_empty());
    }

    #[test]
    fn parse_categories_subset() {
        let set = parse_categories(Some("tokens-css, ssot")).unwrap();
        assert!(set.contains(&Category::TokensCss));
        assert!(set.contains(&Category::Ssot));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn parse_categories_rejects_unknown() {
        assert!(parse_categories(Some("bogus")).is_err());
    }

    #[test]
    fn scan_exits_two_for_medium_findings() {
        let root = temp_scan_root("medium");
        fs::write(
            root.join("motion.ts"),
            "export const motion = { duration: { short: 200 }, easing: { out: [0.16,1,0.3,1] }, spring: { snappy: { stiffness: 520, damping: 42, mass: 1 } } } as const;",
        )
        .unwrap();
        fs::write(root.join("app.ts"), "withTiming(1, { duration: 200 });\n").unwrap();

        let code = run_scan(ScanRequest {
            root: root.clone(),
            format: OutputFormat::Json,
            categories: Some("tokens-reanimated"),
            output: Some(root.join("out.json")),
            max_files: 5000,
            min_severity: MinSeverity::Medium,
            exclude: &[],
            baseline: None,
            write_baseline: None,
        })
        .unwrap();

        assert_eq!(code, 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scan_exits_zero_for_low_only() {
        let root = temp_scan_root("low");
        fs::write(root.join("app.ts"), "withTiming(1, { duration: 237 });\n").unwrap();

        let code = run_scan(ScanRequest {
            root: root.clone(),
            format: OutputFormat::Json,
            categories: Some("tokens-reanimated"),
            output: Some(root.join("out.json")),
            max_files: 5000,
            min_severity: MinSeverity::Medium,
            exclude: &[],
            baseline: None,
            write_baseline: None,
        })
        .unwrap();

        assert_eq!(code, 0);
        fs::remove_dir_all(root).unwrap();
    }

    fn temp_scan_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "motion-token-audit-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn min_severity_ordering_is_monotonic() {
        assert!(MinSeverity::Low.is_met_by(Severity::Low));
        assert!(MinSeverity::Low.is_met_by(Severity::High));
        assert!(!MinSeverity::Medium.is_met_by(Severity::Low));
        assert!(MinSeverity::Medium.is_met_by(Severity::Medium));
        assert!(!MinSeverity::High.is_met_by(Severity::Medium));
        assert!(MinSeverity::High.is_met_by(Severity::High));
    }
}
