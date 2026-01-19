//! Template rendering for prompt variants.
//!
//! Loads templates from the prompts/ directory at runtime, enabling rapid iteration
//! without recompilation.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use minijinja::{Environment, context};
use serde::{Deserialize, Serialize};

use crate::runner::TestInput;

/// Minimal node context for template rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedNodeContext {
    pub path: String,
    pub id: String,
    pub title: String,
    pub goal: String,
    pub acceptance: Vec<String>,
}

impl SelectedNodeContext {
    pub fn from_test_input(input: &TestInput) -> Self {
        Self {
            path: format!("root/{}", input.selected_node.title),
            id: input.selected_node.id.clone(),
            title: input.selected_node.title.clone(),
            goal: input.selected_node.goal.clone(),
            acceptance: input.selected_node.acceptance.clone(),
        }
    }
}

/// Render a prompt template with the given test input.
pub fn render_prompt(template_path: &Path, input: &TestInput) -> Result<String> {
    let template_content = fs::read_to_string(template_path)
        .with_context(|| format!("read template {}", template_path.display()))?;

    let mut env = Environment::new();
    env.add_template("prompt", &template_content)
        .context("parse template")?;

    let template = env.get_template("prompt")?;
    let selected = SelectedNodeContext::from_test_input(input);

    let rendered = template.render(context! {
        goal => &input.context_goal,
        history => input.context_history.as_deref(),
        failure => input.context_failure.as_deref(),
        selected => selected,
        tree_summary => input.tree_summary.as_deref(),
        assumptions => input.assumptions.as_deref(),
        questions => input.questions.as_deref(),
    })?;

    // Strip section markers (they're for budget management, not final output)
    let cleaned = strip_section_markers(&rendered);
    Ok(cleaned)
}

/// Remove HTML comment section markers from rendered output.
fn strip_section_markers(content: &str) -> String {
    use std::sync::LazyLock;
    static SECTION_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"<!--\s*section:\w+\s+(?:required|droppable)\s*-->").unwrap()
    });

    SECTION_RE.replace_all(content, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::SelectedNode;

    #[test]
    fn test_strip_section_markers() {
        let input = "<!-- section:contract required -->\nContent\n<!-- section:goal droppable -->\nMore";
        let output = strip_section_markers(input);
        assert_eq!(output, "\nContent\n\nMore");
    }

    #[test]
    fn test_render_minimal_template() {
        let temp_dir = tempfile::tempdir().unwrap();
        let template_path = temp_dir.path().join("test.md");
        fs::write(&template_path, "Goal: {{ goal }}\nTitle: {{ selected.title }}").unwrap();

        let input = TestInput {
            id: "test".to_string(),
            name: "Test".to_string(),
            selected_node: SelectedNode {
                id: "n1".to_string(),
                title: "Test Node".to_string(),
                goal: "Do something".to_string(),
                acceptance: vec![],
            },
            tree_summary: None,
            context_goal: "Build feature".to_string(),
            context_history: None,
            context_failure: None,
            assumptions: None,
            questions: None,
            expected_decision: None,
        };

        let result = render_prompt(&template_path, &input).unwrap();
        assert!(result.contains("Goal: Build feature"));
        assert!(result.contains("Title: Test Node"));
    }
}
