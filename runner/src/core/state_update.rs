//! Runner-owned state transitions for task trees.

#![allow(dead_code)]

use crate::core::types::{AgentStatus, GuardOutcome, StateUpdateSummary};
use crate::tree::Node;
use std::collections::HashMap;

/// Apply runner-owned state updates to `next` using `prev` as the source of truth.
///
/// This enforces runner ownership of `passes` and `attempts`, applies the
/// selected-node transition, and derives internal passes from children.
pub fn apply_state_updates(
    prev: &Node,
    next: &mut Node,
    selected_id: &str,
    status: AgentStatus,
    guard: GuardOutcome,
) -> Result<StateUpdateSummary, String> {
    let prev_state = index_runner_owned(prev);
    reset_runner_owned_fields(next, &prev_state);

    let selected = find_node_mut(next, selected_id)
        .ok_or_else(|| format!("selected node '{}' not found", selected_id))?;

    let mut summary = StateUpdateSummary {
        status,
        guard_outcome: guard,
        passes_set: Vec::new(),
        attempts_incremented: Vec::new(),
        derived_passes_set: Vec::new(),
    };

    match status {
        AgentStatus::Done => match guard {
            GuardOutcome::Pass => {
                if !selected.passes {
                    selected.passes = true;
                    summary.passes_set.push(selected.id.clone());
                }
            }
            GuardOutcome::Fail => {
                let before = selected.attempts;
                if before < selected.max_attempts {
                    selected.attempts = before + 1;
                    summary.attempts_incremented.push(selected.id.clone());
                }
            }
            GuardOutcome::Skipped => {}
        },
        AgentStatus::Retry => {
            let before = selected.attempts;
            if before < selected.max_attempts {
                selected.attempts = before + 1;
                summary.attempts_incremented.push(selected.id.clone());
            }
        }
        AgentStatus::Decomposed => {}
    }

    derive_internal_passes(next, &mut summary);
    next.sort_children();

    summary.passes_set.sort();
    summary.attempts_incremented.sort();
    summary.derived_passes_set.sort();

    Ok(summary)
}

fn index_runner_owned(node: &Node) -> HashMap<String, (bool, u32)> {
    let mut map = HashMap::new();
    index_runner_owned_inner(node, &mut map);
    map
}

fn index_runner_owned_inner(node: &Node, map: &mut HashMap<String, (bool, u32)>) {
    map.insert(node.id.clone(), (node.passes, node.attempts));
    for child in &node.children {
        index_runner_owned_inner(child, map);
    }
}

fn reset_runner_owned_fields(node: &mut Node, prev_state: &HashMap<String, (bool, u32)>) {
    if let Some((passes, attempts)) = prev_state.get(&node.id) {
        node.passes = *passes;
        node.attempts = *attempts;
    } else {
        node.passes = false;
        node.attempts = 0;
    }

    for child in &mut node.children {
        reset_runner_owned_fields(child, prev_state);
    }
}

fn derive_internal_passes(node: &mut Node, summary: &mut StateUpdateSummary) -> bool {
    if node.children.is_empty() {
        return node.passes;
    }

    let mut all_children_passed = true;
    for child in &mut node.children {
        if !derive_internal_passes(child, summary) {
            all_children_passed = false;
        }
    }

    if node.passes != all_children_passed {
        node.passes = all_children_passed;
        if node.passes {
            summary.derived_passes_set.push(node.id.clone());
        }
    }

    node.passes
}

