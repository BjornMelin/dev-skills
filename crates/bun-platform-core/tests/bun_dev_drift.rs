use bun_platform_core::SkillContext;
use std::{fs, path::PathBuf};

fn skill_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../skills/bun-dev")
}

#[test]
fn tracked_bun_dev_skill_mentions_current_cli_surface() {
    let context = SkillContext::discover(Some(skill_root())).expect("skill context");
    let skill_md = context.read_skill_md().expect("skill");
    let references_index =
        fs::read_to_string(context.references_dir.join("index.md")).expect("references index");

    for snippet in [
        "codex-dev bun audit",
        "codex-dev bun fixes plan",
        "codex-dev bun references status",
        "codex-dev bun references plan",
    ] {
        assert!(
            skill_md.contains(snippet),
            "missing `{snippet}` in SKILL.md"
        );
    }

    for snippet in [
        "ref-bun-release-notes-latest.md",
        "ref-bun-capabilities-latest.md",
        "bun references status",
        "bun references plan",
    ] {
        assert!(
            references_index.contains(snippet),
            "missing `{snippet}` in references/index.md"
        );
    }
}
