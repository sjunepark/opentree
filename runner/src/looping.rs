//! Multi-iteration looping helper for `runner loop`.

use std::path::Path;

use anyhow::{Context, Result, anyhow};

use crate::io::executor::Executor;
use crate::io::guards::GuardRunner;
use crate::io::init::RunnerPaths;
use crate::io::run_state::load_run_state;
use crate::select::{SelectOutcome, select_from_root};
use crate::step::{MaxIterationsExceededError, StepConfig, StepOutcome, StuckLeafError, run_step};

/// Reason why `run_loop` stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopStop {
    /// The tree is complete (no open leaves).
    Complete,
    /// The next selected leaf is stuck (attempts exhausted).
    Stuck {
        id: String,
        path: String,
        attempts: u32,
        max_attempts: u32,
    },
    /// The run exceeded the configured `max_iterations`.
    MaxIterationsExceeded { next_iter: u32, max_iterations: u32 },
}

/// Summary of a loop invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopOutcome {
    pub run_id: String,
    pub started_at_iter: u32,
    pub steps_executed: u32,
    pub stop: LoopStop,
}

/// Run `runner step` repeatedly until the tree completes, a stuck leaf is selected,
/// or a configured iteration limit is reached.
///
/// This stops immediately on any other error (git, executor, guards, schema/invariant violations).
pub fn run_loop<E: Executor, G: GuardRunner, F: FnMut(&StepOutcome)>(
    root: &Path,
    executor: &E,
    guard_runner: &G,
    step_config: &StepConfig,
    mut on_step: F,
) -> Result<LoopOutcome> {
    let paths = RunnerPaths::new(root);
    let run_state = load_run_state(&paths.run_state_path)
        .with_context(|| format!("load {}", paths.run_state_path.display()))?;
    let run_id = run_state
        .run_id
        .ok_or_else(|| anyhow!("missing run id (run `runner start` first)"))?;
    let started_at_iter = run_state.next_iter;

    let mut steps_executed = 0u32;
    loop {
        // Pre-check: exit early on Complete/Stuck without the heavier setup that
        // run_step performs (git policy, config, run_state). Redundant when Open,
        // but agent execution dominates runtime so the extra tree load is negligible.
        match select_from_root(root)? {
            SelectOutcome::Complete => {
                return Ok(LoopOutcome {
                    run_id: run_id.clone(),
                    started_at_iter,
                    steps_executed,
                    stop: LoopStop::Complete,
                });
            }
            SelectOutcome::Stuck(leaf) => {
                return Ok(LoopOutcome {
                    run_id: run_id.clone(),
                    started_at_iter,
                    steps_executed,
                    stop: LoopStop::Stuck {
                        id: leaf.id,
                        path: leaf.path,
                        attempts: leaf.attempts,
                        max_attempts: leaf.max_attempts,
                    },
                });
            }
            SelectOutcome::Open(_) => {}
        }

        match run_step(root, executor, guard_runner, step_config) {
            Ok(step) => {
                steps_executed += 1;
                on_step(&step);
            }
            Err(err) => {
                if let Some(stuck) = err.downcast_ref::<StuckLeafError>() {
                    return Ok(LoopOutcome {
                        run_id: run_id.clone(),
                        started_at_iter,
                        steps_executed,
                        stop: LoopStop::Stuck {
                            id: stuck.id.clone(),
                            path: stuck.path.clone(),
                            attempts: stuck.attempts,
                            max_attempts: stuck.max_attempts,
                        },
                    });
                }
                if let Some(limit) = err.downcast_ref::<MaxIterationsExceededError>() {
                    return Ok(LoopOutcome {
                        run_id: run_id.clone(),
                        started_at_iter,
                        steps_executed,
                        stop: LoopStop::MaxIterationsExceeded {
                            next_iter: limit.next_iter,
                            max_iterations: limit.max_iterations,
                        },
                    });
                }
                return Err(err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{AgentOutput, AgentStatus, GuardOutcome};
    use crate::io::config::{RunnerConfig, write_config};
    use crate::io::git::Git;
    use crate::test_support::{
        ScriptedExec, ScriptedExecutor, ScriptedGuardRunner, ScriptedOutput, TestRepo,
    };
    use crate::tree::NodeNext;

    #[test]
    fn loop_stops_on_complete_without_calling_step_again() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();
        repo.start_run().expect("start");

        // Keep config permissive for the test.
        write_config(
            &root.join(".runner/state/config.toml"),
            &RunnerConfig {
                max_iterations: 10,
                ..RunnerConfig::default()
            },
        )
        .expect("write config");
        let git = Git::new(root);
        git.add_all().expect("git add");
        assert!(git.commit_staged("chore: set config").expect("git commit"));

        let mut tree = repo.read_tree().expect("read tree");
        tree.next = NodeNext::Execute;
        repo.write_tree(&tree).expect("write tree");
        let git = Git::new(root);
        git.add_all().expect("git add");
        assert!(
            git.commit_staged("chore: set root next")
                .expect("git commit")
        );

        let executor = ScriptedExecutor::new(vec![ScriptedExec {
            output: ScriptedOutput::AgentOutput(AgentOutput {
                status: AgentStatus::Done,
                summary: "done".to_string(),
            }),
            tree_update: None,
        }]);
        let guard_runner = ScriptedGuardRunner::new(vec![crate::test_support::ScriptedGuard {
            outcome: GuardOutcome::Pass,
            log: "ok".to_string(),
        }]);

        let outcome = run_loop(
            root,
            &executor,
            &guard_runner,
            &StepConfig::default(),
            |_| {},
        )
        .expect("loop");

        assert_eq!(outcome.steps_executed, 1);
        assert_eq!(outcome.stop, LoopStop::Complete);
    }

    #[test]
    fn loop_stops_on_max_iterations_exceeded() {
        let repo = TestRepo::new().expect("repo");
        let root = repo.root();
        repo.start_run().expect("start");

        write_config(
            &root.join(".runner/state/config.toml"),
            &RunnerConfig {
                max_iterations: 1,
                ..RunnerConfig::default()
            },
        )
        .expect("write config");
        let git = Git::new(root);
        git.add_all().expect("git add");
        assert!(git.commit_staged("chore: set config").expect("git commit"));

        let mut tree = repo.read_tree().expect("read tree");
        tree.next = NodeNext::Execute;
        repo.write_tree(&tree).expect("write tree");
        let git = Git::new(root);
        git.add_all().expect("git add");
        assert!(
            git.commit_staged("chore: set root next")
                .expect("git commit")
        );

        let executor = ScriptedExecutor::new(vec![ScriptedExec {
            output: ScriptedOutput::AgentOutput(AgentOutput {
                status: AgentStatus::Retry,
                summary: "keep going".to_string(),
            }),
            tree_update: None,
        }]);
        let guard_runner = ScriptedGuardRunner::new(Vec::new());

        let outcome = run_loop(
            root,
            &executor,
            &guard_runner,
            &StepConfig::default(),
            |_| {},
        )
        .expect("loop");

        assert_eq!(outcome.steps_executed, 1);
        assert_eq!(
            outcome.stop,
            LoopStop::MaxIterationsExceeded {
                next_iter: 2,
                max_iterations: 1
            }
        );
    }
}
