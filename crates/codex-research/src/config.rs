use crate::*;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConfigReport {
    pub(crate) path: Option<PathBuf>,
    pub(crate) config: ResearchConfig,
}

pub(crate) fn handle_config(
    command: ConfigCommand,
    loaded_path: Option<PathBuf>,
    json_out: bool,
) -> Result<()> {
    match command {
        ConfigCommand::Init { path, force } => {
            let path = path.unwrap_or_else(|| PathBuf::from(".codex/research/config.toml"));
            if path.exists() && !force {
                bail!(
                    "config exists; pass --force to overwrite: {}",
                    path.display()
                );
            }
            ensure_parent(&path)?;
            fs::write(&path, default_config_toml()?.as_bytes())?;
            if json_out {
                print_json(&json!({ "path": path, "written": true }))
            } else {
                println!("config: {}", path.display());
                Ok(())
            }
        }
        ConfigCommand::Show => {
            let loaded = load_config(loaded_path.as_deref())?;
            let report = ConfigReport {
                path: loaded.path,
                config: loaded.config,
            };
            if json_out {
                print_json(&report)
            } else {
                println!("{}", toml::to_string_pretty(&report.config)?);
                Ok(())
            }
        }
    }
}
