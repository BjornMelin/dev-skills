//! Rule-level tests driven through the real CLI.
//!
//! Each test builds a throwaway estate on disk, runs `scan --format json`, and
//! asserts on rule ids. Every rule gets both a positive case and, where the
//! rule has a documented exemption, a negative case: a rule that cannot be
//! shown to fire is indistinguishable from one that silently never fires.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_claude-config-audit")
}

struct Estate {
    root: PathBuf,
}

impl Estate {
    fn new(tag: &str) -> Self {
        let mut root = std::env::temp_dir();
        root.push(format!(
            "cca-test-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("home/skills")).unwrap();
        Self { root }
    }

    fn home(&self) -> PathBuf {
        self.root.join("home")
    }

    /// Write a skill into an arbitrary directory and return its path.
    fn skill_at(dir: &Path, name: &str, frontmatter: &str, body: &str) -> PathBuf {
        let d = dir.join(name);
        fs::create_dir_all(&d).unwrap();
        let p = d.join("SKILL.md");
        fs::write(&p, format!("---\n{frontmatter}\n---\n\n{body}\n")).unwrap();
        p
    }

    fn skill(&self, name: &str, frontmatter: &str, body: &str) -> PathBuf {
        Self::skill_at(&self.home().join("skills"), name, frontmatter, body)
    }

    fn settings(&self, json: &str) {
        fs::write(self.home().join("settings.json"), json).unwrap();
    }

    fn scan(&self, extra: &[&str]) -> Vec<Finding> {
        let mut cmd = Command::new(bin());
        cmd.arg("scan")
            .arg("--home")
            .arg(self.home())
            .arg("--format")
            .arg("json");
        cmd.args(extra);
        let out = cmd.output().expect("run claude-config-audit");
        let text = String::from_utf8_lossy(&out.stdout);
        let v: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("bad json: {e}\n{text}"));
        v["findings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| Finding {
                id: f["id"].as_str().unwrap_or_default().to_string(),
                subject: f["subject"].as_str().unwrap_or_default().to_string(),
            })
            .collect()
    }
}

impl Drop for Estate {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct Finding {
    id: String,
    subject: String,
}

fn ids(findings: &[Finding]) -> Vec<&str> {
    findings.iter().map(|f| f.id.as_str()).collect()
}

fn has(findings: &[Finding], id: &str) -> bool {
    findings.iter().any(|f| f.id == id)
}

fn has_for(findings: &[Finding], id: &str, subject: &str) -> bool {
    findings
        .iter()
        .any(|f| f.id == id && f.subject.contains(subject))
}

#[test]
fn clean_estate_reports_nothing() {
    let e = Estate::new("clean");
    e.skill(
        "alpha",
        "name: alpha\ndescription: A short, honest description.",
        "body",
    );
    let f = e.scan(&[]);
    assert!(f.is_empty(), "expected no findings, got {:?}", ids(&f));
}

// Gated at the function, not just the setup: gating only the `symlink` call
// would leave the assertion running on a target that never created one.
#[cfg(unix)]
#[test]
fn broken_symlink_in_a_claude_root_is_high() {
    let e = Estate::new("brokenlink");
    std::os::unix::fs::symlink(
        e.home().join("skills/does-not-exist"),
        e.home().join("skills/dangling"),
    )
    .unwrap();
    let f = e.scan(&[]);
    assert!(has(&f, "links.broken-skill-symlink"), "got {:?}", ids(&f));
}

#[test]
fn description_over_cap_fires_even_when_model_invocation_is_disabled() {
    let e = Estate::new("overcap");
    let long = "x".repeat(1200);
    e.skill(
        "hog",
        &format!("name: hog\ndisable-model-invocation: true\ndescription: {long}"),
        "body",
    );
    let f = e.scan(&[]);
    // The flag removes a skill from the listing; it does not exempt the
    // frontmatter from the loader's cap.
    assert!(has(&f, "skill.description-over-cap"), "got {:?}", ids(&f));
    // ...but it does exempt it from the listing-cost rule.
    assert!(
        !has(&f, "skill.description-listing-hog"),
        "got {:?}",
        ids(&f)
    );
}

#[test]
fn description_length_counts_characters_not_bytes() {
    let e = Estate::new("unicode");
    // 900 multi-byte chars is under the 1024-char cap but over it in UTF-8
    // bytes, so a byte-based check would report a false positive.
    let accented = "é".repeat(900);
    e.skill(
        "accents",
        &format!("name: accents\ndescription: {accented}"),
        "body",
    );
    let f = e.scan(&[]);
    assert!(!has(&f, "skill.description-over-cap"), "got {:?}", ids(&f));
}

#[test]
fn negated_reasoning_guidance_is_not_a_risk() {
    let e = Estate::new("negation");
    e.skill(
        "safe",
        "name: safe\ndescription: Short description.",
        "Do not show your reasoning; return only conclusions.",
    );
    e.skill(
        "risky",
        "name: risky\ndescription: Short description.",
        "Always show your reasoning in the final answer.",
    );
    let f = e.scan(&[]);
    assert!(
        has_for(&f, "model.reasoning-extraction-risk", "risky"),
        "got {:?}",
        ids(&f)
    );
    assert!(
        !has_for(&f, "model.reasoning-extraction-risk", "safe"),
        "negated guidance must not be reported; got {:?}",
        ids(&f)
    );
}

#[test]
fn stale_override_is_reported_but_a_live_one_is_not() {
    let e = Estate::new("overrides");
    e.skill(
        "alpha",
        "name: alpha\ndescription: Short description.",
        "body",
    );
    e.settings(r#"{"skillOverrides":{"alpha":"off","ghost":"off"}}"#);
    let f = e.scan(&[]);
    assert!(has_for(&f, "overrides.stale", "ghost"), "got {:?}", ids(&f));
    assert!(
        !has_for(&f, "overrides.stale", "alpha"),
        "got {:?}",
        ids(&f)
    );
}

#[test]
fn override_targeting_a_disabled_plugin_is_distinguished_from_stale() {
    let e = Estate::new("disabledplugin");
    let gated = e
        .home()
        .join("plugins/marketplaces/mymarket/plugins/myplug/skills");
    fs::create_dir_all(&gated).unwrap();
    Estate::skill_at(
        &gated,
        "ghost-skill",
        "name: ghost-skill\ndescription: Short.",
        "body",
    );
    e.settings(
        r#"{"enabledPlugins":{"myplug@mymarket":false},
            "skillOverrides":{"ghost-skill":"on"}}"#,
    );
    let f = e.scan(&[]);
    assert!(
        has(&f, "overrides.targets-disabled-plugin"),
        "an override on a disabled plugin's skill has no effect; got {:?}",
        ids(&f)
    );
    assert!(
        !has(&f, "overrides.stale"),
        "the skill exists, so 'stale' would be the wrong diagnosis; got {:?}",
        ids(&f)
    );
}

#[test]
fn override_on_an_enabled_plugin_skill_is_clean() {
    let e = Estate::new("enabledplugin");
    let gated = e
        .home()
        .join("plugins/marketplaces/mymarket/plugins/myplug/skills");
    fs::create_dir_all(&gated).unwrap();
    Estate::skill_at(
        &gated,
        "real-skill",
        "name: real-skill\ndescription: Short.",
        "body",
    );
    e.settings(
        r#"{"enabledPlugins":{"myplug@mymarket":true},
            "skillOverrides":{"real-skill":"on"}}"#,
    );
    let f = e.scan(&[]);
    assert!(f.is_empty(), "expected no findings, got {:?}", ids(&f));
}

