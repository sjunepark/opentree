use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::judge::{CheckOutcome, Judgment};
use crate::results::EvalMeta;

#[derive(Debug, Default)]
pub struct ReportSummary {
    pub runs: usize,
    pub success: usize,
    pub fail: usize,
    pub stuck: usize,
    pub error: usize,
    pub avg_duration_secs: Option<f64>,
    pub check_pass_rates: BTreeMap<String, (usize, usize)>,
}

pub fn load_run_dirs(case_results_dir: &Path) -> Result<Vec<PathBuf>> {
    if !case_results_dir.exists() {
        return Ok(Vec::new());
    }
    let mut dirs = Vec::new();
    for entry in fs::read_dir(case_results_dir)
        .with_context(|| format!("read {}", case_results_dir.display()))?
    {
        let entry = entry.context("read entry")?;
        if entry.path().is_dir() {
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    Ok(dirs)
}

pub fn aggregate(case_results_dir: &Path) -> Result<(ReportSummary, Vec<String>)> {
    let mut summary = ReportSummary::default();
    let mut warnings = Vec::new();

    for run_dir in load_run_dirs(case_results_dir)? {
        let meta_path = run_dir.join("meta.json");
        let checks_path = run_dir.join("checks.json");

        let meta: EvalMeta = match fs::read_to_string(&meta_path)
            .with_context(|| format!("read {}", meta_path.display()))
            .and_then(|contents| serde_json::from_str(&contents).context("parse meta"))
        {
            Ok(meta) => meta,
            Err(err) => {
                warnings.push(format!(
                    "skip {}: meta.json invalid ({err})",
                    run_dir.display()
                ));
                continue;
            }
        };

        let judgment: Judgment = match fs::read_to_string(&checks_path)
            .with_context(|| format!("read {}", checks_path.display()))
            .and_then(|contents| serde_json::from_str(&contents).context("parse checks"))
        {
            Ok(checks) => checks,
            Err(err) => {
                warnings.push(format!(
                    "skip {}: checks.json invalid ({err})",
                    run_dir.display()
                ));
                continue;
            }
        };

        summary.runs += 1;
        match meta.outcome {
            Some(crate::outcome::Outcome::Success) => summary.success += 1,
            Some(crate::outcome::Outcome::Fail) => summary.fail += 1,
            Some(crate::outcome::Outcome::Stuck) => summary.stuck += 1,
            Some(crate::outcome::Outcome::Error) | None => summary.error += 1,
        }

        summary.avg_duration_secs = Some(match summary.avg_duration_secs {
            None => meta.duration_secs,
            Some(avg) => {
                let total = avg * (summary.runs as f64 - 1.0) + meta.duration_secs;
                total / summary.runs as f64
            }
        });

        update_check_pass_rates(&mut summary.check_pass_rates, &judgment);
    }

    Ok((summary, warnings))
}

fn update_check_pass_rates(stats: &mut BTreeMap<String, (usize, usize)>, judgment: &Judgment) {
    for check in &judgment.checks {
        let label = label_for_check(check);
        let entry = stats.entry(label).or_insert((0, 0));
        if check_passed(check) {
            entry.0 += 1;
        }
        entry.1 += 1;
    }
}

fn label_for_check(check: &CheckOutcome) -> String {
    match check {
        CheckOutcome::FileExists { path, .. } => format!("file_exists({path})"),
        CheckOutcome::CommandSucceeds { cmd, .. } => {
            format!("command_succeeds({})", cmd.join(" "))
        }
        CheckOutcome::RunnerCompleted { .. } => "runner_completed".to_string(),
    }
}

fn check_passed(check: &CheckOutcome) -> bool {
    match check {
        CheckOutcome::FileExists { passed, .. } => *passed,
        CheckOutcome::CommandSucceeds { passed, .. } => *passed,
        CheckOutcome::RunnerCompleted { passed, .. } => *passed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::judge::Judgment;
    use crate::outcome::Outcome;
    use tempfile::tempdir;

    fn write_meta(path: &Path, outcome: Outcome, duration: f64) {
        let meta = EvalMeta {
            case_id: "case".to_string(),
            eval_run_id: "run".to_string(),
            case_hash: "hash".to_string(),
            runner_git_sha: None,
            runner_binary: "/bin/runner".to_string(),
            runner_run_id: None,
            outcome: Some(outcome),
            start_time: "now".to_string(),
            end_time: "later".to_string(),
            duration_secs: duration,
            exit_code: Some(0),
            workspace: "/tmp".to_string(),
            errors: Vec::new(),
        };
        let contents = serde_json::to_string_pretty(&meta).expect("meta json");
        fs::write(path, format!("{contents}\n")).expect("write meta");
    }

    fn write_checks(path: &Path, passed: bool) {
        let judgment = Judgment {
            checks: vec![CheckOutcome::RunnerCompleted {
                passed,
                exit_code: Some(0),
            }],
        };
        let contents = serde_json::to_string_pretty(&judgment).expect("checks json");
        fs::write(path, format!("{contents}\n")).expect("write checks");
    }

    #[test]
    fn aggregates_runs() {
        let temp = tempdir().expect("tempdir");
        let run1 = temp.path().join("run1");
        let run2 = temp.path().join("run2");
        fs::create_dir_all(&run1).expect("run1");
        fs::create_dir_all(&run2).expect("run2");

        write_meta(&run1.join("meta.json"), Outcome::Success, 5.0);
        write_checks(&run1.join("checks.json"), true);

        write_meta(&run2.join("meta.json"), Outcome::Fail, 15.0);
        write_checks(&run2.join("checks.json"), false);

        let (summary, warnings) = aggregate(temp.path()).expect("aggregate");
        assert!(warnings.is_empty());
        assert_eq!(summary.runs, 2);
        assert_eq!(summary.success, 1);
        assert_eq!(summary.fail, 1);
        assert_eq!(summary.avg_duration_secs.unwrap(), 10.0);

        let key = "runner_completed".to_string();
        assert_eq!(summary.check_pass_rates.get(&key), Some(&(1, 2)));
    }
}
