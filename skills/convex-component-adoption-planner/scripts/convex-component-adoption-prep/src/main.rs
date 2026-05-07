use anyhow::{Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const COMPONENTS_INDEX_URL: &str = "https://www.convex.dev/components/components.md";
const PLAN_GROUP_SLUG: &str = "convex-components";

#[derive(Parser, Debug)]
#[command(
    name = "convex-component-adoption-prep",
    version,
    about = "Prepare deterministic research inputs for Convex component adoption planning"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show detected local tooling and posture notes.
    Doctor(OutputFlags),
    /// Normalize a component target into docs URLs, output paths, and suggested commands.
    Component(ComponentArgs),
    /// Create the plan folder and stub package files from a normalized component target.
    Scaffold(ScaffoldArgs),
}

#[derive(Args, Debug)]
struct OutputFlags {
    /// Emit JSON to stdout.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct ComponentArgs {
    /// NPM package or component slug, for example @convex-dev/aggregate.
    component: String,

    /// Optional feature or workstream slug.
    #[arg(long)]
    feature: Option<String>,

    /// Optional docs slug override when it does not match the package name.
    #[arg(long)]
    docs_slug: Option<String>,

    /// Optional YYYY-MM-DD date override. Defaults to the local date.
    #[arg(long)]
    date: Option<String>,

    /// Plan root directory. The helper appends YYYY-MM/MM-DD/convex-components/<slug>.
    #[arg(long, default_value = ".agents/plans")]
    plan_root: PathBuf,

    /// Optional repo root override. Defaults to current directory or nearest ancestor containing .agents or .git.
    #[arg(long)]
    repo_root: Option<PathBuf>,

    /// Emit JSON to stdout.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct ScaffoldArgs {
    /// NPM package or component slug, for example @convex-dev/aggregate.
    component: String,

    /// Optional feature or workstream slug.
    #[arg(long)]
    feature: Option<String>,

    /// Optional docs slug override when it does not match the package name.
    #[arg(long)]
    docs_slug: Option<String>,

    /// Optional YYYY-MM-DD date override. Defaults to the local date.
    #[arg(long)]
    date: Option<String>,

    /// Plan root directory. The helper appends YYYY-MM/MM-DD/convex-components/<slug>.
    #[arg(long, default_value = ".agents/plans")]
    plan_root: PathBuf,

    /// Optional repo root override. Defaults to current directory or nearest ancestor containing .agents or .git.
    #[arg(long)]
    repo_root: Option<PathBuf>,

    /// Overwrite existing stub files.
    #[arg(long)]
    force: bool,

    /// Print scaffold file contents to stdout instead of writing them.
    #[arg(long)]
    stdout: bool,

    /// Emit JSON to stdout.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct DoctorOutput {
    cwd: String,
    repo_root: Option<String>,
    tools: Vec<ToolStatus>,
    notes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ToolStatus {
    name: &'static str,
    available: bool,
    command: &'static str,
    role: &'static str,
    recommended: bool,
}

#[derive(Debug, Serialize)]
struct ComponentOutput {
    input: String,
    package_name: String,
    docs_slug: String,
    owner_class: String,
    feature_slug: Option<String>,
    docs: DocsLinks,
    paths: OutputPaths,
    commands: Vec<SuggestedCommand>,
    notes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ScaffoldOutput {
    component: String,
    created_dirs: Vec<String>,
    created_files: Vec<String>,
    skipped_files: Vec<String>,
    rendered_files: Vec<RenderedFile>,
    force: bool,
    stdout: bool,
}

#[derive(Debug, Serialize)]
struct RenderedFile {
    path: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct DocsLinks {
    components_index: String,
    component_markdown: String,
}

#[derive(Debug, Serialize)]
struct OutputPaths {
    date: String,
    year_month: String,
    day_folder: String,
    repo_root: String,
    plan_group_dir: String,
    component_dir: String,
    plan_md: String,
    codex_full_prompt_md: String,
}

#[derive(Debug, Serialize)]
struct SuggestedCommand {
    name: &'static str,
    command: String,
    reason: &'static str,
    available: Option<bool>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Doctor(args) => {
            let output = build_doctor_output()?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                print_doctor_human(&output);
            }
        }
        Commands::Component(args) => {
            let json = args.json;
            let output = build_component_output(args)?;
            if output.paths.repo_root.is_empty() {
                anyhow::bail!("repo root could not be determined");
            }
            if output.paths.component_dir.is_empty() {
                anyhow::bail!("component output path could not be determined");
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                print_component_human(&output);
            }
        }
        Commands::Scaffold(args) => {
            let json = args.json;
            let output = scaffold_package(args)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                print_scaffold_human(&output);
            }
        }
    }

