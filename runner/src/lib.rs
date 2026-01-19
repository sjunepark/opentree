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
//! run_loop():
//!   while true:
//!     outcome = run_step()
//!     if tree_complete() or stuck or max_iterations: break
//!
//! run_step():
//!   // Pre-flight
//!   enforce_git_policy()          // not on main/master, worktree clean
//!   enforce_run_id_matches_goal() // GOAL.md run_id == run_state.run_id
//!   enforce_on_run_branch()       // must be on runner/<run_id>
//!
//!   // Selection
//!   prev_tree = load_tree()
//!   selected = leftmost_open_leaf(prev_tree)  // depth-first, deterministic
//!   if is_stuck(selected): return StuckLeafError
//!
//!   // Context
//!   write_context(goal, history, failure)
//!   prompt_inputs = build_prompt_inputs(selected, tree_summary)
//!
//!   // Phase 1: Tree Agent (decide decompose or execute)
//!   tree_decision = execute(tree_agent_prompt)
//!   match tree_decision:
//!     Decompose:
//!       if children.is_empty(): return Retry
//!       add_children(selected, tree_decision.children)
//!       apply_state_updates(Decomposed)
//!       return  // skip executor
//!
//!     Execute:
//!       // Phase 2: Executor Agent (perform work)
//!       output = execute(executor_prompt + planner_summary)
//!       next_tree = load_tree()  // agent may have modified it
//!
//!       // Validate agent contract
//!       if immutability_violated(prev_tree, next_tree): return Retry
//!       if child_additions_restricted(prev_tree, next_tree): return Retry
//!
//!       // Guards (only if agent claims Done)
//!       guard_outcome = if output.status == Done:
//!         run_guards()  // Pass | Fail
//!       else:
//!         Skipped
//!
//!       // State transitions (runner-owned, not agent-modifiable)
//!       apply_state_updates(prev_tree, next_tree, output.status, guard_outcome)
//!       //   Done + Pass  → passes=true
//!       //   Done + Fail  → attempts++
//!       //   Retry        → attempts++
//!       //   Decomposed   → no change (children added)
//!
//!   // Persist
//!   write_tree(next_tree)
//!   write_run_state(iter++, last_status, last_guard)
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
