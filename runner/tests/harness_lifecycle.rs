//! Loop-level harness tests for full runner lifecycle scenarios.
//!
//! These tests drive `run_step` through multiple iterations to verify
//! end-to-end behavior: tree traversal, state transitions, pass propagation,
//! and loop termination.

use std::fs;

use runner::core::types::{AgentOutput, AgentStatus, GuardOutcome};
use runner::io::git::Git;
use runner::io::run_state::load_run_state;
use runner::io::tree_store::load_tree;
use runner::step::{StepConfig, run_step};
use runner::test_support::{
    ScriptedExec, ScriptedExecutor, ScriptedGuard, ScriptedGuardRunner, TestRepo, leaf,
    node_with_children,
};
use runner::tree::Node;

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
/// 4. Iter 4: `run_step` errors (tree complete)
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

    let tree_after_1 = repo.read_tree().expect("tree after 1");
    assert_eq!(must_find(&tree_after_1, "leaf-a").attempts, 1);
    assert!(!must_find(&tree_after_1, "leaf-a").passes);
    assert_eq!(must_find(&tree_after_1, "leaf-b").attempts, 0);
    assert!(!must_find(&tree_after_1, "leaf-b").passes);

    // Iteration 2: leaf-a → Done + Pass (branch-a auto-passes)
    let outcome2 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step2");
    assert_eq!(outcome2.selected_id, "leaf-a");
    assert_eq!(outcome2.status, AgentStatus::Done);
    assert_eq!(outcome2.guard, GuardOutcome::Pass);

    let history_md = fs::read_to_string(root.join(".runner/context/history.md"))
        .expect("read .runner/context/history.md");
    assert!(
        history_md.contains("need another attempt"),
        "history should include previous retry summary"
    );

    let tree_after_2 = repo.read_tree().expect("tree after 2");
    assert!(must_find(&tree_after_2, "leaf-a").passes);
    assert!(must_find(&tree_after_2, "branch-a").passes);
    assert!(!must_find(&tree_after_2, "leaf-b").passes);

    // Iteration 3: leaf-b → Done + Pass (root auto-passes)
    let outcome3 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step3");
    assert_eq!(outcome3.selected_id, "leaf-b");
    assert_eq!(outcome3.status, AgentStatus::Done);
    assert_eq!(outcome3.guard, GuardOutcome::Pass);

    // Tree should be complete
    let final_tree = load_tree(
        &root.join(".runner/state/schema.json"),
        &root.join(".runner/state/tree.json"),
    )
    .expect("load tree");

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

    // Iteration 4: tree complete → `run_step` errors before invoking executor/guards.
    let empty_executor = ScriptedExecutor::new(Vec::new());
    let empty_guard_runner = ScriptedGuardRunner::new(Vec::new());
    let err = run_step(
        root,
        &empty_executor,
        &empty_guard_runner,
        &StepConfig::default(),
    )
    .expect_err("tree complete should error");
    assert!(err.to_string().contains("tree already complete"));
}

/// Verifies same node is re-selected after guard failure even with a sibling leaf available.
///
/// Tree structure:
/// ```text
/// root
/// ├── leaf-a (order=0)
/// └── leaf-b (order=1)
/// ```
///
/// Execution sequence:
/// 1. Iter 1: leaf-a → Done + Fail (attempts=1)
/// 2. Iter 2: leaf-a re-selected → Done + Fail (attempts=2)
/// 3. Iter 3: leaf-a re-selected → Done + Pass (leaf-a passes)
/// 4. Iter 4: leaf-b selected → Done + Pass (root auto-passes)
/// 5. Iter 5: `run_step` errors (tree complete)
///
/// Tests: guard failure triggers reselection, attempts increment on each Done+Fail,
/// and selection does not skip ahead to a sibling while the leftmost leaf is still open.
#[test]
fn guard_fail_reselection_loop() {
    let repo = TestRepo::new().expect("repo");
    let root = repo.path();
    repo.start_run().expect("start");

    // Two-leaf tree, ensures guard failures do not cause a skip to leaf-b.
    let tree = node_with_children(
        "root",
        0,
        vec![leaf("leaf-a", 0, false), leaf("leaf-b", 1, false)],
    );
    repo.write_tree(&tree).expect("write tree");
    let git = Git::new(root);
    git.add_all().expect("git add");
    assert!(
        git.commit_staged("chore: setup two-leaf tree")
            .expect("git commit")
    );

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
        ScriptedGuard {
            outcome: GuardOutcome::Pass,
            log: "leaf-b pass".to_string(),
        },
    ]);

    // Iter 1: leaf-a → Done + Fail
    let outcome1 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step1");
    assert_eq!(outcome1.selected_id, "leaf-a");
    assert_eq!(outcome1.guard, GuardOutcome::Fail);
    let tree1 = repo.read_tree().expect("tree1");
    assert_eq!(must_find(&tree1, "leaf-a").attempts, 1);
    assert!(!must_find(&tree1, "leaf-a").passes);
    assert!(!must_find(&tree1, "leaf-b").passes);

    // Iter 2: leaf-a re-selected → Done + Fail
    let outcome2 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step2");
    assert_eq!(outcome2.selected_id, "leaf-a");
    assert_eq!(outcome2.guard, GuardOutcome::Fail);
    let tree2 = repo.read_tree().expect("tree2");
    assert_eq!(must_find(&tree2, "leaf-a").attempts, 2);
    assert!(!must_find(&tree2, "leaf-a").passes);
    assert!(!must_find(&tree2, "leaf-b").passes);

    // Iter 3: leaf-a re-selected → Done + Pass
    // Note: Done+Pass sets passes=true but does NOT increment attempts
    let outcome3 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step3");
    assert_eq!(outcome3.selected_id, "leaf-a");
    assert_eq!(outcome3.guard, GuardOutcome::Pass);
    let tree3 = repo.read_tree().expect("tree3");
    assert_eq!(must_find(&tree3, "leaf-a").attempts, 2); // pass doesn't increment
    assert!(must_find(&tree3, "leaf-a").passes);
    assert!(!must_find(&tree3, "leaf-b").passes);

    // Iter 4: leaf-b selected → Done + Pass (root auto-passes)
    let outcome4 = run_step(root, &executor, &guard_runner, &StepConfig::default()).expect("step4");
    assert_eq!(outcome4.selected_id, "leaf-b");
    assert_eq!(outcome4.guard, GuardOutcome::Pass);
    let tree4 = repo.read_tree().expect("tree4");
    assert!(tree4.passes, "root should auto-pass after both leaves pass");

    executor.assert_drained().expect("executor drained");
    guard_runner.assert_drained().expect("guard drained");

    // Iter 5: tree complete → `run_step` errors.
    let empty_executor = ScriptedExecutor::new(Vec::new());
    let empty_guard_runner = ScriptedGuardRunner::new(Vec::new());
    let err = run_step(
        root,
        &empty_executor,
        &empty_guard_runner,
        &StepConfig::default(),
    )
    .expect_err("tree complete should error");
    assert!(err.to_string().contains("tree already complete"));
}

