//! Combination runner for prompt × input testing.
//!
//! Discovers prompts and inputs, executes all combinations, and caches results.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use runner::core::types::DecompositionOutput;
use runner::io::executor::{CodexExecutor, ExecRequest, execute_and_load_json};
use runner::tree::NodeNext;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::cache::{ResultCache, content_hash};
use crate::render::render_prompt;

/// A selected node specification from test input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedNode {
    pub id: String,
    pub title: String,
    pub goal: String,
    #[serde(default)]
    pub acceptance: Vec<String>,
    #[serde(default = "default_next")]
    pub next: NodeNext,
}

fn default_next() -> NodeNext {
    NodeNext::Decompose
}

/// Test input for a prompt combination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInput {
    pub id: String,
    pub name: String,
    pub selected_node: SelectedNode,
    #[serde(default)]
    pub tree_summary: Option<String>,
    #[serde(default)]
    pub context_goal: String,
    #[serde(default)]
    pub context_history: Option<String>,
    #[serde(default)]
    pub context_failure: Option<String>,
    #[serde(default)]
    pub assumptions: Option<String>,
    #[serde(default)]
    pub questions: Option<String>,
    /// Expected next classification inferred from children (execute or decompose).
    #[serde(default)]
    pub expected_decision: Option<String>,
}

/// Result of running a single combination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinationResult {
    pub prompt_name: String,
    pub prompt_hash: String,
    pub input_id: String,
    pub input_name: String,
    pub expected_decision: Option<String>,
    pub actual_decision: Option<String>,
    pub matches_expected: Option<bool>,
    pub output: Option<DecompositionOutput>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub timestamp: String,
}

/// Discovered prompt template.
struct Prompt {
    name: String,
    path: PathBuf,
    #[allow(dead_code)]
    content: String,
    hash: String,
}

/// Run all combinations for an agent.
pub fn run_agent(lab_root: &Path, agent: &str, force: bool) -> Result<()> {
    let prompts = discover_prompts(lab_root, agent)?;
    let inputs = discover_inputs(lab_root, agent)?;

    if prompts.is_empty() {
        anyhow::bail!("No prompts found in prompts/{}/", agent);
    }
    if inputs.is_empty() {
        anyhow::bail!("No inputs found in inputs/{}/", agent);
    }

    info!(
        prompts = prompts.len(),
        inputs = inputs.len(),
        total = prompts.len() * inputs.len(),
        "discovered combinations"
    );

    let cache = ResultCache::new(lab_root);
    let executor = CodexExecutor;

    for prompt in &prompts {
        for input in &inputs {
            let cached = !force && cache.has_cached(agent, &prompt.hash, &input.id);
            if cached {
                debug!(
                    prompt = %prompt.name,
                    input = %input.id,
                    "skipping cached combination"
                );
                continue;
            }

            info!(prompt = %prompt.name, input = %input.id, "running combination");
            let result = run_combination(&executor, lab_root, prompt, input)?;

            let status = if result.error.is_some() {
                "error"
            } else if result.matches_expected == Some(true) {
                "pass"
            } else if result.matches_expected == Some(false) {
                "fail"
            } else {
                "ok"
            };
            info!(
                prompt = %prompt.name,
                input = %input.id,
                status = status,
                decision = ?result.actual_decision,
                "combination completed"
            );

            cache.save_result(agent, &prompt.hash, &input.id, &result)?;
        }
    }

    // Generate index.json for dashboard
    cache.generate_index(agent)?;

    println!(
        "\nResults saved to: {}/results/{}/",
        lab_root.display(),
        agent
    );
    Ok(())
}

/// Discover prompts for an agent.
fn discover_prompts(lab_root: &Path, agent: &str) -> Result<Vec<Prompt>> {
    let prompts_dir = lab_root.join("prompts").join(agent);
    if !prompts_dir.exists() {
        return Ok(vec![]);
    }

    let mut prompts = Vec::new();
    for entry in fs::read_dir(&prompts_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "md") {
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let content = fs::read_to_string(&path)
                .with_context(|| format!("read prompt {}", path.display()))?;
            let hash = content_hash(&content);
            prompts.push(Prompt {
                name,
                path,
                content,
                hash,
            });
        }
    }

    prompts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(prompts)
}

/// Discover test inputs for an agent.
fn discover_inputs(lab_root: &Path, agent: &str) -> Result<Vec<TestInput>> {
    let inputs_dir = lab_root.join("inputs").join(agent);
    if !inputs_dir.exists() {
        return Ok(vec![]);
    }

    let mut inputs = Vec::new();
    for entry in fs::read_dir(&inputs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("read input {}", path.display()))?;
            let input: TestInput = serde_json::from_str(&content)
                .with_context(|| format!("parse input {}", path.display()))?;
            inputs.push(input);
        }
    }

    inputs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(inputs)
}