#[test]
fn mirror_drift_reports_only_skills_that_actually_differ() {
    let e = Estate::new("mirror");
    let mirror = e.root.join("mirror");
    fs::create_dir_all(&mirror).unwrap();

    // same on both sides
    e.skill(
        "same",
        "name: same\ndescription: Short description.",
        "identical",
    );
    Estate::skill_at(
        &mirror,
        "same",
        "name: same\ndescription: Short description.",
        "identical",
    );

    // authored edit that was never deployed
    e.skill(
        "drifted",
        "name: drifted\ndescription: Short description.",
        "old body",
    );
    Estate::skill_at(
        &mirror,
        "drifted",
        "name: drifted\ndescription: Short description.",
        "new body",
    );

    // authored but deliberately not installed: not drift
    Estate::skill_at(
        &mirror,
        "not-installed",
        "name: not-installed\ndescription: Short.",
        "body",
    );

    let f = e.scan(&["--mirror", mirror.to_str().unwrap()]);
    assert!(has_for(&f, "mirror.drift", "drifted"), "got {:?}", ids(&f));
    assert!(!has_for(&f, "mirror.drift", "same"), "got {:?}", ids(&f));
    assert!(
        !has_for(&f, "mirror.drift", "not-installed"),
        "a skill that was never installed is a choice, not drift; got {:?}",
        ids(&f)
    );
}

#[cfg(unix)]
#[test]
fn farm_dangling_and_duplicate_links_are_reported() {
    let e = Estate::new("farm");
    let real = e.root.join("shared/alpha");
    fs::create_dir_all(&real).unwrap();
    fs::write(
        real.join("SKILL.md"),
        "---\nname: alpha\ndescription: Short.\n---\nbody\n",
    )
    .unwrap();
    let farm = e.root.join("farm");
    fs::create_dir_all(&farm).unwrap();
    std::os::unix::fs::symlink(&real, farm.join("alpha")).unwrap();
    std::os::unix::fs::symlink(&real, farm.join("alpha-old-name")).unwrap();
    std::os::unix::fs::symlink(e.root.join("shared/gone"), farm.join("dangling")).unwrap();
    let f = e.scan(&["--farm", farm.to_str().unwrap()]);
    assert!(has(&f, "farm.broken-symlink"), "got {:?}", ids(&f));
    assert!(has(&f, "farm.duplicate-target"), "got {:?}", ids(&f));
}

