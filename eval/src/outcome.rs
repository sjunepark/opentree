//! Outcome classification based on runner exit code and check results.

use serde::{Deserialize, Serialize};

use crate::judge::Judgment;

/// Final outcome of an eval run.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Runner completed and all checks passed.
    Success,
    /// Runner completed but some checks failed.
    Fail,
    /// Runner hit max_attempts on a node (exit code 3).
    Stuck,
    /// Runner internal error or crash.
    Error,
}

/// Classify the run outcome based on exit code and check results.
///
/// - Exit 0 + all checks pass → Success
/// - Exit 0 + any check fails → Fail
/// - Exit 3 → Stuck
/// - Other → Error
pub fn classify_outcome(runner_exit_code: Option<i32>, judgment: &Judgment) -> Outcome {
    match runner_exit_code {
        Some(0) => {
            if judgment.checks.iter().all(check_passed) {
                Outcome::Success
            } else {
                Outcome::Fail
            }
        }
        Some(3) => Outcome::Stuck,
        Some(1) | Some(2) | None => Outcome::Error,
        Some(_) => Outcome::Error,
    }
}

fn check_passed(check: &crate::judge::CheckOutcome) -> bool {
    match check {
        crate::judge::CheckOutcome::FileExists { passed, .. } => *passed,
        crate::judge::CheckOutcome::CommandSucceeds { passed, .. } => *passed,
        crate::judge::CheckOutcome::RunnerCompleted { passed, .. } => *passed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::judge::{CheckOutcome, Judgment};

    fn judgment(pass: bool) -> Judgment {
        Judgment {
            checks: vec![CheckOutcome::RunnerCompleted {
                passed: pass,
                exit_code: Some(0),
            }],
        }
    }

    #[test]
    fn success_when_complete_and_checks_pass() {
        let outcome = classify_outcome(Some(0), &judgment(true));
        assert_eq!(outcome, Outcome::Success);
    }

    #[test]
    fn fail_when_complete_but_checks_fail() {
        let outcome = classify_outcome(Some(0), &judgment(false));
        assert_eq!(outcome, Outcome::Fail);
    }

    #[test]
    fn stuck_when_runner_stuck() {
        let outcome = classify_outcome(Some(3), &judgment(true));
        assert_eq!(outcome, Outcome::Stuck);
    }

    #[test]
    fn error_when_runner_error_or_unknown() {
        let outcome = classify_outcome(Some(1), &judgment(true));
        assert_eq!(outcome, Outcome::Error);
        let outcome = classify_outcome(Some(2), &judgment(true));
        assert_eq!(outcome, Outcome::Error);
        let outcome = classify_outcome(None, &judgment(true));
        assert_eq!(outcome, Outcome::Error);
    }
}
