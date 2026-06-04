use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::KimiSyncReport;
use super::config::project_hash;
use crate::write_json;

pub(super) fn write_kimi_mirror(report: &KimiSyncReport) -> Result<()> {
    fs::create_dir_all(&report.sync_root)
        .with_context(|| format!("failed to create {}", report.sync_root.display()))?;
    assert_safe_sync_path(&report.sync_root, &report.mirror_root)?;
    let tmp_root = report.sync_root.join("tmp").join(format!(
        "{}-{}",
        project_hash(report.project_root.as_deref()),
        process_id()
    ));
    if tmp_root.exists() {
        fs::remove_dir_all(&tmp_root)
            .with_context(|| format!("failed to remove {}", tmp_root.display()))?;
    }
    let tmp_skills = tmp_root.join("skills");
    fs::create_dir_all(&tmp_skills)
        .with_context(|| format!("failed to create {}", tmp_skills.display()))?;
    for skill in &report.included {
        let link_path = tmp_skills.join(&skill.name);
        create_dir_symlink(&skill.source_path, &link_path)?;
    }
    write_json(tmp_root.join("manifest.json"), report)?;

    if let Some(parent) = report.mirror_root.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let backup_root = backup_mirror_root(&report.mirror_root);
    if backup_root.exists() {
        fs::remove_dir_all(&backup_root)
            .with_context(|| format!("failed to remove stale {}", backup_root.display()))?;
    }
    if report.mirror_root.exists() {
        fs::rename(&report.mirror_root, &backup_root).with_context(|| {
            format!(
                "failed to move existing mirror {} into {}",
                report.mirror_root.display(),
                backup_root.display()
            )
        })?;
    }
    if let Err(error) = fs::rename(&tmp_root, &report.mirror_root) {
        if backup_root.exists() {
            let _ = fs::rename(&backup_root, &report.mirror_root);
        }
        return Err(error).with_context(|| {
            format!(
                "failed to move {} into {}",
                tmp_root.display(),
                report.mirror_root.display()
            )
        });
    }
    if backup_root.exists() {
        fs::remove_dir_all(&backup_root)
            .with_context(|| format!("failed to remove backup {}", backup_root.display()))?;
    }
    Ok(())
}

fn assert_safe_sync_path(sync_root: &Path, path: &Path) -> Result<()> {
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        bail!("refusing unsafe generated mirror path: {}", path.display());
    }
    if !path.starts_with(sync_root) {
        bail!(
            "refusing to write outside Kimi sync root: {} is not under {}",
            path.display(),
            sync_root.display()
        );
    }
    if fs::symlink_metadata(sync_root)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        bail!("refusing symlinked Kimi sync root: {}", sync_root.display());
    }
    let canonical_sync_root = fs::canonicalize(sync_root)
        .with_context(|| format!("failed to canonicalize {}", sync_root.display()))?;
    let existing_ancestor = nearest_existing_ancestor(path);
    let canonical_ancestor = fs::canonicalize(existing_ancestor).with_context(|| {
        format!(
            "failed to canonicalize generated mirror ancestor {}",
            existing_ancestor.display()
        )
    })?;
    if !canonical_ancestor.starts_with(&canonical_sync_root) {
        bail!(
            "refusing generated mirror path outside canonical Kimi sync root: {} is not under {}",
            canonical_ancestor.display(),
            canonical_sync_root.display()
        );
    }
    Ok(())
}

fn nearest_existing_ancestor(path: &Path) -> &Path {
    path.ancestors()
        .find(|ancestor| ancestor.exists())
        .unwrap_or(path)
}

fn backup_mirror_root(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("mirror");
    path.with_file_name(format!("{file_name}.old"))
}

fn process_id() -> u32 {
    std::process::id()
}

#[cfg(unix)]
fn create_dir_symlink(source: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(source, link).with_context(|| {
        format!(
            "failed to symlink {} -> {}",
            link.display(),
            source.display()
        )
    })
}

#[cfg(windows)]
fn create_dir_symlink(source: &Path, link: &Path) -> Result<()> {
    std::os::windows::fs::symlink_dir(source, link).with_context(|| {
        format!(
            "failed to symlink {} -> {}",
            link.display(),
            source.display()
        )
    })
}
