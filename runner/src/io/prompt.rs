//! Prompt pack builder for deterministic executor input.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use minijinja::{Environment, context};
use serde::Serialize;
use tracing::debug;

use crate::tree::Node;

const DECOMPOSER_TEMPLATE: &str = include_str!("prompts/decomposer.md");
const EXECUTOR_TEMPLATE: &str = include_str!("prompts/executor.md");

/// Selected node context for template rendering.
#[derive(Debug, Clone, Serialize)]
struct SelectedNodeContext {
    path: String,
    id: String,
    title: String,
    goal: String,
    acceptance: Vec<String>,
    next: String,
}

impl SelectedNodeContext {
    fn from_node(path: &str, node: &Node) -> Self {
        Self {
            path: path.to_string(),
            id: node.id.clone(),
            title: node.title.clone(),
            goal: node.goal.clone(),
            acceptance: node.acceptance.clone(),
            next: node.next.as_str().to_string(),
        }
    }
}

/// Template engine wrapper around minijinja.
struct PromptEngine {
    env: Environment<'static>,
}

impl PromptEngine {
    fn new() -> Self {
        let mut env = Environment::new();
        env.add_template("decomposer", DECOMPOSER_TEMPLATE)
            .expect("decomposer template should be valid");
        env.add_template("executor", EXECUTOR_TEMPLATE)
            .expect("executor template should be valid");
        Self { env }
    }

    fn render_decomposer(&self, input: &PromptInputs) -> Result<String> {
        let selected = SelectedNodeContext::from_node(&input.selected_path, &input.selected_node);
        let template = self.env.get_template("decomposer")?;
        let rendered = template.render(context! {
            goal => input.context_goal.trim(),
            history => input.context_history.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            failure => input.context_failure.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            selected => selected,
            tree_summary => (!input.tree_summary.trim().is_empty()).then(|| input.tree_summary.trim()),
            assumptions => (!input.assumptions.trim().is_empty()).then(|| input.assumptions.trim()),
            questions => (!input.questions.trim().is_empty()).then(|| input.questions.trim()),
        })?;
        Ok(rendered)
    }

    fn render_executor(&self, input: &PromptInputs, planner_notes: Option<&str>) -> Result<String> {
        let selected = SelectedNodeContext::from_node(&input.selected_path, &input.selected_node);
        let template = self.env.get_template("executor")?;
        let rendered = template.render(context! {
            planner_notes => planner_notes.map(str::trim).filter(|s| !s.is_empty()),
            goal => input.context_goal.trim(),
            history => input.context_history.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            failure => input.context_failure.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            selected => selected,
            tree_summary => (!input.tree_summary.trim().is_empty()).then(|| input.tree_summary.trim()),
            assumptions => (!input.assumptions.trim().is_empty()).then(|| input.assumptions.trim()),
            questions => (!input.questions.trim().is_empty()).then(|| input.questions.trim()),
        })?;
        Ok(rendered)
    }
}

/// A parsed section from rendered template output.
#[derive(Debug, Clone)]
struct ParsedSection {
    /// Section identifier (e.g., "contract", "goal").
    key: String,
    /// Whether this section is required (cannot be dropped).
    required: bool,
    /// Full section content including header.
    content: String,
}

/// Parse sections from rendered template output using HTML comment markers.
///
/// Markers follow format: `<!-- section:KEY required|droppable -->`
fn parse_sections(rendered: &str) -> Vec<ParsedSection> {
    use std::sync::LazyLock;
    static SECTION_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"<!--\s*section:(\w+)\s+(required|droppable)\s*-->").unwrap()
    });

    let mut sections = Vec::new();
    let matches: Vec<_> = SECTION_RE.captures_iter(rendered).collect();

    for (i, caps) in matches.iter().enumerate() {
        let key = caps.get(1).unwrap().as_str().to_string();
        let required = caps.get(2).unwrap().as_str() == "required";
        let start = caps.get(0).unwrap().end();
        let end = matches
            .get(i + 1)
            .map(|m| m.get(0).unwrap().start())
            .unwrap_or(rendered.len());

        // Content after marker, excluding the marker itself
        let content = rendered[start..end].trim().to_string();
        if !content.is_empty() || required {
            sections.push(ParsedSection {
                key,
                required,
                content,
            });
        }
    }

    sections
}

