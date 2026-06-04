use std::fs;
use std::path::Path;

use tempfile::tempdir;

use super::*;

#[test]
fn path_rule_after_name_rule_disables_skill() {
    let temp = tempdir().expect("tempdir");
    let codex = temp.path().join(".codex");
    let agents = temp.path().join(".agents");
    let kimi = temp.path().join(".kimi-code");
    let plugin = codex.join("plugins/cache/clerk/clerk-skills/1.0.0/skills/core/clerk-setup");
    write_skill(&plugin, "clerk-setup");
    write_text(
        &codex.join("plugins/cache/clerk/clerk-skills/1.0.0/.codex-plugin/plugin.json"),
        r#"{"name":"clerk-skills","skills":"./skills/"}"#,
    );
    write_text(
        &codex.join("config.toml"),
        &format!(
            r#"
[[skills.config]]
name = "clerk-skills:clerk-setup"
enabled = true

[[skills.config]]
path = "{}"
enabled = false

[plugins."clerk-skills@clerk"]
enabled = true
"#,
            plugin.join("SKILL.md").display()
        ),
    );

    let report = kimi_sync(KimiSyncArgs {
        apply: false,
        scope: KimiSyncScope::Focused,
        codex_home: Some(codex),
        agents_home: Some(agents),
        kimi_home: Some(kimi),
        project_root: None,
        checked_at: None,
    })
    .expect("sync");

    assert!(
        !report
            .included
            .iter()
            .any(|skill| skill.name == "clerk-setup")
    );
    assert!(
        report
            .excluded
            .iter()
            .any(|skill| skill.name == "clerk-setup")
    );
}

#[test]
fn plain_name_rule_disables_global_skill() {
    let temp = tempdir().expect("tempdir");
    let codex = temp.path().join(".codex");
    let agents = temp.path().join(".agents");
    let kimi = temp.path().join(".kimi-code");
    write_skill(&agents.join("skills/disabled-global"), "disabled-global");
    write_skill(&agents.join("skills/enabled-global"), "enabled-global");
    write_text(
        &codex.join("config.toml"),
        r#"[[skills.config]]
name = "disabled-global"
enabled = false
"#,
    );

    let report = kimi_sync(KimiSyncArgs {
        apply: false,
        scope: KimiSyncScope::GlobalOnly,
        codex_home: Some(codex),
        agents_home: Some(agents),
        kimi_home: Some(kimi),
        project_root: None,
        checked_at: None,
    })
    .expect("sync");

    assert!(
        report
            .included
            .iter()
            .any(|skill| skill.name == "enabled-global")
    );
    assert!(
        !report
            .included
            .iter()
            .any(|skill| skill.name == "disabled-global")
    );
    assert!(report.excluded.iter().any(|skill| {
        skill.name == "disabled-global" && skill.reason == "disabled by Codex skills.config"
    }));
}

#[test]
fn missing_config_is_empty_and_global_only_skips_project_skills() {
    let temp = tempdir().expect("tempdir");
    let codex = temp.path().join(".codex");
    let agents = temp.path().join(".agents");
    let kimi = temp.path().join(".kimi-code");
    let project = temp.path().join("project");
    write_skill(&agents.join("skills/global-skill"), "global-skill");
    write_skill(
        &project.join(".agents/skills/project-skill"),
        "project-skill",
    );
    write_skill(&project.join(".kimi-code/skills/kimi-only"), "kimi-only");
    fs::create_dir_all(&codex).expect("codex dir");

    let report = kimi_sync(KimiSyncArgs {
        apply: false,
        scope: KimiSyncScope::GlobalOnly,
        codex_home: Some(codex),
        agents_home: Some(agents),
        kimi_home: Some(kimi),
        project_root: Some(project),
        checked_at: None,
    })
    .expect("sync");

    assert!(
        report
            .included
            .iter()
            .any(|skill| skill.name == "global-skill")
    );
    assert!(
        !report
            .included
            .iter()
            .any(|skill| skill.name == "project-skill")
    );
    assert!(
        !report
            .included
            .iter()
            .any(|skill| skill.name == "kimi-only")
    );
}

