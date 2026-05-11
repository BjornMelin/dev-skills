use crate::*;

pub(crate) fn doctor(json_out: bool) -> Result<()> {
    let paths = research_paths()?;
    let mut env = BTreeMap::new();
    for key in [
        "CONTEXT7_API_KEY",
        "FIRECRAWL_API_KEY",
        "GITHUB_TOKEN",
        "GH_TOKEN",
        "EXA_API_KEY",
        "CODEX_RESEARCH_HOME",
    ] {
        env.insert(key, std::env::var_os(key).is_some());
    }

    let mut tools = BTreeMap::new();
    tools.insert("gh", command_version("gh", &["--version"]));
    tools.insert(
        "agent-browser",
        command_version("agent-browser", &["--version"]),
    );
    tools.insert("ctx7", command_version("ctx7", &["--version"]));
    tools.insert("opensrc", command_version("opensrc", &["--version"]));

    let notes = vec![
        "Codex-native web.search_query/open/find are session tools, not external CLI APIs.".to_string(),
        "Use Context7 REST API directly for library docs and refreshes.".to_string(),
        "Use Firecrawl only after classification or when the task explicitly needs rendered/crawl extraction.".to_string(),
    ];

    let report = DoctorReport {
        cache_dir: paths.cache_dir,
        database: paths.database,
        blobs_dir: paths.blobs_dir,
        env,
        tools,
        notes,
    };
    if json_out {
        print_json(&report)
    } else {
        println!("cache: {}", report.cache_dir.display());
        println!("database: {}", report.database.display());
        println!("blobs: {}", report.blobs_dir.display());
        println!("env:");
        for (key, present) in report.env {
            println!("  {key}: {}", if present { "present" } else { "missing" });
        }
        println!("tools:");
        for (name, version) in report.tools {
            let status = version
                .as_deref()
                .and_then(|v| v.lines().next())
                .unwrap_or("missing");
            println!("  {name}: {status}");
        }
        Ok(())
    }
}
