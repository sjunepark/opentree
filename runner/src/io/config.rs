//! Runner configuration stored under `.runner/state/config.toml`.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

/// Runner configuration (TOML).
///
/// This file is intended to be edited by humans and must remain stable and
/// automatable. Missing fields default to sensible MVP values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RunnerConfig {
    /// Default `max_attempts` to use when bootstrapping new nodes.
    pub max_attempts_default: u32,

    /// Total per-iteration wall-clock budget in seconds (agent + guards).
    pub iteration_timeout_secs: u64,

    /// Truncate executor stdout/stderr logs beyond this many bytes.
    pub executor_output_limit_bytes: usize,

    /// Truncate guard stdout/stderr logs beyond this many bytes.
    pub guard_output_limit_bytes: usize,

    pub guard: GuardConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GuardConfig {
    /// Command to execute for guard checks (e.g. `["just","ci"]`).
    pub command: Vec<String>,
}

impl Default for GuardConfig {
    fn default() -> Self {
        Self {
            command: vec!["just".to_string(), "ci".to_string()],
        }
    }
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            max_attempts_default: 3,
            iteration_timeout_secs: 30 * 60,
            executor_output_limit_bytes: 100_000,
            guard_output_limit_bytes: 100_000,
            guard: GuardConfig::default(),
        }
    }
}

impl RunnerConfig {
    pub fn validate(&self) -> Result<()> {
        if self.iteration_timeout_secs == 0 {
            return Err(anyhow!("iteration_timeout_secs must be > 0"));
        }
        if self.executor_output_limit_bytes == 0 {
            return Err(anyhow!("executor_output_limit_bytes must be > 0"));
        }
        if self.guard_output_limit_bytes == 0 {
            return Err(anyhow!("guard_output_limit_bytes must be > 0"));
        }
        if self.guard.command.is_empty() || self.guard.command[0].trim().is_empty() {
            return Err(anyhow!("guard.command must be a non-empty array"));
        }
        Ok(())
    }
}

/// Load config from a TOML file.
///
/// If the file is missing, returns `RunnerConfig::default()`.
pub fn load_config(path: &Path) -> Result<RunnerConfig> {
    if !path.exists() {
        let cfg = RunnerConfig::default();
        cfg.validate()?;
        return Ok(cfg);
    }
    let contents = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let cfg: RunnerConfig =
        toml::from_str(&contents).with_context(|| format!("parse {}", path.display()))?;
    cfg.validate()?;
    Ok(cfg)
}

/// Atomically write config to disk (temp file + rename).
pub fn write_config(path: &Path, cfg: &RunnerConfig) -> Result<()> {
    cfg.validate()?;
    let mut buf = toml::to_string_pretty(cfg).context("serialize config toml")?;
    buf.push('\n');
    write_atomic(path, &buf)
}

fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("config path missing parent {}", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("create directory {}", parent.display()))?;
    let tmp_path = path.with_extension("toml.tmp");
    fs::write(&tmp_path, contents)
        .with_context(|| format!("write temp config {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).with_context(|| format!("replace config {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_returns_default() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cfg = load_config(&temp.path().join("missing.toml")).expect("load");
        assert_eq!(cfg, RunnerConfig::default());
    }

    #[test]
    fn write_then_load_round_trips() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("config.toml");
        let cfg = RunnerConfig::default();
        write_config(&path, &cfg).expect("write");
        let loaded = load_config(&path).expect("load");
        assert_eq!(loaded, cfg);
    }
}
