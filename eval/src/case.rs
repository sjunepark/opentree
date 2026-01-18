//! Case file parsing and validation.
//!
//! Cases are TOML files defining goals and verification checks.
//! See `eval/cases/` for examples.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;

/// A parsed case file containing goal, config, and checks.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CaseFile {
    pub case: CaseMeta,
    #[serde(default)]
    pub config: CaseConfig,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub checks: Vec<Check>,
}

/// Case metadata: identifier and goal description.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CaseMeta {
    /// Unique identifier (slug format: `[a-z0-9_-]+`).
    pub id: String,
    /// Goal description passed to the runner.
    pub goal: String,
}

/// Runner configuration overrides for the case.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct CaseConfig {
    /// Maximum iterations before stopping the run.
    pub max_iterations: Option<u32>,
    /// Default max attempts for new leaves.
    pub max_attempts_default: Option<u32>,
    /// Guard command override (default: `["just", "ci"]`).
    pub guard: Option<GuardOverride>,
}

/// Custom guard command configuration.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GuardOverride {
    /// Command to run as guard (e.g., `["make", "ci"]`).
    pub command: Vec<String>,
}

/// Verification check to run after the runner loop completes.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Check {
    /// Check that a file exists in the workspace.
    FileExists { path: PathBuf },
    /// Check that a command exits successfully.
    CommandSucceeds { cmd: Vec<String> },
    /// Check that the runner completed (exit code 0).
    RunnerCompleted,
}

impl CaseFile {
    /// Load and validate a case file from the given path.
    pub fn load(path: &Path) -> Result<Self> {
        let contents =
            fs::read_to_string(path).with_context(|| format!("read case {}", path.display()))?;
        let case: CaseFile =
            toml::from_str(&contents).with_context(|| format!("parse case {}", path.display()))?;
        case.validate()
            .with_context(|| format!("validate case {}", path.display()))?;
        Ok(case)
    }

    #[cfg(test)]
    pub fn parse_str(contents: &str) -> Result<Self> {
        let case: CaseFile = toml::from_str(contents).context("parse case")?;
        case.validate()?;
        Ok(case)
    }

    fn validate(&self) -> Result<()> {
        validate_case_id(&self.case.id)?;
        if self.case.goal.trim().is_empty() {
            bail!("case.goal must be non-empty");
        }
        if let Some(max_iterations) = self.config.max_iterations
            && max_iterations == 0
        {
            bail!("config.max_iterations must be > 0");
        }
        if let Some(max_attempts_default) = self.config.max_attempts_default
            && max_attempts_default == 0
        {
            bail!("config.max_attempts_default must be > 0");
        }
        if let Some(guard) = &self.config.guard
            && (guard.command.is_empty() || guard.command[0].trim().is_empty())
        {
            bail!("config.guard.command must be a non-empty array");
        }
        if self.checks.is_empty() {
            bail!("checks must be a non-empty array");
        }
        for (index, check) in self.checks.iter().enumerate() {
            check
                .validate()
                .with_context(|| format!("checks[{}] invalid", index))?;
        }
        for (key, value) in &self.env {
            if key.trim().is_empty() {
                bail!("env key must be non-empty");
            }
            if value.is_empty() {
                bail!("env {} must be non-empty", key);
            }
        }
        Ok(())
    }
}

impl Check {
    fn validate(&self) -> Result<()> {
        match self {
            Check::FileExists { path } => {
                if path.as_os_str().is_empty() {
                    bail!("file_exists.path must be non-empty");
                }
            }
            Check::CommandSucceeds { cmd } => {
                if cmd.is_empty() || cmd[0].trim().is_empty() {
                    bail!("command_succeeds.cmd must be a non-empty array");
                }
            }
            Check::RunnerCompleted => {}
        }
        Ok(())
    }
}

/// Discover and load all case files from a directory.
///
/// Returns cases sorted by id. Errors if duplicate ids are found.
pub fn discover_cases(dir: &Path) -> Result<Vec<CaseFile>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut cases = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read cases dir {}", dir.display()))? {
        let entry = entry.context("read case entry")?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        cases.push(CaseFile::load(&path)?);
    }
    cases.sort_by(|left, right| left.case.id.cmp(&right.case.id));
    for pair in cases.windows(2) {
        if pair[0].case.id == pair[1].case.id {
            return Err(anyhow!("duplicate case.id {}", pair[0].case.id));
        }
    }
    Ok(cases)
}

fn validate_case_id(id: &str) -> Result<()> {
    if id.trim().is_empty() {
        bail!("case.id must be non-empty");
    }
    if id.contains('/') || id.contains('\\') {
        bail!("case.id must not contain path separators");
    }
    if id.contains("..") {
        bail!("case.id must not contain '..'");
    }
    if !id
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        bail!("case.id must use [a-z0-9_-] only");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_case() {
        let input = r#"
[case]
id = "calculator-go"
goal = "do the thing"

[config]
max_iterations = 12
max_attempts_default = 2

[config.guard]
command = ["just", "ci"]

[[checks]]
type = "file_exists"
path = "main.go"

[[checks]]
type = "command_succeeds"
cmd = ["go", "build", "."]

[[checks]]
type = "runner_completed"
"#;
        let case = CaseFile::parse_str(input).expect("case parses");
        assert_eq!(case.case.id, "calculator-go");
        assert_eq!(case.checks.len(), 3);
    }

    #[test]
    fn rejects_invalid_id() {
        let input = r#"
[case]
id = "bad/id"
goal = "do the thing"

[[checks]]
type = "runner_completed"
"#;
        let err = CaseFile::parse_str(input).expect_err("invalid id");
        assert!(err.to_string().contains("case.id"));
    }

    #[test]
    fn rejects_malformed_checks() {
        let input = r#"
[case]
id = "calculator-go"
goal = "do the thing"

[[checks]]
type = "command_succeeds"
cmd = []
"#;
        let _err = CaseFile::parse_str(input).expect_err("invalid check");
    }
}