fn find_node_mut<'a>(node: &'a mut Node, target_id: &str) -> Option<&'a mut Node> {
    if node.id == target_id {
        return Some(node);
    }

    for child in &mut node.children {
        if let Some(found) = find_node_mut(child, target_id) {
            return Some(found);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{leaf, node, node_with_children};

    /// Runner-owned fields (passes, attempts) are reset even if agent sets them.
    #[test]
    fn apply_state_updates_overwrites_runner_owned_fields() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        let mut next = node_with_children("root", 0, vec![leaf("a", 0, true)]);
        next.children[0].attempts = 5;

        let summary = apply_state_updates(
            &prev,
            &mut next,
            "a",
            AgentStatus::Decomposed,
            GuardOutcome::Skipped,
        )
        .expect("state update");

        assert!(!next.children[0].passes);
        assert_eq!(next.children[0].attempts, 0);
        assert!(summary.passes_set.is_empty());
        assert!(summary.attempts_incremented.is_empty());
    }

    /// Done + Pass sets passes=true and derives parent passes when all children pass.
    #[test]
    fn apply_state_updates_sets_pass_and_derives_internal_passes() {
        let mut prev = node_with_children("root", 0, vec![leaf("a", 0, false), leaf("b", 1, true)]);
        prev.passes = false;
        let mut next = prev.clone();

        let summary =
            apply_state_updates(&prev, &mut next, "a", AgentStatus::Done, GuardOutcome::Pass)
                .expect("state update");

        assert!(next.children[0].passes);
        assert!(next.passes);
        assert_eq!(summary.passes_set, vec!["a".to_string()]);
        assert_eq!(summary.derived_passes_set, vec!["root".to_string()]);
    }

    /// Done + Fail increments attempts (agent claimed done but guards failed).
    #[test]
    fn apply_state_updates_increments_attempts_on_guard_fail() {
        let mut prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        prev.children[0].attempts = 1;
        prev.children[0].max_attempts = 2;
        let mut next = prev.clone();

        let summary =
            apply_state_updates(&prev, &mut next, "a", AgentStatus::Done, GuardOutcome::Fail)
                .expect("state update");

        assert_eq!(next.children[0].attempts, 2);
        assert_eq!(summary.attempts_incremented, vec!["a".to_string()]);
    }

    /// Attempts don't exceed max_attempts (saturation behavior).
    #[test]
    fn apply_state_updates_saturates_attempts_at_max() {
        let mut prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        prev.children[0].attempts = 2;
        prev.children[0].max_attempts = 2;
        let mut next = prev.clone();

        let summary =
            apply_state_updates(&prev, &mut next, "a", AgentStatus::Done, GuardOutcome::Fail)
                .expect("state update");

        assert_eq!(next.children[0].attempts, 2);
        assert!(summary.attempts_incremented.is_empty());
    }

    /// Missing selected_id returns an error (indicates bug in selection).
    #[test]
    fn apply_state_updates_errors_on_missing_selected_id() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        let mut next = prev.clone();
        let err = apply_state_updates(
            &prev,
            &mut next,
            "missing",
            AgentStatus::Decomposed,
            GuardOutcome::Skipped,
        )
        .expect_err("expected error");
        assert!(err.contains("missing"));
    }

    /// New nodes added by agent have runner-owned fields reset to defaults.
    #[test]
    fn apply_state_updates_resets_new_nodes() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        let mut new_node = node("b", 1);
        new_node.passes = true;
        new_node.attempts = 9;
        let mut next = node_with_children("root", 0, vec![leaf("a", 0, false), new_node]);

        let summary = apply_state_updates(
            &prev,
            &mut next,
            "a",
            AgentStatus::Decomposed,
            GuardOutcome::Skipped,
        )
        .expect("state update");

        assert!(!next.children[1].passes);
        assert_eq!(next.children[1].attempts, 0);
        assert!(summary.passes_set.is_empty());
    }

    /// Decomposed status never marks passes=true (work is split, not complete).
    #[test]
    fn apply_state_updates_decompose_does_not_mark_pass() {
        let prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        let mut next = node_with_children("root", 0, vec![leaf("a", 0, true)]);

        let summary = apply_state_updates(
            &prev,
            &mut next,
            "a",
            AgentStatus::Decomposed,
            GuardOutcome::Skipped,
        )
        .expect("state update");

        assert!(!next.children[0].passes);
        assert!(summary.passes_set.is_empty());
    }

    /// Happy path: Done + Pass propagates passes up through completed subtrees.
    #[test]
    fn apply_state_updates_happy_path_transition() {
        let prev = node_with_children(
            "root",
            0,
            vec![
                node_with_children("group", 0, vec![leaf("a", 0, false), leaf("b", 1, true)]),
                leaf("c", 1, false),
            ],
        );
        let mut next = prev.clone();
        next.children.push(node("new", 2));

        let summary =
            apply_state_updates(&prev, &mut next, "a", AgentStatus::Done, GuardOutcome::Pass)
                .expect("state update");

        let group = &next.children[0];
        assert!(group.children[0].passes);
        assert!(group.passes);
        assert!(!next.passes);
        assert!(summary.passes_set.contains(&"a".to_string()));
        assert!(summary.derived_passes_set.contains(&"group".to_string()));
    }

    /// Retry status increments attempts (agent wants another try).
    #[test]
    fn apply_state_updates_increments_attempts_on_retry() {
        let mut prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        prev.children[0].attempts = 1;
        prev.children[0].max_attempts = 3;
        let mut next = prev.clone();

        let summary = apply_state_updates(
            &prev,
            &mut next,
            "a",
            AgentStatus::Retry,
            GuardOutcome::Skipped,
        )
        .expect("state update");

        assert_eq!(next.children[0].attempts, 2);
        assert_eq!(summary.attempts_incremented, vec!["a".to_string()]);
    }

    /// Retry also saturates at max_attempts.
    #[test]
    fn apply_state_updates_retry_saturates_attempts_at_max() {
        let mut prev = node_with_children("root", 0, vec![leaf("a", 0, false)]);
        prev.children[0].attempts = 2;
        prev.children[0].max_attempts = 2;
        let mut next = prev.clone();

        let summary = apply_state_updates(
            &prev,
            &mut next,
            "a",
            AgentStatus::Retry,
            GuardOutcome::Skipped,
        )
        .expect("state update");

        assert_eq!(next.children[0].attempts, 2);
        assert!(summary.attempts_incremented.is_empty());
    }
}
