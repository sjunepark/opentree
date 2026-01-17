//! Prompt pack builder for deterministic executor input.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::tree::Node;

const RUNNER_CONTRACT: &str = "Runner contract:\n- Do not modify passed nodes.\n- Do not set `passes=true` (runner-owned).\n- Only add children when declaring `decomposed`.\n- Output must be structured JSON with `status` and `summary`.";
const OUTPUT_CONTRACT: &str =
    "Output contract:\nYou MUST write output.json with `status` and `summary` before session ends.";

#[derive(Debug, Clone)]
pub struct PromptInputs {
    pub selected_path: String,
    pub selected_node: Node,
    pub tree_summary: String,
    pub context_goal: String,
    pub context_history: Option<String>,
    pub context_failure: Option<String>,
    pub assumptions: String,
    pub questions: String,
}

impl PromptInputs {
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

#[derive(Debug, Clone)]
pub struct PromptBuilder {
    budget_bytes: usize,
}

impl PromptBuilder {
    pub fn new(budget_bytes: usize) -> Self {
        Self { budget_bytes }
    }

    pub fn build(&self, input: &PromptInputs) -> PromptPack {
        let mut sections = vec![
            PromptSection::required("contract", "Runner Contract", RUNNER_CONTRACT),
            PromptSection::required("goal", "Goal", input.context_goal.trim()),
            PromptSection::droppable(
                "history",
                "History (previous attempt)",
                input.context_history.as_deref().unwrap_or("").trim(),
            ),
            PromptSection::droppable(
                "failure",
                "Failure (guard output)",
                input.context_failure.as_deref().unwrap_or("").trim(),
            ),
            PromptSection::required(
                "selected",
                "Selected Node",
                &render_selected_node(&input.selected_path, &input.selected_node),
            ),
            PromptSection::droppable("tree", "Tree Summary", input.tree_summary.trim()),
            PromptSection::droppable("assumptions", "Assumptions", input.assumptions.trim()),
            PromptSection::droppable("questions", "Open Questions", input.questions.trim()),
            PromptSection::required("output", "Output Contract", OUTPUT_CONTRACT),
        ];

        sections.retain(|s| !s.content.is_empty() || s.required);
        apply_budget(&mut sections, self.budget_bytes);

        PromptPack { sections }
    }
}

#[derive(Debug, Clone)]
pub struct PromptPack {
    pub sections: Vec<PromptSection>,
}

impl PromptPack {
    pub fn render(&self) -> String {
        let mut buf = String::new();
        for section in &self.sections {
            buf.push_str(&section.render());
            buf.push('\n');
        }
        buf
    }
}

#[derive(Debug, Clone)]
pub struct PromptSection {
    key: &'static str,
    title: String,
    content: String,
    required: bool,
}

impl PromptSection {
    fn required(key: &'static str, title: &str, content: &str) -> Self {
        Self {
            key,
            title: title.to_string(),
            content: content.to_string(),
            required: true,
        }
    }

    fn droppable(key: &'static str, title: &str, content: &str) -> Self {
        Self {
            key,
            title: title.to_string(),
            content: content.to_string(),
            required: false,
        }
    }

    fn render(&self) -> String {
        format!("### {}\n\n{}\n", self.title, self.content)
    }

    fn render_len(&self) -> usize {
        self.render().len()
    }

    fn truncate_to(&mut self, max_len: usize) {
        let header = format!("### {}\n\n", self.title);
        let footer = "\n";
        let available = max_len.saturating_sub(header.len() + footer.len());
        if self.content.len() <= available {
            return;
        }
        if available == 0 {
            self.content.clear();
            return;
        }
        let suffix = "\n[truncated]";
        if available <= suffix.len() {
            self.content = suffix[..available].to_string();
            return;
        }
        self.content.truncate(available - suffix.len());
        self.content.push_str(suffix);
    }
}

fn read_optional(path: impl Into<PathBuf>) -> Result<Option<String>> {
    let path = path.into();
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("read prompt input {}", path.display()))?;
    Ok(Some(contents))
}

fn render_selected_node(path: &str, node: &Node) -> String {
    let mut buf = String::new();
    buf.push_str(&format!("path: {}\n", path));
    buf.push_str(&format!("id: {}\n", node.id));
    buf.push_str(&format!("title: {}\n", node.title));
    buf.push_str(&format!("goal: {}\n", node.goal));
    if !node.acceptance.is_empty() {
        buf.push_str("acceptance:\n");
        for item in &node.acceptance {
            buf.push_str(&format!("- {}\n", item));
        }
    }
    buf
}

fn apply_budget(sections: &mut Vec<PromptSection>, budget: usize) {
    let mut total: usize = sections.iter().map(PromptSection::render_len).sum();
    if total <= budget {
        return;
    }

    let drop_order = ["tree", "assumptions", "questions", "history", "failure"];
    for key in drop_order {
        if total <= budget {
            break;
        }
        if let Some(idx) = sections.iter().position(|s| s.key == key) {
            total = total.saturating_sub(sections[idx].render_len());
            sections.remove(idx);
        }
    }

    if total <= budget || sections.is_empty() {
        return;
    }

    let last_idx = sections.len() - 1;
    let other_len: usize = sections
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx != last_idx)
        .map(|(_, s)| s.render_len())
        .sum();
    let allowed = budget.saturating_sub(other_len);
    sections[last_idx].truncate_to(allowed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::default_tree;

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

        let pack = PromptBuilder::new(10_000).build(&input);
        let keys: Vec<&str> = pack.sections.iter().map(|s| s.key).collect();
        assert_eq!(
            keys,
            vec![
                "contract",
                "goal",
                "history",
                "failure",
                "selected",
                "tree",
                "assumptions",
                "questions",
                "output"
            ]
        );
    }

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

        let pack = PromptBuilder::new(300).build(&input);
        let keys: Vec<&str> = pack.sections.iter().map(|s| s.key).collect();
        assert!(!keys.contains(&"tree"));
        assert!(!keys.contains(&"assumptions"));
    }
}
