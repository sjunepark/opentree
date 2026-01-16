use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub order: i64,
    pub title: String,
    pub goal: String,
    pub acceptance: Vec<String>,
    pub passes: bool,
    pub attempts: u32,
    pub max_attempts: u32,
    pub children: Vec<Node>,
}

impl Node {
    pub fn sort_children(&mut self) {
        self.children
            .sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.id.cmp(&b.id)));
        for child in &mut self.children {
            child.sort_children();
        }
    }
}

pub fn default_tree() -> Node {
    Node {
        id: "root".to_string(),
        order: 0,
        title: "Root".to_string(),
        goal: "Top-level goal (see .runner/GOAL.md)".to_string(),
        acceptance: Vec::new(),
        passes: false,
        attempts: 0,
        max_attempts: 3,
        children: Vec::new(),
    }
}
