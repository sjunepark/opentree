//! Investigation tests for `TreeAgent` behavior when backed by a real LLM via Codex CLI.
//!
//! These tests validate the *intended* heuristic behavior:
//! - Complicated goals should be decomposed into child specs.
//! - Simple goals should be executed directly.
//!
//! They are ignored by default because they require:
//! - Codex CLI installed and configured with API credentials
//! - Network access
//! - Non-deterministic model behavior
//!
//! Run with:
//!
//! ```bash
//! cargo test -p runner --test investigation_llm tree_agent_ -- --ignored
//! # or
//! just investigate-llm tree_agent_
//!
//! # With logging (use --nocapture to see output for passing tests):
//! TEST_LOG=1 cargo test -p runner --test investigation_llm tree_agent_ -- --ignored --nocapture
//! TEST_LOG=1 RUST_LOG=debug just investigate-llm tree_agent_
//! ```

use std::sync::Once;
use std::time::{Duration, Instant};

use runner::agents::tree::TreeAgent;
use runner::core::types::TreeDecisionKind;
use runner::io::executor::CodexExecutor;
use runner::io::prompt::PromptInputs;
use runner::tree::Node;
use tempfile::tempdir;
use tracing::{debug, info};

/// Default timeout for LLM calls.
const CODEX_TIMEOUT: Duration = Duration::from_secs(120);

static INIT_LOGGING: Once = Once::new();

fn init_test_logging() {
    INIT_LOGGING.call_once(|| {
        if std::env::var("TEST_LOG").is_ok() {
            tracing_subscriber::fmt()
                .with_test_writer()
                .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".to_string()))
                .init();
        }
    });
}

const PROMPT_BUDGET_BYTES: usize = 40_000;
const OUTPUT_LIMIT_BYTES: usize = 100_000;

fn prompt_inputs_for(title: &str, goal: &str, acceptance: &[&str]) -> PromptInputs {
    let mut acceptance_lines = String::new();
    if !acceptance.is_empty() {
        acceptance_lines.push_str("\nacceptance:\n");
        for item in acceptance {
            acceptance_lines.push_str(&format!("- {item}\n"));
        }
    }

    let context_goal = format!("title: {title}\ngoal: {goal}{acceptance_lines}");

    PromptInputs {
        selected_path: "root".to_string(),
        selected_node: Node {
            id: "root".to_string(),
            order: 0,
            title: title.to_string(),
            goal: goal.to_string(),
            acceptance: acceptance.iter().map(|s| s.to_string()).collect(),
            passes: false,
            attempts: 0,
            max_attempts: 3,
            children: Vec::new(),
        },
        tree_summary: "- root (passes=false, attempts=0/3)".to_string(),
        context_goal,
        context_history: None,
        context_failure: None,
        assumptions: String::new(),
        questions: String::new(),
    }
}

#[test]
#[ignore]
fn tree_agent_decomposes_complicated_goal() {
    init_test_logging();
    info!("starting: tree_agent_decomposes_complicated_goal");

    let tmp = tempdir().expect("create tempdir");
    let root = tmp.path();
    debug!(workdir = %root.display(), "created tempdir");

    let state_dir = root.join(".runner/state");
    let iter_dir = root.join(".runner/iterations/run-test/1");

    let agent = TreeAgent::new(&state_dir, PROMPT_BUDGET_BYTES, OUTPUT_LIMIT_BYTES);
    let executor = CodexExecutor;

    let inputs = prompt_inputs_for(
        "Implement end-to-end OAuth2 login + RBAC + audit logging",
        "Add OAuth2 (PKCE), session/refresh tokens, role-based access control, audit logs, and full test coverage. Ensure deterministic behavior in the runner, update docs, and add lifecycle tests.",
        &[
            "OAuth2 login flow implemented (PKCE)",
            "Refresh tokens with rotation + revocation",
            "RBAC enforcement in request middleware",
            "Audit logging for auth events",
            "Docs updated (how to configure, how to run tests)",
            "Unit + integration tests added",
        ],
    );
    debug!(title = %inputs.selected_node.title, "running tree agent");

    let decision = agent
        .run(
            &executor,
            root,
            &iter_dir,
            &inputs,
            Instant::now() + CODEX_TIMEOUT,
        )
        .expect("tree agent run");

    info!(
        decision = ?decision.decision,
        children = decision.children.len(),
        "tree agent returned"
    );

    assert_eq!(
        decision.decision,
        TreeDecisionKind::Decompose,
        "tree agent returned: {decision:?}"
    );
    assert!(
        !decision.children.is_empty(),
        "expected 1+ child specs for decision=decompose"
    );

    for (i, child) in decision.children.iter().enumerate() {
        debug!(index = i, title = %child.title, "child spec");
        assert!(
            !child.title.trim().is_empty(),
            "child title must be non-empty"
        );
        assert!(
            !child.goal.trim().is_empty(),
            "child goal must be non-empty"
        );
    }

    info!("passed: tree_agent_decomposes_complicated_goal");
}

#[test]
#[ignore]
fn tree_agent_executes_simple_goal() {
    init_test_logging();
    info!("starting: tree_agent_executes_simple_goal");

    let tmp = tempdir().expect("create tempdir");
    let root = tmp.path();
    debug!(workdir = %root.display(), "created tempdir");

    let state_dir = root.join(".runner/state");
    let iter_dir = root.join(".runner/iterations/run-test/1");

    let agent = TreeAgent::new(&state_dir, PROMPT_BUDGET_BYTES, OUTPUT_LIMIT_BYTES);
    let executor = CodexExecutor;

    let inputs = prompt_inputs_for(
        "Fix a single typo in README",
        "Update README.md to fix the spelling of the project name in the first header line.",
        &["README header spelling is corrected"],
    );
    debug!(title = %inputs.selected_node.title, "running tree agent");

    let decision = agent
        .run(
            &executor,
            root,
            &iter_dir,
            &inputs,
            Instant::now() + CODEX_TIMEOUT,
        )
        .expect("tree agent run");

    info!(
        decision = ?decision.decision,
        children = decision.children.len(),
        "tree agent returned"
    );

    assert_eq!(
        decision.decision,
        TreeDecisionKind::Execute,
        "tree agent returned: {decision:?}"
    );
    assert!(
        decision.children.is_empty(),
        "expected no child specs for decision=execute"
    );

    info!("passed: tree_agent_executes_simple_goal");
}
