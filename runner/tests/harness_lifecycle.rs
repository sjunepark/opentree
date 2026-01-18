//! Loop-level harness tests for full runner lifecycle scenarios.
//!
//! These tests drive `run_step` through multiple iterations to verify
//! end-to-end behavior: tree traversal, state transitions, pass propagation,
//! and loop termination.

use runner::core::selector::leftmost_open_leaf;
use runner::core::types::{AgentOutput, AgentStatus, GuardOutcome};
use runner::io::git::Git;
use runner::io::run_state::load_run_state;
use runner::io::tree_store::load_tree;
use runner::step::{StepConfig, run_step};
use runner::test_support::{
    ScriptedExec, ScriptedExecutor, ScriptedGuard, ScriptedGuardRunner, TestRepo, leaf,
    node_with_children,
};

/// Full lifecycle test: drives runner loop from start → retries → tree complete.
///
/// Tree structure:
/// ```text
/// root (passes=false)
/// ├── branch-a (order=0)
/// │   └── leaf-a (order=0)
/// └── leaf-b (order=1)
/// ```
///
/// Execution sequence:
/// 1. Iter 1: Select leaf-a → Retry (attempts=1)
/// 2. Iter 2: Select leaf-a → Done + Pass (leaf-a passes → branch-a auto-passes)
/// 3. Iter 3: Select leaf-b → Done + Pass (leaf-b passes → root auto-passes)
/// 4. Selection returns None (tree complete)
///
/// Tests: nested structure, recursive pass propagation, retry → done transition,
/// history accumulation, and loop termination.
#[test]
fn full_lifecycle_completes_tree_with_retries() {
    let repo = TestRepo::new().expect("repo");
    let root = repo.path();
    repo.start_run().expect("start");

    // Build nested tree: root -> branch-a -> leaf-a, root -> leaf-b
    let tree = node_with_children(
        "root",
        0,
        vec![
            node_with_children("branch-a", 0, vec![leaf("leaf-a", 0, false)]),
            leaf("leaf-b", 1, false),
        ],
    );
    repo.write_tree(&tree).expect("write tree");
    let git = Git::new(root);
    git.add_all().expect("git add");
    assert!(
        git.commit_staged("chore: setup nested tree")
            .expect("git commit")
    );

    // Queue: 3 executor responses (retry, done, done), 2 guard responses (pass, pass)
    let executor = ScriptedExecutor::new(vec![
        ScriptedExec {
            output: AgentOutput {
                status: AgentStatus::Retry,
                summary: "need another attempt".to_string(),
            },
            tree_update: None,
        },
        ScriptedExec {
            output: AgentOutput {
                status: AgentStatus::Done,
                summary: "leaf-a complete".to_string(),
            },
            tree_update: None,
        },
        ScriptedExec {
            output: AgentOutput {
                status: AgentStatus::Done,
                summary: "leaf-b complete".to_string(),
            },
            tree_update: None,
        },
    ]);
    let guard_runner = ScriptedGuardRunner::new(vec![
        ScriptedGuard {
            outcome: GuardOutcome::Pass,
            log: "leaf-a guard ok".to_string(),
        },
        ScriptedGuard {
            outcome: GuardOutcome::Pass,
            log: "leaf-b guard ok".to_string(),
        },
    ]);

    // Iteration 1: leaf-a → Retry
    let outcome1 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step1");
    assert_eq!(outcome1.selected_id, "leaf-a");
    assert_eq!(outcome1.status, AgentStatus::Retry);
    assert_eq!(outcome1.guard, GuardOutcome::Skipped);

    // Iteration 2: leaf-a → Done + Pass (branch-a auto-passes)
    let outcome2 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step2");
    assert_eq!(outcome2.selected_id, "leaf-a");
    assert_eq!(outcome2.status, AgentStatus::Done);
    assert_eq!(outcome2.guard, GuardOutcome::Pass);

    // Iteration 3: leaf-b → Done + Pass (root auto-passes)
    let outcome3 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step3");
    assert_eq!(outcome3.selected_id, "leaf-b");
    assert_eq!(outcome3.status, AgentStatus::Done);
    assert_eq!(outcome3.guard, GuardOutcome::Pass);

    // Tree should be complete (no open leaf)
    let final_tree = load_tree(
        &root.join(".runner/state/schema.json"),
        &root.join(".runner/state/tree.json"),
    )
    .expect("load tree");
    assert!(
        leftmost_open_leaf(&final_tree).is_none(),
        "tree should be complete"
    );

    // Verify final tree state
    assert!(final_tree.passes, "root should pass");
    assert!(
        final_tree.children[0].passes,
        "branch-a should pass (derived)"
    );
    assert!(
        final_tree.children[0].children[0].passes,
        "leaf-a should pass"
    );
    assert!(final_tree.children[1].passes, "leaf-b should pass");

    // Verify run_state
    let run_state = load_run_state(&root.join(".runner/state/run_state.json")).expect("run state");
    assert_eq!(run_state.next_iter, 4);
    assert_eq!(run_state.last_status, Some(AgentStatus::Done));
    assert_eq!(run_state.last_guard, Some(GuardOutcome::Pass));

    // Verify queues drained
    executor.assert_drained().expect("executor drained");
    guard_runner.assert_drained().expect("guard drained");
}
