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

/// Verifies same node is re-selected after guard failure, attempts accumulate, eventually passes.
///
/// Tree structure: single leaf (root)
///
/// Execution sequence:
/// 1. Iter 1: root → Done + Fail (attempts=1)
/// 2. Iter 2: root re-selected → Done + Fail (attempts=2)
/// 3. Iter 3: root re-selected → Done + Pass (passes=true)
///
/// Tests: guard failure triggers reselection, attempts increment on each Done+Fail,
/// eventual pass after repeated attempts.
#[test]
fn guard_fail_reselection_loop() {
    let repo = TestRepo::new().expect("repo");
    let root = repo.path();
    repo.start_run().expect("start");

    let executor = ScriptedExecutor::new(vec![
        ScriptedExec {
            output: AgentOutput {
                status: AgentStatus::Done,
                summary: "attempt 1".to_string(),
            },
            tree_update: None,
        },
        ScriptedExec {
            output: AgentOutput {
                status: AgentStatus::Done,
                summary: "attempt 2".to_string(),
            },
            tree_update: None,
        },
        ScriptedExec {
            output: AgentOutput {
                status: AgentStatus::Done,
                summary: "attempt 3 success".to_string(),
            },
            tree_update: None,
        },
    ]);
    let guard_runner = ScriptedGuardRunner::new(vec![
        ScriptedGuard {
            outcome: GuardOutcome::Fail,
            log: "fail 1".to_string(),
        },
        ScriptedGuard {
            outcome: GuardOutcome::Fail,
            log: "fail 2".to_string(),
        },
        ScriptedGuard {
            outcome: GuardOutcome::Pass,
            log: "pass".to_string(),
        },
    ]);

    // Iter 1: root → Done + Fail
    let outcome1 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step1");
    assert_eq!(outcome1.selected_id, "root");
    assert_eq!(outcome1.guard, GuardOutcome::Fail);
    let tree1 = repo.read_tree().expect("tree1");
    assert_eq!(tree1.attempts, 1);
    assert!(!tree1.passes);

    // Iter 2: root re-selected → Done + Fail
    let outcome2 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step2");
    assert_eq!(outcome2.selected_id, "root");
    assert_eq!(outcome2.guard, GuardOutcome::Fail);
    let tree2 = repo.read_tree().expect("tree2");
    assert_eq!(tree2.attempts, 2);
    assert!(!tree2.passes);

    // Iter 3: root re-selected → Done + Pass
    // Note: Done+Pass sets passes=true but does NOT increment attempts
    let outcome3 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step3");
    assert_eq!(outcome3.selected_id, "root");
    assert_eq!(outcome3.guard, GuardOutcome::Pass);
    let tree3 = repo.read_tree().expect("tree3");
    assert_eq!(tree3.attempts, 2); // Attempts stay at 2 (pass doesn't increment)
    assert!(tree3.passes);

    // Tree should be complete
    assert!(
        leftmost_open_leaf(&tree3).is_none(),
        "tree should be complete"
    );

    executor.assert_drained().expect("executor drained");
    guard_runner.assert_drained().expect("guard drained");
}

