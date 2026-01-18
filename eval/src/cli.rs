//! CLI command implementations.

use std::path::Path;

use anyhow::{Context, Result, bail};
use tracing::{debug, info};

use crate::case::{CaseFile, discover_cases};
use crate::report::aggregate;
use crate::run::run_case;

/// List all available cases.
pub fn list_cases(repo_root: &Path) -> Result<()> {
    let cases_dir = repo_root.join("eval").join("cases");
    let cases = discover_cases(&cases_dir)?;
    for case in cases {
        println!("{}", case.case.id);
    }
    Ok(())
}

/// Run a case by id (optionally multiple times).
pub fn run_case_by_id(repo_root: &Path, case_id: &str, runs: u32) -> Result<()> {
    let cases_dir = repo_root.join("eval").join("cases");
    let case_path = cases_dir.join(format!("{case_id}.toml"));
    if !case_path.exists() {
        bail!("case {} not found at {}", case_id, case_path.display());
    }
    let case = CaseFile::load(&case_path).context("load case")?;
    debug!(case_id, runs, "case loaded");

    info!(case_id, runs, "starting runs");
    for run_num in 1..=runs {
        debug!(case_id, run_num, runs, "starting run");
        let outcome = run_case(repo_root, &case_path, &case).context("run case")?;
        println!(
            "run: case={} eval_run_id={} outcome={:?} results={}",
            case_id,
            outcome.eval_run_id,
            outcome.outcome,
            outcome.results_dir.display()
        );
    }
    Ok(())
}

/// Show aggregated results for a case.
pub fn report_case(repo_root: &Path, case_id: &str) -> Result<()> {
    let results_dir = repo_root.join("eval").join("results").join(case_id);
    let (summary, warnings) = aggregate(&results_dir)?;
    println!("report: case={} runs={}", case_id, summary.runs);
    println!(
        "report: success={} fail={} stuck={} error={}",
        summary.success, summary.fail, summary.stuck, summary.error
    );
    if let Some(avg) = summary.avg_duration_secs {
        println!("report: avg_duration_secs={:.2}", avg);
    }
    for (label, (passed, total)) in summary.check_pass_rates {
        println!("report: check {} {}/{}", label, passed, total);
    }
    for warning in warnings {
        eprintln!("warning: {}", warning);
    }
    Ok(())
}

/// Remove workspaces and results for a case.
pub fn clean_case(repo_root: &Path, case_id: &str) -> Result<()> {
    let workspaces_dir = repo_root.join("eval").join("workspaces");
    let results_dir = repo_root.join("eval").join("results");

    if workspaces_dir.exists() {
        for entry in std::fs::read_dir(&workspaces_dir)
            .with_context(|| format!("read {}", workspaces_dir.display()))?
        {
            let entry = entry.context("read entry")?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(case_id) {
                std::fs::remove_dir_all(entry.path())
                    .with_context(|| format!("remove {}", entry.path().display()))?;
            }
        }
    }

    let case_results = results_dir.join(case_id);
    if case_results.exists() {
        std::fs::remove_dir_all(&case_results)
            .with_context(|| format!("remove {}", case_results.display()))?;
    }

    println!(
        "clean: case={} workspaces={} results={}",
        case_id,
        workspaces_dir.display(),
        case_results.display()
    );
    Ok(())
}