#[test]
fn project_sync_ignores_kimi_code_skills() {
    let temp = tempdir().expect("tempdir");
    let codex = temp.path().join(".codex");
    let agents = temp.path().join(".agents");
    let kimi = temp.path().join(".kimi-code");
    let project = temp.path().join("project");
    write_text(&codex.join("config.toml"), "");
    write_skill(
        &project.join(".agents/skills/project-skill"),
        "project-skill",
    );
    write_skill(&project.join(".kimi-code/skills/kimi-only"), "kimi-only");

    let report = kimi_sync(KimiSyncArgs {
        apply: false,
        scope: KimiSyncScope::Focused,
        codex_home: Some(codex),
        agents_home: Some(agents),
        kimi_home: Some(kimi),
        project_root: Some(project),
        checked_at: None,
    })
    .expect("sync");

    assert!(
        report
            .included
            .iter()
            .any(|skill| skill.name == "project-skill")
    );
    assert!(
        !report
            .included
            .iter()
            .any(|skill| skill.name == "kimi-only")
    );
}

#[test]
fn source_qualified_plugin_entries_do_not_collapse() {
    let temp = tempdir().expect("tempdir");
    let codex = temp.path().join(".codex");
    let agents = temp.path().join(".agents");
    let kimi = temp.path().join(".kimi-code");
    write_skill(
        &codex.join("plugins/cache/source-a/expo/1.0.0/skills/expo-a"),
        "expo-a",
    );
    write_skill(
        &codex.join("plugins/cache/source-b/expo/1.0.0/skills/expo-b"),
        "expo-b",
    );
    write_text(
        &codex.join("plugins/cache/source-a/expo/1.0.0/.codex-plugin/plugin.json"),
        r#"{"name":"expo","skills":"./skills/"}"#,
    );
    write_text(
        &codex.join("plugins/cache/source-b/expo/1.0.0/.codex-plugin/plugin.json"),
        r#"{"name":"expo","skills":"./skills/"}"#,
    );
    write_text(
        &codex.join("config.toml"),
        r#"[plugins."expo@source-a"]
enabled = true

[plugins."expo@source-b"]
enabled = true
"#,
    );

    let report = kimi_sync(KimiSyncArgs {
        apply: false,
        scope: KimiSyncScope::AllEnabled,
        codex_home: Some(codex),
        agents_home: Some(agents),
        kimi_home: Some(kimi),
        project_root: None,
        checked_at: None,
    })
    .expect("sync");

    assert!(report.included.iter().any(|skill| skill.name == "expo-a"));
    assert!(report.included.iter().any(|skill| skill.name == "expo-b"));
}

#[test]
fn plugin_skills_root_escape_is_skipped() {
    let temp = tempdir().expect("tempdir");
    let codex = temp.path().join(".codex");
    let agents = temp.path().join(".agents");
    let kimi = temp.path().join(".kimi-code");
    write_skill(
        &codex.join("plugins/cache/source/external/escape"),
        "escape",
    );
    write_text(
        &codex.join("plugins/cache/source/expo/1.0.0/.codex-plugin/plugin.json"),
        r#"{"name":"expo","skills":"../../../external"}"#,
    );
    write_text(
        &codex.join("config.toml"),
        r#"[plugins."expo@source"]
enabled = true
"#,
    );

    let report = kimi_sync(KimiSyncArgs {
        apply: false,
        scope: KimiSyncScope::Focused,
        codex_home: Some(codex),
        agents_home: Some(agents),
        kimi_home: Some(kimi),
        project_root: None,
        checked_at: None,
    })
    .expect("sync");

    assert!(!report.included.iter().any(|skill| skill.name == "escape"));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "plugin_skills_root_escape"
            || diagnostic.code == "plugin_skills_root_canonicalize_error"
    }));
}

