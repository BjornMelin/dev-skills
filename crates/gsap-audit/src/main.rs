//! gsap-audit: static auditor CLI for GSAP usage anti-patterns.
//!
//! Thin clap-derive front end over `gsap-audit-core`. Exit codes:
//! - 0: no findings, or only low-severity findings.
//! - 2: at least one medium- or high-severity finding.
//! - 1: usage or IO error.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use audit_gate::{Baseline, GateFinding, GateSeverity, to_sarif};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use gsap_audit_core::output::{format_catalog_json, format_catalog_markdown};
use gsap_audit_core::{
    Category, ScanOptions, Severity, TOOL_NAME, TOOL_VERSION, format_json, format_markdown,
    scan_root,
};

#[derive(Parser, Debug)]
#[command(
    name = "gsap-audit",
    version,
    about = "Statically audit GSAP (GreenSock) usage in JS/TS source and report anti-patterns.",
    long_about = "gsap-audit parses JS/TS/JSX/TSX with oxc, runs semantic analysis, and reports GSAP anti-patterns (trial imports, dev-only plugins, ScrollTrigger debug config, GSAP-2 signatures, layout-prop animation, missing plugin registration, React useGSAP/context misuse, and more).",
    propagate_version = true,
    after_long_help = "Examples:\n  gsap-audit scan --root . --format markdown\n  gsap-audit scan --root ./src --format json --categories react,scrolltrigger\n  gsap-audit doctor --format json\n  gsap-audit completions zsh"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(
        about = "Scan a directory tree for GSAP anti-patterns.",
        long_about = "Walk the given root, parse every supported source file, and report findings. Exit code is 2 when any finding meets --min-severity (default medium), otherwise 0.",
        after_long_help = "Example:\n  gsap-audit scan --root . --format json --categories react,core"
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
            help = "Comma-separated subset of: core,react,scrolltrigger,timeline,plugins,performance,utils. Default = all."
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
        after_long_help = "Example:\n  gsap-audit doctor --format json"
    )]
    Doctor {
        #[arg(long, value_enum, default_value_t = OutputFormat::Markdown, help = "Output format.")]
        format: OutputFormat,
    },
    #[command(
        about = "Generate shell completions.",
        long_about = "Print a shell completion script for the requested shell.",
        after_long_help = "Example:\n  gsap-audit completions zsh"
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

/// Run the CLI, returning the intended process exit code.
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
            generate(shell, &mut command, "gsap-audit", &mut io::stdout());
            Ok(0)
        }
    }
}