    Ok(())
}

fn build_doctor_output() -> Result<DoctorOutput> {
    let cwd = env::current_dir().context("failed to determine current directory")?;
    let repo_root = find_repo_root(&cwd);

    let tools = vec![
        ToolStatus {
            name: "curl",
            available: command_exists("curl"),
            command: "curl",
            role: "required for live Convex component markdown fetches",
            recommended: true,
        },
        ToolStatus {
            name: "rg",
            available: command_exists("rg"),
            command: "rg",
            role: "preferred repo search tool",
            recommended: true,
        },
        ToolStatus {
            name: "opensrc",
            available: command_exists("opensrc"),
            command: "opensrc",
            role: "dependency source inspection",
            recommended: true,
        },
        ToolStatus {
            name: "ctx7",
            available: command_exists("ctx7"),
            command: "ctx7",
            role: "optional library docs lookup",
            recommended: true,
        },
        ToolStatus {
            name: "gh",
            available: command_exists("gh"),
            command: "gh",
            role: "optional GitHub context only when PRs or issues matter",
            recommended: false,
        },
        ToolStatus {
            name: "bunx",
            available: command_exists("bunx"),
            command: "bunx",
            role: "optional Convex CLI access via bunx convex",
            recommended: false,
        },
        ToolStatus {
            name: "cargo",
            available: command_exists("cargo"),
            command: "cargo",
            role: "build or install this helper locally",
            recommended: true,
        },
    ];

    let notes = vec![
        "Prefer curl plus repo reads first; do not replace recommendation reasoning with tooling.".to_string(),
        "Use opensrc when package internals, table ownership, or generated APIs matter.".to_string(),
        "Use ctx7 for supplemental docs lookup only after the live component markdown page is fetched.".to_string(),
        "Treat gh as low priority unless a GitHub issue, PR, or release artifact is part of the decision.".to_string(),
    ];

    Ok(DoctorOutput {
        cwd: cwd.display().to_string(),
        repo_root: repo_root.map(|path| path.display().to_string()),
        tools,
        notes,
    })
}

