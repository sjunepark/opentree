//! Runner configuration merging.
//!
//! Applies case-specific overrides to the default runner configuration.

use anyhow::Result;
use runner::io::config::RunnerConfig;

use crate::case::CaseConfig;

/// Apply case configuration overrides to the base runner config.
pub fn apply_case_config(mut base: RunnerConfig, overrides: &CaseConfig) -> Result<RunnerConfig> {
    if let Some(max_iterations) = overrides.max_iterations {
        base.max_iterations = max_iterations;
    }
    if let Some(max_attempts_default) = overrides.max_attempts_default {
        base.max_attempts_default = max_attempts_default;
    }
    if let Some(guard) = &overrides.guard {
        base.guard.command = guard.command.clone();
    }
    base.validate()?;
    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case::GuardOverride;

    #[test]
    fn preserves_guard_when_no_override() {
        let base = RunnerConfig::default();
        let overrides = CaseConfig::default();
        let merged = apply_case_config(base.clone(), &overrides).expect("merge");
        assert_eq!(merged.guard.command, base.guard.command);
    }

    #[test]
    fn applies_guard_override() {
        let base = RunnerConfig::default();
        let overrides = CaseConfig {
            max_iterations: None,
            max_attempts_default: None,
            guard: Some(GuardOverride {
                command: vec!["make".to_string(), "ci".to_string()],
            }),
        };
        let merged = apply_case_config(base, &overrides).expect("merge");
        assert_eq!(merged.guard.command, vec!["make", "ci"]);
    }
}