/// Verifies decomposition adds children that become selectable.
///
/// Tree structure: single leaf (root) → decomposed into child-1, child-2
///
/// Execution sequence:
/// 1. Iter 1: root → Decomposed (adds child-1, child-2)
/// 2. Iter 2: child-1 selected → Done + Pass
/// 3. Iter 3: child-2 selected → Done + Pass
/// 4. Iter 4: `run_step` errors (tree complete)
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
    assert_eq!(tree1.children[0].id, "child-1");
    assert_eq!(tree1.children[1].id, "child-2");
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

    executor.assert_drained().expect("executor drained");
    guard_runner.assert_drained().expect("guard drained");

    // Iter 4: tree complete → `run_step` errors.
    let empty_executor = ScriptedExecutor::new(Vec::new());
    let empty_guard_runner = ScriptedGuardRunner::new(Vec::new());
    let err = run_step(
        root,
        &empty_executor,
        &empty_guard_runner,
        &StepConfig::default(),
    )
    .expect_err("tree complete should error");
    assert!(err.to_string().contains("tree already complete"));
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

    let history_md = fs::read_to_string(root.join(".runner/context/history.md"))
        .expect("read .runner/context/history.md");
    assert!(
        history_md.contains("None."),
        "history should be empty when last_status != Retry"
    );
    assert!(
        !history_md.contains("previous work"),
        "history should not include prior done summary"
    );

    // Verify tree state
    let final_tree = repo.read_tree().expect("final tree");
    assert!(final_tree.children[0].passes, "passed-child still passed");
    assert!(final_tree.children[1].passes, "open-child now passed");
    assert!(final_tree.passes, "root auto-passes");

    let iter_dir = root
        .join(".runner/iterations")
        .join(&start.run_id)
        .join("3");
    assert!(iter_dir.join("meta.json").exists());
    assert!(iter_dir.join("output.json").exists());
    assert!(iter_dir.join("guard.log").exists());

    // Verify run_state updated
    let final_state = repo.read_run_state().expect("final state");
    assert_eq!(final_state.next_iter, 4);
    assert_eq!(final_state.last_status, Some(AgentStatus::Done));
    assert_eq!(
        final_state.last_summary,
        Some("open-child complete".to_string())
    );
    assert_eq!(final_state.last_guard, Some(GuardOutcome::Pass));

    executor.assert_drained().expect("executor drained");
    guard_runner.assert_drained().expect("guard drained");

    // Iter 4: tree complete → `run_step` errors.
    let empty_executor = ScriptedExecutor::new(Vec::new());
    let empty_guard_runner = ScriptedGuardRunner::new(Vec::new());
    let err = run_step(
        root,
        &empty_executor,
        &empty_guard_runner,
        &StepConfig::default(),
    )
    .expect_err("tree complete should error");
    assert!(err.to_string().contains("tree already complete"));
}

fn must_find<'a>(node: &'a Node, id: &str) -> &'a Node {
    find_node(node, id).unwrap_or_else(|| panic!("missing node id={id}"))
}

fn find_node<'a>(node: &'a Node, id: &str) -> Option<&'a Node> {
    if node.id == id {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_node(child, id) {
            return Some(found);
        }
    }
    None
}