fn build_component_output(args: ComponentArgs) -> Result<ComponentOutput> {
    let cwd = env::current_dir().context("failed to determine current directory")?;
    let repo_root = args
        .repo_root
        .clone()
        .or_else(|| find_repo_root(&cwd))
        .unwrap_or(cwd);

    let date = match args.date.as_deref() {
        Some(raw) => NaiveDate::parse_from_str(raw, "%Y-%m-%d")
            .with_context(|| format!("failed to parse date '{raw}' as YYYY-MM-DD"))?,
        None => Local::now().date_naive(),
    };

    let package_name = args.component.trim().to_string();
    let docs_slug = args
        .docs_slug
        .as_deref()
        .map(normalize_slug)
        .unwrap_or_else(|| infer_docs_slug(&package_name));
    let owner_class = if package_name.starts_with("@convex-dev/") {
        "official".to_string()
    } else {
        "third_party_or_unknown".to_string()
    };
    let feature_slug = args.feature.as_deref().map(normalize_slug);

    let year_month = format!("{:04}-{:02}", date.year(), date.month());
    let day_folder = format!("{:02}-{:02}", date.month(), date.day());
    let plan_group_dir = repo_root
        .join(&args.plan_root)
        .join(&year_month)
        .join(&day_folder)
        .join(PLAN_GROUP_SLUG);
    let component_dir = plan_group_dir.join(&docs_slug);
    let plan_md = component_dir.join("PLAN.md");
    let codex_full_prompt_md = component_dir.join("CODEX_FULL_PROMPT.md");
    let component_markdown =
        format!("https://www.convex.dev/components/{0}/{0}.md", docs_slug);

    let feature_query = feature_slug
        .as_deref()
        .unwrap_or("component adoption");
    let repo_grep = if let Some(feature) = &feature_slug {
        format!("{docs_slug}|{feature}")
    } else {
        docs_slug.clone()
    };

    let commands = vec![
        SuggestedCommand {
            name: "curl-index",
            command: format!("curl -s {COMPONENTS_INDEX_URL}"),
            reason: "fetch the live Convex components index",
            available: Some(command_exists("curl")),
        },
        SuggestedCommand {
            name: "curl-component",
            command: format!("curl -s {component_markdown}"),
            reason: "fetch the live markdown page for the component under review",
            available: Some(command_exists("curl")),
        },
        SuggestedCommand {
            name: "opensrc-source",
            command: format!("opensrc path {package_name}"),
            reason: "inspect package source when internals or ownership matter",
            available: Some(command_exists("opensrc")),
        },
        SuggestedCommand {
            name: "ctx7-library",
            command: format!("ctx7 library convex \"components {docs_slug}\" --json"),
            reason: "resolve the best Convex library ID for supplemental docs lookup",
            available: Some(command_exists("ctx7")),
        },
        SuggestedCommand {
            name: "ctx7-docs",
            command: format!(
                "ctx7 docs <resolved-library-id> \"{feature_query} {docs_slug}\" --json"
            ),
            reason: "query supplemental library docs after resolving the library ID",
            available: Some(command_exists("ctx7")),
        },
        SuggestedCommand {
            name: "repo-components",
            command: "rg -n \"app\\.use|components\\.\" packages/backend/convex/convex.config.ts packages/backend/convex".to_string(),
            reason: "map current mounted components and component call sites",
            available: Some(command_exists("rg")),
        },
        SuggestedCommand {
            name: "repo-feature",
            command: format!(
                "rg -n \"{repo_grep}\" packages/backend docs .agents/signr-implementation-prompts-modernization-package"
            ),
            reason: "find feature ownership and prompt-package overlap",
            available: Some(command_exists("rg")),
        },
    ];

    let mut notes = vec![
        "Docs URL is inferred from the slug. If the components index shows a different mapping, rerun with --docs-slug.".to_string(),
        "The helper prepares research inputs only. It does not decide whether the component should be adopted.".to_string(),
        "If the feature already exists in the modernization-package prompts, prompt-package alignment belongs in the same plan.".to_string(),
    ];

    if feature_slug.is_none() {
        notes.push("No feature slug was supplied. The final plan should still lock the target workstream before making a recommendation.".to_string());
    }

    Ok(ComponentOutput {
        input: args.component,
        package_name,
        docs_slug,
        owner_class,
        feature_slug,
        docs: DocsLinks {
            components_index: COMPONENTS_INDEX_URL.to_string(),
            component_markdown,
        },
        paths: OutputPaths {
            date: date.format("%Y-%m-%d").to_string(),
            year_month,
            day_folder,
            repo_root: repo_root.display().to_string(),
            plan_group_dir: plan_group_dir.display().to_string(),
            component_dir: component_dir.display().to_string(),
            plan_md: plan_md.display().to_string(),
            codex_full_prompt_md: codex_full_prompt_md.display().to_string(),
        },
        commands,
        notes,
    })
}

fn scaffold_package(args: ScaffoldArgs) -> Result<ScaffoldOutput> {
    let component_output = build_component_output(ComponentArgs {
        component: args.component.clone(),
        feature: args.feature.clone(),
        docs_slug: args.docs_slug.clone(),
        date: args.date.clone(),
        plan_root: args.plan_root.clone(),
        repo_root: args.repo_root.clone(),
        json: false,
    })?;

    let component_dir = PathBuf::from(&component_output.paths.component_dir);
    let plan_md = PathBuf::from(&component_output.paths.plan_md);
    let codex_full_prompt_md = PathBuf::from(&component_output.paths.codex_full_prompt_md);
    let plan_content = build_plan_stub(&component_output);
    let prompt_content = build_prompt_stub(&component_output);

    if args.stdout {
        return Ok(ScaffoldOutput {
            component: component_output.package_name,
            created_dirs: Vec::new(),
            created_files: Vec::new(),
            skipped_files: Vec::new(),
            rendered_files: vec![
                RenderedFile {
                    path: plan_md.display().to_string(),
                    content: plan_content,
                },
                RenderedFile {
                    path: codex_full_prompt_md.display().to_string(),
                    content: prompt_content,
                },
            ],
            force: args.force,
            stdout: true,
        });
    }

    fs::create_dir_all(&component_dir).with_context(|| {
        format!(
            "failed to create component output directory '{}'",
            component_dir.display()
        )
    })?;

    let mut created_files = Vec::new();
    let mut skipped_files = Vec::new();

    write_scaffold_file(
        &plan_md,
        &plan_content,
        args.force,
        &mut created_files,
        &mut skipped_files,
    )?;
    write_scaffold_file(
        &codex_full_prompt_md,
        &prompt_content,
        args.force,
        &mut created_files,
        &mut skipped_files,
    )?;

    Ok(ScaffoldOutput {
        component: component_output.package_name,
        created_dirs: vec![component_dir.display().to_string()],
        created_files,
        skipped_files,
        rendered_files: Vec::new(),
        force: args.force,
        stdout: false,
    })
}