/// Execute the `scan` subcommand and compute its exit code.
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
    let options =
        ScanOptions::new(root.clone(), categories, max_files).with_exclude(exclude.to_vec());
    // The two baseline flags are separate modes. Accepting both would silently
    // ignore one of them, including a malformed path the caller expected to be
    // read.
    anyhow::ensure!(
        !(baseline.is_some() && write_baseline.is_some()),
        "--baseline and --write-baseline are separate modes; pass one"
    );

    let mut outcome = scan_root(&options)?;

    if let Some(path) = write_baseline {
        // A baseline recorded from a truncated scan blesses an inventory that
        // was never taken. Later runs under the same cap would then pass while
        // everything past it stayed unexamined.
        anyhow::ensure!(
            !outcome.truncated,
            "refusing to write a baseline from a truncated scan ({} files hit --max-files); \
             raise --max-files or narrow --root",
            outcome.files_scanned
        );
        Baseline::from_findings(&to_gate_findings(&outcome.findings)).save(path)?;
        return Ok(0);
    }

    // A baseline suppresses known findings from the report as well as from the
    // exit code. Leaving them in a report that claims to be gated would make
    // "clean" mean two different things in the same output.
    if let Some(path) = baseline {
        let known = Baseline::load(path)?;
        let unseen = known.unseen(&to_gate_findings(&outcome.findings));
        outcome.findings = outcome
            .findings
            .into_iter()
            .zip(unseen)
            .filter_map(|(finding, keep)| keep.then_some(finding))
            .collect();
    }

    let rendered = match format {
        OutputFormat::Markdown => {
            let mut text = format_markdown(TOOL_NAME, TOOL_VERSION, &outcome.findings);
            if outcome.truncated {
                write!(
                    text,
                    "\nLimitation: file walk truncated at {} files; some files were not analyzed.\n",
                    outcome.files_scanned
                )
                .expect("writing to a String is infallible");
            }
            text
        }
        // Findings are relative to --root; SARIF consumers resolve against the
        // repository, so re-anchor them or annotations land on the wrong file.
        OutputFormat::Sarif => serde_json::to_string_pretty(&to_sarif(
            TOOL_NAME,
            TOOL_VERSION,
            &to_gate_findings(&outcome.findings),
            &sarif_path_prefix(&root),
        ))?,
        OutputFormat::Json => {
            let mut value = format_json(TOOL_NAME, TOOL_VERSION, &outcome.findings);
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

    // Exit-code contract: 2 when any finding meets --min-severity
    // (default medium), else 0. Configurable so a repo can ratchet:
    // block on `high` today and tighten to `low` once clean.
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

/// Parse the `--categories` CSV into a category set. An empty/missing value
/// means "all categories". An unknown token is a usage error.
/// The scan root expressed relative to the current directory, for SARIF URIs.
///
/// `--root app` yields findings like `src/x.ts`; a repository uploader needs
/// `app/src/x.ts`. An absolute or outside-the-cwd root yields no prefix rather
/// than a wrong one.
fn sarif_path_prefix(root: &std::path::Path) -> String {
    let Ok(current) = std::env::current_dir() else {
        return String::new();
    };
    let Ok(absolute) = root.canonicalize() else {
        return String::new();
    };
    absolute
        .strip_prefix(&current)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default()
}

/// Convert core findings into the shared gate shape.
///
/// The cores keep their own severity enums; this is the one place the mapping
/// lives, so the gate layer needs no dependency on any parser crate.
fn to_gate_findings(findings: &[gsap_audit_core::types::Finding]) -> Vec<GateFinding> {
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

/// Write text followed by a newline to stdout.
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
        let set = parse_categories(Some("react, core")).unwrap();
        assert!(set.contains(&Category::React));
        assert!(set.contains(&Category::Core));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn parse_categories_rejects_unknown() {
        assert!(parse_categories(Some("bogus")).is_err());
    }

    #[test]
    fn scan_exits_two_for_medium_findings() {
        let root = temp_scan_root("medium");
        fs::write(root.join("app.ts"), "gsap.ticker.lagSmoothing(0);\n").unwrap();

        let code = run_scan(ScanRequest {
            root: root.clone(),
            format: OutputFormat::Json,
            categories: Some("performance"),
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
    fn scan_exits_zero_when_clean() {
        let root = temp_scan_root("clean");
        fs::write(root.join("app.ts"), "const value = 1;\n").unwrap();

        let code = run_scan(ScanRequest {
            root: root.clone(),
            format: OutputFormat::Json,
            categories: None,
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

    #[test]
    fn min_severity_raises_the_bar_without_hiding_findings() {
        let root = temp_scan_root("threshold");
        // A medium finding and nothing more severe.
        fs::write(root.join("app.ts"), "gsap.ticker.lagSmoothing(0);\n").unwrap();

        let at_medium = run_scan(ScanRequest {
            root: root.clone(),
            format: OutputFormat::Json,
            categories: Some("performance"),
            output: Some(root.join("medium.json")),
            max_files: 5000,
            min_severity: MinSeverity::Medium,
            exclude: &[],
            baseline: None,
            write_baseline: None,
        })
        .unwrap();
        let at_high = run_scan(ScanRequest {
            root: root.clone(),
            format: OutputFormat::Json,
            categories: Some("performance"),
            output: Some(root.join("high.json")),
            max_files: 5000,
            min_severity: MinSeverity::High,
            exclude: &[],
            baseline: None,
            write_baseline: None,
        })
        .unwrap();

        assert_eq!(at_medium, 2, "a medium finding must trip the default gate");
        assert_eq!(
            at_high, 0,
            "raising the threshold to high must let a medium-only tree pass"
        );

        // Raising the threshold changes the exit code, never the report: a gate
        // that silently dropped findings from its output would hide the work
        // still to do.
        let report = fs::read_to_string(root.join("high.json")).unwrap();
        assert!(
            report.contains("lag-smoothing-disabled"),
            "the finding must still be reported: {report}"
        );

        fs::remove_dir_all(root).unwrap();
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

    #[test]
    fn exclude_baseline_and_new_findings_compose() {
        let root = temp_scan_root("gate");
        fs::create_dir_all(root.join("vendor")).unwrap();
        let offending = "gsap.ticker.lagSmoothing(0);\n";
        fs::write(root.join("app.ts"), offending).unwrap();
        fs::write(root.join("vendor/copy.ts"), offending).unwrap();

        // Exclusion removes a finding from the report, not just the exit code:
        // a gate that hid a path from its own output would be lying.
        let excluded_report = root.join("excluded.json");
        run_scan(ScanRequest {
            root: root.clone(),
            format: OutputFormat::Json,
            categories: None,
            output: Some(excluded_report.clone()),
            max_files: 5000,
            min_severity: MinSeverity::Medium,
            exclude: &["vendor".to_string()],
            baseline: None,
            write_baseline: None,
        })
        .unwrap();
        let text = fs::read_to_string(&excluded_report).unwrap();
        assert!(
            !text.contains("vendor/copy.ts"),
            "excluded path still reported"
        );
        assert!(text.contains("app.ts"), "non-excluded path missing");

        // Baselining the current state makes the gate pass...
        let baseline = root.join("baseline.json");
        let code = run_scan(ScanRequest {
            root: root.clone(),
            format: OutputFormat::Json,
            categories: None,
            output: Some(root.join("out.json")),
            max_files: 5000,
            min_severity: MinSeverity::Medium,
            exclude: &[],
            baseline: None,
            write_baseline: Some(baseline.as_path()),
        })
        .unwrap();
        assert_eq!(code, 0, "writing a baseline exits 0 without gating");

        let code = run_scan(ScanRequest {
            root: root.clone(),
            format: OutputFormat::Json,
            categories: None,
            output: Some(root.join("out2.json")),
            max_files: 5000,
            min_severity: MinSeverity::Medium,
            exclude: &[],
            baseline: Some(baseline.as_path()),
            write_baseline: None,
        })
        .unwrap();
        assert_eq!(code, 0, "a fully baselined tree must pass");

        // ...but a NEW finding still trips it. Without this a baseline would be
        // indistinguishable from switching the gate off.
        fs::write(
            root.join("new.ts"),
            "import { Flip } from \"gsap-trial\";\nexport const f = Flip;\n",
        )
        .unwrap();
        let code = run_scan(ScanRequest {
            root: root.clone(),
            format: OutputFormat::Json,
            categories: None,
            output: Some(root.join("out3.json")),
            max_files: 5000,
            min_severity: MinSeverity::Medium,
            exclude: &[],
            baseline: Some(baseline.as_path()),
            write_baseline: None,
        })
        .unwrap();
        assert_eq!(code, 2, "a finding absent from the baseline must gate");

        fs::remove_dir_all(root).unwrap();
    }

    fn temp_scan_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("gsap-audit-{name}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
