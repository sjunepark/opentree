//! Investigation tests for Codex CLI `--output-schema` and `--output-last-message` flags.
//!
//! These tests validate that Codex CLI correctly enforces structured output via JSON Schema.
//! They require the Codex CLI to be installed and configured with valid API credentials.
//!
//! # Prerequisites
//!
//! - Codex CLI installed (`npm install -g @anthropic/codex` or similar)
//! - Valid API credentials configured (e.g., `ANTHROPIC_API_KEY` environment variable)
//!
//! # Running
//!
//! ```bash
//! # Run all investigation tests
//! cargo test --test investigation -- --ignored
//!
//! # Run specific test
//! cargo test --test investigation codex_cli_available -- --ignored
//! ```

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use tempfile::tempdir;
use wait_timeout::ChildExt;

/// Default timeout for Codex CLI calls (60 seconds to accommodate LLM latency).
const CODEX_TIMEOUT: Duration = Duration::from_secs(60);

/// Path to the schema file relative to the runner crate root.
const SCHEMA_PATH: &str = "schemas/agent_output.schema.json";

/// Verifies that the Codex CLI is available in PATH.
#[test]
#[ignore]
fn codex_cli_available() {
    let output = Command::new("codex")
        .arg("--version")
        .output()
        .expect("codex not in PATH - install with: npm install -g @anthropic/codex");

    assert!(
        output.status.success(),
        "codex --version failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let version = String::from_utf8_lossy(&output.stdout);
    println!("Codex CLI version: {}", version.trim());
}

/// Verifies that `--output-schema` produces valid JSON matching the schema.
///
/// Runs Codex with a simple prompt and validates the output conforms to the
/// agent_output.schema.json schema (status enum + summary string).
#[test]
#[ignore]
fn output_schema_produces_valid_json() {
    let tmp = tempdir().expect("create tempdir");
    let schema_dest = tmp.path().join("schema.json");

    // Copy schema to temp directory
    copy_schema(&schema_dest);

    let output_path = tmp.path().join("output.json");

    let mut child = Command::new("codex")
        .args([
            "exec",
            "--output-schema",
            schema_dest.to_str().unwrap(),
            "--output-last-message",
            output_path.to_str().unwrap(),
            "--",
            "Output JSON with status='done' and summary='test completed'",
        ])
        .spawn()
        .expect("spawn codex");

    let status = child
        .wait_timeout(CODEX_TIMEOUT)
        .expect("wait")
        .expect("codex timed out");

    assert!(status.success(), "codex exec failed");
    assert!(output_path.exists(), "output file not created");

    let content = fs::read_to_string(&output_path).expect("read output");
    let json: serde_json::Value = serde_json::from_str(&content).expect("parse JSON");

    // Validate structure
    assert!(json.is_object(), "output should be an object");
    assert!(json.get("status").is_some(), "missing 'status' field");
    assert!(json.get("summary").is_some(), "missing 'summary' field");

    // Validate status is valid enum value
    let status_val = json["status"].as_str().expect("status should be string");
    assert!(
        ["done", "retry", "decomposed"].contains(&status_val),
        "status '{}' not in enum",
        status_val
    );

    // Validate summary is string
    assert!(json["summary"].is_string(), "summary should be string");
}

/// Verifies that `--output-last-message` writes output to the specified path.
#[test]
#[ignore]
fn output_last_message_writes_to_path() {
    let tmp = tempdir().expect("create tempdir");
    let schema_dest = tmp.path().join("schema.json");
    copy_schema(&schema_dest);

    // Use a nested path to verify parent directories are created
    let output_path = tmp.path().join("nested/dir/output.json");

    let mut child = Command::new("codex")
        .args([
            "exec",
            "--output-schema",
            schema_dest.to_str().unwrap(),
            "--output-last-message",
            output_path.to_str().unwrap(),
            "--",
            "Output JSON with status='done' and summary='path test'",
        ])
        .spawn()
        .expect("spawn codex");

    let status = child
        .wait_timeout(CODEX_TIMEOUT)
        .expect("wait")
        .expect("codex timed out");

    assert!(status.success(), "codex exec failed");
    assert!(
        output_path.exists(),
        "output file not created at nested path"
    );

    let content = fs::read_to_string(&output_path).expect("read output");
    assert!(!content.is_empty(), "output file is empty");

    // Verify it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&content).expect("output should be valid JSON");
}

/// Verifies that the schema's enum constraint is enforced.
///
/// Requests a specific enum value and confirms the output contains one of the
/// valid enum values (done, retry, decomposed).
#[test]
#[ignore]
fn schema_with_enum_constrains_values() {
    let tmp = tempdir().expect("create tempdir");
    let schema_dest = tmp.path().join("schema.json");
    copy_schema(&schema_dest);

    let output_path = tmp.path().join("output.json");

    // Request 'retry' status specifically
    let mut child = Command::new("codex")
        .args([
            "exec",
            "--output-schema",
            schema_dest.to_str().unwrap(),
            "--output-last-message",
            output_path.to_str().unwrap(),
            "--",
            "Output JSON with status='retry' and summary='need more work'",
        ])
        .spawn()
        .expect("spawn codex");

    let status = child
        .wait_timeout(CODEX_TIMEOUT)
        .expect("wait")
        .expect("codex timed out");

    assert!(status.success(), "codex exec failed");

    let content = fs::read_to_string(&output_path).expect("read output");
    let json: serde_json::Value = serde_json::from_str(&content).expect("parse JSON");

    let status_val = json["status"].as_str().expect("status should be string");

    // The schema should constrain status to valid enum values
    // (LLM should respect the constraint, but we verify the output is valid)
    assert!(
        ["done", "retry", "decomposed"].contains(&status_val),
        "status '{}' is not a valid enum value; schema constraint may not be enforced",
        status_val
    );
}

/// Copies the schema file from the runner crate to the destination path.
fn copy_schema(dest: &Path) {
    // Locate schema relative to CARGO_MANIFEST_DIR
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let schema_src = Path::new(manifest_dir).join(SCHEMA_PATH);

    assert!(
        schema_src.exists(),
        "schema not found at: {}",
        schema_src.display()
    );

    // Create parent directories if needed
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }

    fs::copy(&schema_src, dest).expect("copy schema");
}