fn print_doctor_human(output: &DoctorOutput) {
    println!("cwd: {}", output.cwd);
    if let Some(repo_root) = &output.repo_root {
        println!("repo_root: {repo_root}");
    }
    println!();
    println!("tools:");
    for tool in &output.tools {
        let status = if tool.available { "yes" } else { "no" };
        println!(
            "- {}: {} | {} | {}",
            tool.name, status, tool.command, tool.role
        );
    }
    println!();
    println!("notes:");
    for note in &output.notes {
        println!("- {note}");
    }
}

fn print_scaffold_human(output: &ScaffoldOutput) {
    println!("component: {}", output.component);
    println!("force: {}", output.force);
    println!("stdout: {}", output.stdout);
    if output.stdout {
        println!();
        println!("rendered_files:");
        for file in &output.rendered_files {
            println!("--- {}", file.path);
            println!("{}", file.content);
        }
        return;
    }
    println!();
    println!("created_dirs:");
    for dir in &output.created_dirs {
        println!("- {dir}");
    }
    println!();
    println!("created_files:");
    for file in &output.created_files {
        println!("- {file}");
    }
    if !output.skipped_files.is_empty() {
        println!();
        println!("skipped_files:");
        for file in &output.skipped_files {
            println!("- {file}");
        }
    }
}

fn print_component_human(output: &ComponentOutput) {
    println!("component: {}", output.package_name);
    println!("docs_slug: {}", output.docs_slug);
    println!("owner_class: {}", output.owner_class);
    if let Some(feature_slug) = &output.feature_slug {
        println!("feature_slug: {feature_slug}");
    }
    println!("components_index: {}", output.docs.components_index);
    println!("component_markdown: {}", output.docs.component_markdown);
    println!();
    println!("output_paths:");
    println!("- plan_group_dir: {}", output.paths.plan_group_dir);
    println!("- component_dir: {}", output.paths.component_dir);
    println!("- plan_md: {}", output.paths.plan_md);
    println!(
        "- codex_full_prompt_md: {}",
        output.paths.codex_full_prompt_md
    );
    println!();
    println!("suggested_commands:");
    for command in &output.commands {
        println!("- {}: {}", command.name, command.command);
    }
    println!();
    println!("notes:");
    for note in &output.notes {
        println!("- {note}");
    }
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(path) = current {
        if path.join(".agents").exists() || path.join(".git").exists() {
            return Some(path.to_path_buf());
        }
        current = path.parent();
    }
    None
}

fn command_exists(name: &str) -> bool {
    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&path_var).any(|dir| dir.join(name).exists())
}

fn infer_docs_slug(component: &str) -> String {
    let base = component
        .rsplit('/')
        .next()
        .unwrap_or(component)
        .trim()
        .trim_start_matches('@');
    normalize_slug(base)
}

