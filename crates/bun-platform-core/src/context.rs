use anyhow::{Context, Result};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Default skill root under `$HOME`; override with `BUN_PLATFORM_SKILL_ROOT` or
/// the CLI `--skill-root` flag when targeting repo-local skill sources.
const DEFAULT_SKILL_ROOT: &str = ".agents/skills/bun-dev";

#[derive(Clone, Debug)]
pub struct SkillContext {
    pub skill_root: PathBuf,
    pub rules_dir: PathBuf,
    pub references_dir: PathBuf,
}

impl SkillContext {
    pub fn discover(skill_root_override: Option<PathBuf>) -> Result<Self> {
        let skill_root = if let Some(override_path) = skill_root_override {
            override_path
        } else if let Ok(from_env) = env::var("BUN_PLATFORM_SKILL_ROOT") {
            PathBuf::from(from_env)
        } else {
            let home =
                env::var("HOME").context("HOME is not set and --skill-root was not provided")?;
            PathBuf::from(home).join(DEFAULT_SKILL_ROOT)
        };
        let skill_root = skill_root
            .canonicalize()
            .with_context(|| format!("failed to resolve skill root {}", skill_root.display()))?;
        Ok(Self {
            rules_dir: skill_root.join("rules"),
            references_dir: skill_root.join("references"),
            skill_root,
        })
    }

    pub fn list_rule_ids(&self) -> Result<Vec<String>> {
        let mut rule_ids = Vec::new();
        for entry in fs::read_dir(&self.rules_dir)? {
            let path = entry?.path();
            if !path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
            {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if stem == "_index" {
                continue;
            }
            rule_ids.push(stem.to_string());
        }
        rule_ids.sort();
        Ok(rule_ids)
    }

    pub fn explain_rule(&self, rule_id: &str) -> Result<String> {
        fs::read_to_string(self.rules_dir.join(format!("{rule_id}.md"))).with_context(|| {
            format!(
                "failed to read rule {}",
                self.rules_dir.join(format!("{rule_id}.md")).display()
            )
        })
    }

    pub fn is_references_flat(&self) -> Result<bool> {
        for entry in fs::read_dir(&self.references_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn read_skill_md(&self) -> Result<String> {
        fs::read_to_string(self.skill_root.join("SKILL.md")).context("failed to read SKILL.md")
    }

    pub fn skill_path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.skill_root.join(relative)
    }
}
