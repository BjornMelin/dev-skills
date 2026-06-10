use crate::types::ReleaseSyncReport;
use anyhow::{Context, Result};
use directories::BaseDirs;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct PlatformPaths {
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl PlatformPaths {
    pub fn discover() -> Result<Self> {
        let base = BaseDirs::new().context("failed to discover base directories")?;
        Ok(Self {
            config_dir: base.config_dir().join("dev-skills").join("bun-platform"),
            state_dir: base
                .state_dir()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| base.data_local_dir().to_path_buf())
                .join("dev-skills")
                .join("bun-platform"),
            cache_dir: base.cache_dir().join("dev-skills").join("bun-platform"),
        })
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.state_dir)?;
        fs::create_dir_all(self.scan_cache_dir())?;
        fs::create_dir_all(self.rollback_dir())?;
        fs::create_dir_all(self.reports_dir())?;
        Ok(())
    }

    pub fn scan_cache_dir(&self) -> PathBuf {
        self.cache_dir.join("scan-cache")
    }

    pub fn rollback_dir(&self) -> PathBuf {
        self.state_dir.join("rollbacks")
    }

    pub fn reports_dir(&self) -> PathBuf {
        self.state_dir.join("reports")
    }

    /// Build the scan-cache path for a canonical repository root. Callers must
    /// pass an absolute, resolved root or equivalent repos can hash differently.
    pub fn cache_file_for(&self, root: &Path, fingerprint: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(root.to_string_lossy().as_bytes());
        let root_hash = format!("{:x}", hasher.finalize());
        self.scan_cache_dir()
            .join(root_hash)
            .join(format!("{fingerprint}.json"))
    }

    pub fn write_cache(&self, root: &Path, fingerprint: &str, findings_json: &str) -> Result<()> {
        let path = self.cache_file_for(root, fingerprint);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, findings_json)?;
        Ok(())
    }

    pub fn read_cache(&self, root: &Path, fingerprint: &str) -> Result<Option<String>> {
        let path = self.cache_file_for(root, fingerprint);
        if !path.exists() {
            return Ok(None);
        }
        fs::read_to_string(&path)
            .with_context(|| format!("failed to read cache {}", path.display()))
            .map(Some)
    }

    pub fn write_release_report(&self, report: &ReleaseSyncReport) -> Result<PathBuf> {
        let path = self.reports_dir().join("release-sync-report.json");
        fs::write(&path, serde_json::to_vec_pretty(report)?)?;
        Ok(path)
    }
}