/// Verifies decomposition adds children that become selectable.
///
/// Tree structure: single leaf (root) → decomposed into child-1, child-2
///
/// Execution sequence:
/// 1. Iter 1: root → Decomposed (adds child-1, child-2)
/// 2. Iter 2: child-1 selected → Done + Pass
/// 3. Iter 3: child-2 selected → Done + Pass
/// 4. Selection returns None (root auto-passes via derivation)
///
/// Tests: decomposition children are selectable, order field respected,
/// parent passes derived from children.
#[test]
fn decomposition_changes_selection_path() {
    let repo = TestRepo::new().expect("repo");
    let root = repo.path();
    repo.start_run().expect("start");

    // Decomposed tree: root with two children (order 0 and 1)
    let decomposed_tree = node_with_children(
        "root",
        0,
        vec![leaf("child-1", 0, false), leaf("child-2", 1, false)],
    );

    let executor = ScriptedExecutor::new(vec![
        ScriptedExec {
            output: AgentOutput {
                status: AgentStatus::Decomposed,
                summary: "split into children".to_string(),
            },
            tree_update: Some(decomposed_tree),
        },
        ScriptedExec {
            output: AgentOutput {
                status: AgentStatus::Done,
                summary: "child-1 complete".to_string(),
            },
            tree_update: None,
        },
        ScriptedExec {
            output: AgentOutput {
                status: AgentStatus::Done,
                summary: "child-2 complete".to_string(),
            },
            tree_update: None,
        },
    ]);
    let guard_runner = ScriptedGuardRunner::new(vec![
        ScriptedGuard {
            outcome: GuardOutcome::Pass,
            log: "child-1 ok".to_string(),
        },
        ScriptedGuard {
            outcome: GuardOutcome::Pass,
            log: "child-2 ok".to_string(),
        },
    ]);

    // Iter 1: root → Decomposed
    let outcome1 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step1");
    assert_eq!(outcome1.selected_id, "root");
    assert_eq!(outcome1.status, AgentStatus::Decomposed);
    assert_eq!(outcome1.guard, GuardOutcome::Skipped);

    let tree1 = repo.read_tree().expect("tree1");
    assert_eq!(tree1.children.len(), 2);
    assert!(!tree1.passes); // Root not yet passed

    // Iter 2: child-1 selected (lower order)
    let outcome2 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step2");
    assert_eq!(outcome2.selected_id, "child-1");
    assert_eq!(outcome2.guard, GuardOutcome::Pass);

    let tree2 = repo.read_tree().expect("tree2");
    assert!(tree2.children[0].passes); // child-1 passed
    assert!(!tree2.children[1].passes); // child-2 not yet
    assert!(!tree2.passes); // Root not yet (child-2 incomplete)

    // Iter 3: child-2 selected
    let outcome3 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step3");
    assert_eq!(outcome3.selected_id, "child-2");
    assert_eq!(outcome3.guard, GuardOutcome::Pass);

    let tree3 = repo.read_tree().expect("tree3");
    assert!(tree3.children[0].passes);
    assert!(tree3.children[1].passes);
    assert!(tree3.passes); // Root auto-passes (all children passed)

    // Tree should be complete
    assert!(
        leftmost_open_leaf(&tree3).is_none(),
        "tree should be complete"
    );

    executor.assert_drained().expect("executor drained");
    guard_runner.assert_drained().expect("guard drained");
}

/// Verifies runner can resume mid-run from persisted state.
///
/// Setup: Create run_state with next_iter=3, tree with one passed node and one open node.
///
/// Execution sequence:
/// 1. Iter 3: remaining open leaf → Done + Pass
///
/// Tests: continues from iter 3 (not iter 1), previously passed nodes remain passed,
/// run_state.next_iter becomes 4.
#[test]
fn resumption_from_saved_state() {
    use runner::io::run_state::{RunState, write_run_state};

    let repo = TestRepo::new().expect("repo");
    let root = repo.path();
    let start = repo.start_run().expect("start");

    // Write tree with one passed child and one open child
    let partial_tree = node_with_children(
        "root",
        0,
        vec![
            leaf("passed-child", 0, true), // Already passed
            leaf("open-child", 1, false),  // Still open
        ],
    );
    repo.write_tree(&partial_tree).expect("write tree");

    // Write run_state with next_iter=3 (simulating 2 previous iterations)
    let run_state = RunState {
        run_id: Some(start.run_id.clone()),
        next_iter: 3,
        last_status: Some(AgentStatus::Done),
        last_summary: Some("previous work".to_string()),
        last_guard: Some(GuardOutcome::Pass),
    };
    write_run_state(&root.join(".runner/state/run_state.json"), &run_state).expect("write state");

    // Commit the partial state
    let git = Git::new(root);
    git.add_all().expect("git add");
    assert!(
        git.commit_staged("chore: partial progress")
            .expect("git commit")
    );

    let executor = ScriptedExecutor::new(vec![ScriptedExec {
        output: AgentOutput {
            status: AgentStatus::Done,
            summary: "open-child complete".to_string(),
        },
        tree_update: None,
    }]);
    let guard_runner = ScriptedGuardRunner::new(vec![ScriptedGuard {
        outcome: GuardOutcome::Pass,
        log: "ok".to_string(),
    }]);

    // Run one step - should resume at iter 3
    let outcome = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step");

    // Verify resumed from iter 3
    assert_eq!(outcome.iter, 3);
    assert_eq!(outcome.selected_id, "open-child");
    assert_eq!(outcome.guard, GuardOutcome::Pass);

    // Verify tree state
    let final_tree = repo.read_tree().expect("final tree");
    assert!(final_tree.children[0].passes, "passed-child still passed");
    assert!(final_tree.children[1].passes, "open-child now passed");
    assert!(final_tree.passes, "root auto-passes");

    // Verify run_state updated
    let final_state = repo.read_run_state().expect("final state");
    assert_eq!(final_state.next_iter, 4);

    // Tree should be complete
    assert!(
        leftmost_open_leaf(&final_tree).is_none(),
        "tree should be complete"
    );

    executor.assert_drained().expect("executor drained");
    guard_runner.assert_drained().expect("guard drained");
}