fn normalize_slug(raw: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in raw.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            slug.push(lower);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

fn write_scaffold_file(
    path: &Path,
    content: &str,
    force: bool,
    created_files: &mut Vec<String>,
    skipped_files: &mut Vec<String>,
) -> Result<()> {
    if path.exists() && !force {
        skipped_files.push(path.display().to_string());
        return Ok(());
    }

    fs::write(path, content)
        .with_context(|| format!("failed to write scaffold file '{}'", path.display()))?;
    created_files.push(path.display().to_string());
    Ok(())
}

fn build_plan_stub(output: &ComponentOutput) -> String {
    let feature = output
        .feature_slug
        .clone()
        .unwrap_or_else(|| "REPLACE_ME_FEATURE_SLUG".to_string());

    format!(
        "# Convex component adoption plan\n\n\
## Objective\n\n\
- Component: `{package}`\n\
- Docs slug: `{docs_slug}`\n\
- Feature/workstream: `{feature}`\n\
- Decision: `REPLACE_ME_ADOPT_OR_REJECT`\n\n\
## Locked decisions\n\n\
- Recommendation: `REPLACE_ME`\n\
- Weighted score: `REPLACE_ME`\n\
- Notes: `REPLACE_ME`\n\n\
## Live source set\n\n\
- Components index: {components_index}\n\
- Component markdown: {component_markdown}\n\
- `opensrc` package: `{package}`\n\n\
## Repo context to read\n\n\
- `packages/backend/package.json`\n\
- `packages/backend/convex/convex.config.ts`\n\
- `packages/backend/convex/schema.ts`\n\
- `REPLACE_ME_FEATURE_FILES`\n\
- `REPLACE_ME_PROMPT_PACKAGE_FILES_IF_APPLICABLE`\n\n\
## Context regathering commands\n\n\
```bash\n\
curl -s {components_index}\n\
curl -s {component_markdown}\n\
opensrc path {package}\n\
rg -n \"app\\.use|components\\.\" packages/backend/convex/convex.config.ts packages/backend/convex\n\
rg -n \"{feature}|{docs_slug}\" packages/backend docs .agents/signr-implementation-prompts-modernization-package\n\
```\n\n\
## Preferred architecture\n\n\
- Current durable owner: `REPLACE_ME`\n\
- Proposed owner after decision: `REPLACE_ME`\n\
- Duplicate ownership risks: `REPLACE_ME`\n\n\
## Implementation sequence\n\n\
1. `REPLACE_ME`\n\
2. `REPLACE_ME`\n\
3. `REPLACE_ME`\n\n\
## Validation\n\n\
```bash\n\
bun run validate:local:agent\n\
```\n\n\
## Acceptance criteria\n\n\
- `REPLACE_ME`\n\n\
## Risks and rollback posture\n\n\
- `REPLACE_ME`\n",
        package = output.package_name,
        docs_slug = output.docs_slug,
        feature = feature,
        components_index = output.docs.components_index,
        component_markdown = output.docs.component_markdown,
    )
}

fn build_prompt_stub(output: &ComponentOutput) -> String {
    let feature = output
        .feature_slug
        .clone()
        .unwrap_or_else(|| "REPLACE_ME_FEATURE_SLUG".to_string());

    format!(
        "# Fresh-session execution prompt\n\n\
You are working in the Signr repo. Adopt or definitively reject the Convex component `{package}` for the `{feature}` workstream.\n\n\
## Locked inputs\n\n\
- Package: `{package}`\n\
- Docs slug: `{docs_slug}`\n\
- Components index: {components_index}\n\
- Component markdown: {component_markdown}\n\
- Plan file: `{plan_md}`\n\n\
## Read first\n\n\
- `packages/backend/package.json`\n\
- `packages/backend/convex/convex.config.ts`\n\
- `packages/backend/convex/schema.ts`\n\
- `REPLACE_ME_FEATURE_FILES`\n\
- `REPLACE_ME_PROMPT_PACKAGE_FILES_IF_APPLICABLE`\n\n\
## Regather external context\n\n\
```bash\n\
curl -s {components_index}\n\
curl -s {component_markdown}\n\
opensrc path {package}\n\
```\n\n\
## Required output\n\n\
Update `PLAN.md` and this prompt with final locked decisions, exact integration steps, prompt-package alignment, and validation commands.\n\n\
## Final response contract\n\n\
- state adopt vs reject\n\
- explain durable ownership\n\
- list exact files to change later\n\
- list validation commands to run later\n",
        package = output.package_name,
        docs_slug = output.docs_slug,
        feature = feature,
        components_index = output.docs.components_index,
        component_markdown = output.docs.component_markdown,
        plan_md = output.paths.plan_md,
    )
}