/// Apply budget to parsed sections, dropping droppable sections as needed.
///
/// Drop order: tree -> assumptions -> questions -> history -> failure -> planner
fn apply_budget_to_sections(sections: &mut Vec<ParsedSection>, budget: usize) {
    let total_len =
        |secs: &[ParsedSection]| -> usize { secs.iter().map(|s| s.content.len()).sum() };

    if total_len(sections) <= budget {
        return;
    }

    let drop_order = [
        "tree",
        "assumptions",
        "questions",
        "history",
        "failure",
        "planner",
    ];
    for key in drop_order {
        if total_len(sections) <= budget {
            break;
        }
        if let Some(idx) = sections.iter().position(|s| s.key == key && !s.required) {
            let dropped_len = sections[idx].content.len();
            debug!(
                section = key,
                bytes_dropped = dropped_len,
                "dropped section for budget"
            );
            sections.remove(idx);
        }
    }

    // If still over budget, truncate the last section
    if total_len(sections) > budget && !sections.is_empty() {
        let other_len: usize = sections
            .iter()
            .take(sections.len() - 1)
            .map(|s| s.content.len())
            .sum();
        let allowed = budget.saturating_sub(other_len);
        let last = sections.last_mut().unwrap();
        let before_len = last.content.len();
        if last.content.len() > allowed {
            if allowed > 12 {
                last.content.truncate(allowed - 12);
                last.content.push_str("\n[truncated]");
            } else {
                last.content.truncate(allowed);
            }
            debug!(
                section = last.key,
                before_len,
                after_len = last.content.len(),
                "truncated section for budget"
            );
        }
    }
}