#[test]
fn forced_vercel_shadcn_exclusion_preserves_global_shadcn() {
    let temp = tempdir().expect("tempdir");
    let codex = temp.path().join(".codex");
    let agents = temp.path().join(".agents");
    let kimi = temp.path().join(".kimi-code");
    write_skill(&agents.join("skills/shadcn"), "shadcn");
    write_skill(
        &codex.join("plugins/cache/openai-curated/vercel/2abb1c44/skills/shadcn"),
        "shadcn",
    );
    write_text(
        &codex.join("plugins/cache/openai-curated/vercel/2abb1c44/.codex-plugin/plugin.json"),
        r#"{"name":"vercel","skills":"./skills/"}"#,
    );
    write_text(
        &codex.join("config.toml"),
        r#"[plugins."vercel@openai-curated"]
enabled = true
"#,
    );

    let report = kimi_sync(KimiSyncArgs {
        apply: false,
        scope: KimiSyncScope::Focused,
        codex_home: Some(codex),
        agents_home: Some(agents),
        kimi_home: Some(kimi),
        project_root: None,
        checked_at: None,
    })
    .expect("sync");

    let included = report
        .included
        .iter()
        .find(|skill| skill.name == "shadcn")
        .expect("global shadcn included");
    assert_eq!(included.source_kind, KimiSyncSourceKind::GlobalSkill);
    assert!(
        report
            .excluded
            .iter()
            .any(|skill| skill.plugin.as_deref() == Some("vercel") && skill.name == "shadcn")
    );
}

#[test]
fn apply_writes_symlink_mirror_only_under_kimi_sync_root() {
    let temp = tempdir().expect("tempdir");
    let codex = temp.path().join(".codex");
    let agents = temp.path().join(".agents");
    let kimi = temp.path().join(".kimi-code");
    let source_skill = agents.join("skills/deep-researcher");
    write_skill(&source_skill, "deep-researcher");
    write_text(&codex.join("config.toml"), "");

    let report = kimi_sync(KimiSyncArgs {
        apply: true,
        scope: KimiSyncScope::GlobalOnly,
        codex_home: Some(codex),
        agents_home: Some(agents),
        kimi_home: Some(kimi),
        project_root: None,
        checked_at: None,
    })
    .expect("sync");

    let mirror_skill = report.skills_root.join("deep-researcher");
    let link_metadata = fs::symlink_metadata(&mirror_skill).expect("mirror skill metadata");
    assert!(link_metadata.file_type().is_symlink());
    assert_eq!(
        fs::read_link(&mirror_skill).expect("mirror skill symlink target"),
        source_skill
    );

    let manifest_path = report.mirror_root.join("manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("read mirror manifest"))
            .expect("valid mirror manifest json");
    let included = manifest["included"].as_array().expect("included skills");
    assert!(
        included
            .iter()
            .any(|skill| skill["name"] == "deep-researcher")
    );
}

#[cfg(unix)]
#[test]
fn apply_rejects_symlinked_sync_root() {
    let temp = tempdir().expect("tempdir");
    let codex = temp.path().join(".codex");
    let agents = temp.path().join(".agents");
    let kimi = temp.path().join(".kimi-code");
    let outside = temp.path().join("outside-sync");
    write_skill(&agents.join("skills/deep-researcher"), "deep-researcher");
    write_text(&codex.join("config.toml"), "");
    fs::create_dir_all(&kimi).expect("kimi home");
    fs::create_dir_all(&outside).expect("outside sync");
    std::os::unix::fs::symlink(&outside, kimi.join("codex-sync")).expect("sync symlink");

    let error = kimi_sync(KimiSyncArgs {
        apply: true,
        scope: KimiSyncScope::GlobalOnly,
        codex_home: Some(codex),
        agents_home: Some(agents),
        kimi_home: Some(kimi),
        project_root: None,
        checked_at: None,
    })
    .expect_err("symlinked sync root rejected");

    assert!(error.to_string().contains("symlinked Kimi sync root"));
}

#[test]
fn explicit_missing_project_root_is_rejected() {
    let temp = tempdir().expect("tempdir");
    let codex = temp.path().join(".codex");
    let agents = temp.path().join(".agents");
    let kimi = temp.path().join(".kimi-code");
    fs::create_dir_all(&codex).expect("codex dir");

    let error = kimi_sync(KimiSyncArgs {
        apply: false,
        scope: KimiSyncScope::GlobalOnly,
        codex_home: Some(codex),
        agents_home: Some(agents),
        kimi_home: Some(kimi),
        project_root: Some(temp.path().join("missing-project")),
        checked_at: None,
    })
    .expect_err("missing explicit project root rejected");

    assert!(error.to_string().contains("project root does not exist"));
}

fn write_skill(path: &Path, name: &str) {
    write_text(
        &path.join("SKILL.md"),
        &format!("---\nname: {name}\ndescription: Test skill.\n---\n\n# {name}\n"),
    );
}

fn write_text(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, text).expect("write text");
}