/// Run a single prompt × input combination.
fn run_combination(
    executor: &CodexExecutor,
    lab_root: &Path,
    prompt: &Prompt,
    input: &TestInput,
) -> Result<CombinationResult> {
    let start = Instant::now();
    let timestamp = chrono_lite_timestamp();

    // Render the prompt with test input
    let rendered = match render_prompt(&prompt.path, input) {
        Ok(r) => r,
        Err(e) => {
            return Ok(CombinationResult {
                prompt_name: prompt.name.clone(),
                prompt_hash: prompt.hash.clone(),
                input_id: input.id.clone(),
                input_name: input.name.clone(),
                expected_decision: input.expected_decision.clone(),
                actual_decision: None,
                matches_expected: None,
                output: None,
                error: Some(format!("render error: {}", e)),
                duration_ms: start.elapsed().as_millis() as u64,
                timestamp,
            });
        }
    };

    // Set up execution paths
    let temp_dir = lab_root.join("tmp");
    fs::create_dir_all(&temp_dir)?;

    let run_id = format!("{}_{}", prompt.hash, input.id);
    let output_path = temp_dir.join(format!("{}_output.json", run_id));
    let log_path = temp_dir.join(format!("{}_log.txt", run_id));

    // Find the schema path
    let schema_path = find_schema_path(lab_root, "decomposer_output")?;

    let request = ExecRequest {
        workdir: lab_root.to_path_buf(),
        prompt: rendered,
        output_schema_path: schema_path,
        output_path: output_path.clone(),
        executor_log_path: log_path,
        timeout: Duration::from_secs(120),
        output_limit_bytes: 100_000,
        stream_path: None,
    };

    // Execute
    let output_result: Result<DecompositionOutput> = execute_and_load_json(executor, &request);

    let (output, error) = match output_result {
        Ok(decision) => (Some(decision), None),
        Err(e) => {
            warn!(error = %e, "execution failed");
            (None, Some(format!("{:#}", e)))
        }
    };

    let actual_decision = output.as_ref().map(|o| {
        if o.children.iter().any(|c| c.next == NodeNext::Decompose) {
            "decompose".to_string()
        } else {
            "execute".to_string()
        }
    });
    let matches_expected = match (&input.expected_decision, &actual_decision) {
        (Some(expected), Some(actual)) => Some(expected.to_lowercase() == *actual),
        _ => None,
    };

    Ok(CombinationResult {
        prompt_name: prompt.name.clone(),
        prompt_hash: prompt.hash.clone(),
        input_id: input.id.clone(),
        input_name: input.name.clone(),
        expected_decision: input.expected_decision.clone(),
        actual_decision,
        matches_expected,
        output,
        error,
        duration_ms: start.elapsed().as_millis() as u64,
        timestamp,
    })
}

/// Find the JSON schema for decomposer outputs.
fn find_schema_path(lab_root: &Path, schema_name: &str) -> Result<PathBuf> {
    // Look in common locations
    let candidates = [
        lab_root
            .join("schemas")
            .join(format!("{}.json", schema_name)),
        lab_root
            .parent()
            .map(|p| p.join("schemas").join(format!("{}.json", schema_name)))
            .unwrap_or_default(),
        lab_root
            .parent()
            .map(|p| {
                p.join(".runner")
                    .join("schemas")
                    .join(format!("{}.json", schema_name))
            })
            .unwrap_or_default(),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // Create a minimal schema if none found
    let schema_dir = lab_root.join("schemas");
    fs::create_dir_all(&schema_dir)?;
    let schema_path = schema_dir.join(format!("{}.json", schema_name));

    let schema = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "additionalProperties": false,
        "required": ["decision", "summary", "children"],
        "properties": {
            "decision": {
                "type": "string",
                "enum": ["execute", "decompose"]
            },
            "summary": {
                "type": "string"
            },
            "children": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["title", "goal", "acceptance"],
                    "properties": {
                        "title": { "type": "string" },
                        "goal": { "type": "string" },
                        "acceptance": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    }
                }
            }
        }
    });

    fs::write(&schema_path, serde_json::to_string_pretty(&schema)?)?;
    info!(path = %schema_path.display(), "created default schema");
    Ok(schema_path)
}

/// Generate ISO 8601 timestamp without external dependencies.
fn chrono_lite_timestamp() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    format!("{}", secs) // Unix timestamp for simplicity
}