/// Render sections back to a single string.
fn render_sections(sections: &[ParsedSection]) -> String {
    sections
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// All inputs needed to build a prompt pack.
#[derive(Debug, Clone)]
pub struct PromptInputs {
    /// Path from root to selected node (e.g., "root/child/leaf").
    pub selected_path: String,
    /// The selected node to work on.
    pub selected_node: Node,
    /// Bounded summary of the full tree.
    pub tree_summary: String,
    /// Goal content from `.runner/context/goal.md`.
    pub context_goal: String,
    /// History content from `.runner/context/history.md`.
    pub context_history: Option<String>,
    /// Failure content from `.runner/context/failure.md`.
    pub context_failure: Option<String>,
    /// Accumulated assumptions from `.runner/state/assumptions.md`.
    pub assumptions: String,
    /// Open questions from `.runner/state/questions.md`.
    pub questions: String,
}

impl PromptInputs {
    /// Load prompt inputs from disk given context and state directories.
    pub fn from_root(
        root: &Path,
        selected_path: String,
        selected_node: Node,
        tree_summary: String,
    ) -> Result<Self> {
        let context_dir = root.join(".runner").join("context");
        let state_dir = root.join(".runner").join("state");

        Ok(Self {
            selected_path,
            selected_node,
            tree_summary,
            context_goal: read_optional(context_dir.join("goal.md"))?.unwrap_or_default(),
            context_history: read_optional(context_dir.join("history.md"))?,
            context_failure: read_optional(context_dir.join("failure.md"))?,
            assumptions: read_optional(state_dir.join("assumptions.md"))?.unwrap_or_default(),
            questions: read_optional(state_dir.join("questions.md"))?.unwrap_or_default(),
        })
    }
}

/// Builds a prompt pack within a byte budget, dropping less critical sections first.
#[derive(Debug, Clone)]
pub struct PromptBuilder {
    budget_bytes: usize,
}

impl PromptBuilder {
    /// Create a builder with the given byte budget.
    pub fn new(budget_bytes: usize) -> Self {
        Self { budget_bytes }
    }

    /// Build a prompt pack for the decomposer agent.
    pub fn build_decomposer(&self, input: &PromptInputs) -> PromptPack {
        let engine = PromptEngine::new();
        let rendered = engine
            .render_decomposer(input)
            .expect("decomposer template rendering should not fail");

        let mut sections = parse_sections(&rendered);
        apply_budget_to_sections(&mut sections, self.budget_bytes);

        PromptPack {
            content: render_sections(&sections),
        }
    }

    /// Build a prompt pack for the executor agent.
    pub fn build_executor(&self, input: &PromptInputs, planner_notes: Option<&str>) -> PromptPack {
        let engine = PromptEngine::new();
        let rendered = engine
            .render_executor(input, planner_notes)
            .expect("executor template rendering should not fail");

        let mut sections = parse_sections(&rendered);
        apply_budget_to_sections(&mut sections, self.budget_bytes);

        PromptPack {
            content: render_sections(&sections),
        }
    }
}

/// A rendered prompt ready to send to the executor.
#[derive(Debug, Clone)]
pub struct PromptPack {
    content: String,
}

impl PromptPack {
    /// Get the rendered prompt content.
    pub fn render(&self) -> String {
        self.content.clone()
    }
}

/// Read file contents if it exists, returning `None` for missing files.
fn read_optional(path: impl Into<PathBuf>) -> Result<Option<String>> {
    let path = path.into();
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("read prompt input {}", path.display()))?;
    Ok(Some(contents))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::default_tree;

    /// Verifies prompt sections appear in deterministic order.
    ///
    /// Order matters for prompt consistency: contract -> goal -> history -> failure ->
    /// selected -> tree -> assumptions -> questions.
    #[test]
    fn prompt_ordering_is_stable() {
        let input = PromptInputs {
            selected_path: "root>child".to_string(),
            selected_node: default_tree(),
            tree_summary: "summary".to_string(),
            context_goal: "goal".to_string(),
            context_history: Some("history".to_string()),
            context_failure: Some("failure".to_string()),
            assumptions: "assumptions".to_string(),
            questions: "questions".to_string(),
        };

        let pack = PromptBuilder::new(10_000).build_executor(&input, None);
        let content = pack.render();

        // Verify sections appear in expected order
        let contract_pos = content
            .find("### Executor Contract")
            .expect("contract section");
        let goal_pos = content.find("### Goal").expect("goal section");
        let history_pos = content.find("### History").expect("history section");
        let failure_pos = content.find("### Failure").expect("failure section");
        let selected_pos = content.find("### Selected Node").expect("selected section");
        let tree_pos = content.find("### Tree Summary").expect("tree section");
        let assumptions_pos = content
            .find("### Assumptions")
            .expect("assumptions section");
        let questions_pos = content
            .find("### Open Questions")
            .expect("questions section");

        assert!(contract_pos < goal_pos, "contract before goal");
        assert!(goal_pos < history_pos, "goal before history");
        assert!(history_pos < failure_pos, "history before failure");
        assert!(failure_pos < selected_pos, "failure before selected");
        assert!(selected_pos < tree_pos, "selected before tree");
        assert!(tree_pos < assumptions_pos, "tree before assumptions");
        assert!(
            assumptions_pos < questions_pos,
            "assumptions before questions"
        );
    }

    /// Verifies budget enforcement drops less critical sections first.
    ///
    /// With a tight budget, tree and assumptions (low priority) should be dropped
    /// while required sections (contract, goal, selected) remain.
    #[test]
    fn budget_drops_less_critical_sections_first() {
        let input = PromptInputs {
            selected_path: "root>child".to_string(),
            selected_node: default_tree(),
            tree_summary: "tree".repeat(200),
            context_goal: "goal".to_string(),
            context_history: Some("history".to_string()),
            context_failure: Some("failure".to_string()),
            assumptions: "assumptions".repeat(50),
            questions: "questions".repeat(50),
        };

        let pack = PromptBuilder::new(600).build_executor(&input, None);
        let content = pack.render();

        // Tree and assumptions should be dropped (low priority droppable sections)
        assert!(
            !content.contains("### Tree Summary"),
            "tree should be dropped"
        );
        assert!(
            !content.contains("### Assumptions"),
            "assumptions should be dropped"
        );
        // Required sections should remain
        assert!(
            content.contains("### Executor Contract"),
            "contract should remain"
        );
        assert!(content.contains("### Goal"), "goal should remain");
        assert!(
            content.contains("### Selected Node"),
            "selected should remain"
        );
    }

    /// Verifies template renders with XML tags for semantic structure.
    #[test]
    fn template_uses_xml_tags() {
        let input = PromptInputs {
            selected_path: "root".to_string(),
            selected_node: default_tree(),
            tree_summary: "".to_string(),
            context_goal: "test goal".to_string(),
            context_history: None,
            context_failure: None,
            assumptions: "".to_string(),
            questions: "".to_string(),
        };

        let pack = PromptBuilder::new(10_000).build_decomposer(&input);
        let content = pack.render();

        assert!(content.contains("<contract>"), "should have contract tag");
        assert!(
            content.contains("</contract>"),
            "should have contract close tag"
        );
        assert!(content.contains("<goal>"), "should have goal tag");
        assert!(content.contains("</goal>"), "should have goal close tag");
        assert!(content.contains("<selected>"), "should have selected tag");
        assert!(
            content.contains("</selected>"),
            "should have selected close tag"
        );
    }
}