#[test]
fn an_explicitly_named_root_that_does_not_exist_is_an_error() {
    // Silently skipping a mistyped --mirror or --farm would let the scan print
    // "No drift detected" having audited neither, which is exactly the silent
    // pass this tool exists to eliminate.
    let e = Estate::new("missingroot");
    e.skill("alpha", "name: alpha\ndescription: Short.", "body");

    for flag in ["--mirror", "--farm"] {
        let out = Command::new(bin())
            .arg("scan")
            .arg("--home")
            .arg(e.home())
            .arg(flag)
            .arg(e.root.join("nope"))
            .output()
            .expect("run");
        assert_eq!(
            out.status.code(),
            Some(1),
            "{flag} on a missing path must exit with the IO-error code"
        );
    }
}

#[test]
fn mirror_drift_covers_the_whole_skill_directory() {
    // references/, scripts/ and assets/ change a skill's behaviour as much as
    // its entrypoint. An entrypoint-only comparison missed a real 51-line
    // reference drift in the estate this tool audits.
    let e = Estate::new("mirrortree");
    let mirror = e.root.join("mirror");
    fs::create_dir_all(&mirror).unwrap();

    let front = "name: deep\ndescription: Identical entrypoint.";
    e.skill("deep", front, "same body");
    Estate::skill_at(&mirror, "deep", front, "same body");
    fs::create_dir_all(e.home().join("skills/deep/references")).unwrap();
    fs::create_dir_all(mirror.join("deep/references")).unwrap();
    fs::write(
        e.home().join("skills/deep/references/guide.md"),
        "old guidance\n",
    )
    .unwrap();
    fs::write(mirror.join("deep/references/guide.md"), "new guidance\n").unwrap();

    let f = e.scan(&["--mirror", mirror.to_str().unwrap()]);
    assert!(
        has_for(&f, "mirror.drift", "deep"),
        "a reference-only difference is still drift; got {:?}",
        ids(&f)
    );
}

#[test]
fn agent_shadowing_across_scopes_is_allowed_but_within_a_scope_is_not() {
    let e = Estate::new("agents");
    let user = e.home().join("agents");
    fs::create_dir_all(&user).unwrap();
    fs::write(
        user.join("a.md"),
        "---\nname: dup\ndescription: x\n---\nbody\n",
    )
    .unwrap();
    fs::write(
        user.join("b.md"),
        "---\nname: dup\ndescription: x\n---\nbody\n",
    )
    .unwrap();

    let project = e.root.join("proj");
    fs::create_dir_all(project.join(".claude/agents")).unwrap();
    fs::write(
        project.join(".claude/agents/shadow.md"),
        "---\nname: dup\ndescription: x\n---\nbody\n",
    )
    .unwrap();

    let f = e.scan(&["--project", project.to_str().unwrap()]);
    let dups: Vec<_> = f
        .iter()
        .filter(|x| x.id == "agent.duplicate-name")
        .collect();
    assert_eq!(
        dups.len(),
        1,
        "only the within-scope collision is a defect; got {:?}",
        ids(&f)
    );
    assert!(dups[0].subject.contains("user"), "got {}", dups[0].subject);
}

/// Every rule id the scanner can emit. Kept exhaustive on purpose: a partial
/// list lets a new rule ship undocumented, which is the drift this tool is
/// supposed to catch in other people's configuration.
const ALL_RULE_IDS: &[&str] = &[
    "links.broken-skill-symlink",
    "skill.oversized-skipped",
    "skill.body-too-long",
    "skill.missing-frontmatter",
    "skill.name-mismatch",
    "skill.description-over-cap",
    "skill.description-listing-hog",
    "model.reasoning-extraction-risk",
    "overrides.stale",
    "overrides.targets-disabled-plugin",
    "mirror.drift",
    "farm.broken-symlink",
    "farm.duplicate-target",
    "agent.duplicate-name",
    "agent.description-bloat",
    "guide.over-line-budget",
];

#[test]
fn doctor_lists_every_rule_the_scanner_can_emit() {
    let out = Command::new(bin()).arg("doctor").output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    for id in ALL_RULE_IDS {
        assert!(text.contains(id), "doctor is missing {id}");
    }
    // Catch the reverse too: a rule in the catalog that this list forgot.
    let listed = text
        .lines()
        .filter(|line| !line.starts_with(' ') && line.contains('.') && !line.trim().is_empty())
        .count();
    assert_eq!(
        listed,
        ALL_RULE_IDS.len(),
        "doctor lists {listed} rules but ALL_RULE_IDS has {}; update both",
        ALL_RULE_IDS.len()
    );
}
