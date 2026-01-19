//! Deterministic goal-driven agent loop runner.
//!
//! This crate implements a task-tree execution model where an agent iteratively
//! works through goals until the tree is complete or stuck. The architecture
//! enforces a strict separation:
//!
//! - **[`core`]**: Pure, deterministic logic (selection, validation, state updates).
//!   No I/O, fully testable in isolation.
//! - **[`io`]**: Side-effecting operations (filesystem, git, process execution).
//!   Isolated to enable mocking in tests.
//!
//! Orchestration modules ([`step`], [`start`], [`select`], [`validate`]) coordinate
//! core logic with I/O to implement CLI commands.
//!
//! # Execution Flow (Pseudo Code)
//!
//! ```text
//! run_step():
//!   // Pre-flight
//!   enforce_git_policy()           // not on main/master, worktree clean
//!   enforce_run_id_matches_goal()  // GOAL.md run_id == run_state.run_id
//!   enforce_on_run_branch()        // must be on runner/<run_id>
//!
//!   prev_tree = load_tree()
//!   selected = leftmost_open_leaf(prev_tree)
//!   if is_stuck(selected): return StuckLeafError
//!
//!   write_context(goal, history, failure)
//!
//!   match selected.next:
//!     Decompose:
//!       children = run_decomposer_agent()
//!       next_tree = add_children(prev_tree, selected, children)
//!       validate_contract(prev_tree, next_tree)
//!       apply_state_updates(Decomposed)
//!       write_tree(next_tree)
//!
//!     Execute:
//!       output = run_executor_agent()
//!       next_tree = load_tree()  // agent may have modified
//!       validate_contract(prev_tree, next_tree)  // retry on violation
//!       guard = run_guards_if_done(output.status)
//!       apply_state_updates(output.status, guard)
//!       write_tree(next_tree)
//!
//!   // State transitions (runner-owned, agent cannot modify):
//!   //   Done + Pass  → passes=true
//!   //   Done + Fail  → attempts++
//!   //   Retry        → attempts++
//!   //   Decomposed   → unchanged (children added)
//!
//!   // Persist & commit
//!   write_run_state(iter++, status, guard)
//!   write_iteration_log(meta, output, tree_before, tree_after)
//!   git_commit()
//! ```

pub mod agents;
pub mod core;
pub mod exit_codes;
pub mod io;
pub mod logging;
pub mod looping;
pub mod select;
pub mod start;
pub mod step;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod tree;
pub mod validate;
