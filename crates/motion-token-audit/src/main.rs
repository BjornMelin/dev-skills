//! motion-token-audit: static auditor CLI for cross-stack motion token drift.

use std::collections::BTreeSet;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use motion_token_audit_core::output::{format_catalog_json, format_catalog_markdown};
use motion_token_audit_core::{
    Category, ScanOptions, Severity, TOOL_NAME, TOOL_VERSION, format_json, format_markdown,
    highest_severity, scan_root,
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
        long_about = "Walk the given root, discover motion tokens, parse supported source files, and report drift/orphan findings. Exit code is 2 when any medium- or high-severity finding is present, otherwise 0.",
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Markdown,
    Json,
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
        } => run_scan(root, format, categories.as_deref(), output, max_files),
        Commands::Doctor { format } => {
            let text = match format {
                OutputFormat::Markdown => format_catalog_markdown(TOOL_NAME, TOOL_VERSION),
                OutputFormat::Json => {
                    let value = format_catalog_json(TOOL_NAME, TOOL_VERSION);
                    serde_json::to_string_pretty(&value)?
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

fn run_scan(
    root: PathBuf,
    format: OutputFormat,
    categories: Option<&str>,
    output: Option<PathBuf>,
    max_files: usize,
) -> Result<i32> {
    let categories = parse_categories(categories)?;
    let options = ScanOptions::new(root, categories, max_files);
    let outcome = scan_root(&options)?;

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

    let code = match highest_severity(&outcome.findings) {
        Some(Severity::High | Severity::Medium) => 2,
        _ => 0,
    };
    Ok(code)
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

        let code = run_scan(
            root.clone(),
            OutputFormat::Json,
            Some("tokens-reanimated"),
            Some(root.join("out.json")),
            5000,
        )
        .unwrap();

        assert_eq!(code, 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scan_exits_zero_for_low_only() {
        let root = temp_scan_root("low");
        fs::write(root.join("app.ts"), "withTiming(1, { duration: 237 });\n").unwrap();

        let code = run_scan(
            root.clone(),
            OutputFormat::Json,
            Some("tokens-reanimated"),
            Some(root.join("out.json")),
            5000,
        )
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
}
