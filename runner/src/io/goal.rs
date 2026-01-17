//! Helpers for reading and writing `.runner/GOAL.md`.
//!
//! We treat the goal document as a human-facing spec, but we also store a
//! stable `id` in YAML frontmatter to tie runs to a goal identifier.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};

/// Read the `id` from YAML frontmatter in the goal document, if present.
pub fn read_goal_id(path: &Path) -> Result<Option<String>> {
    let contents = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(parse_frontmatter_id(&contents))
}

/// Ensure the goal document has an `id` set in YAML frontmatter.
///
/// Preserves any existing frontmatter keys, updating `id` if present.
pub fn ensure_goal_id(path: &Path, id: &str) -> Result<()> {
    validate_id(id)?;
    let contents = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let updated = set_frontmatter_id(&contents, id);
    fs::write(path, updated).with_context(|| format!("write {}", path.display()))
}

/// Validate that an id is safe for use in `runner/<id>` branch names.
pub fn validate_id(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(anyhow!("id must not be empty"));
    }
    if id.contains('/') {
        return Err(anyhow!("id must not contain '/'"));
    }
    if id
        .chars()
        .any(|c| !(c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-'))
    {
        return Err(anyhow!("id must be [A-Za-z0-9._-] only (got '{id}')"));
    }
    Ok(())
}

fn parse_frontmatter_id(contents: &str) -> Option<String> {
    let (frontmatter, _) = split_frontmatter(contents)?;
    let frontmatter = frontmatter?;
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = trimmed.split_once(':')?;
        if key.trim() != "id" {
            continue;
        }
        let mut v = value.trim().to_string();
        if ((v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')))
            && v.len() >= 2
        {
            v = v[1..v.len() - 1].to_string();
        }
        if v.is_empty() {
            return None;
        }
        return Some(v);
    }
    None
}

fn set_frontmatter_id(contents: &str, id: &str) -> String {
    match split_frontmatter(contents) {
        Some((Some(frontmatter), rest)) => {
            let updated_frontmatter = upsert_id_line(frontmatter, id);
            render_frontmatter(&updated_frontmatter, rest)
        }
        Some((None, rest)) => render_frontmatter(&format!("id: {id}\n"), rest),
        None => render_frontmatter(&format!("id: {id}\n"), contents),
    }
}

fn upsert_id_line(frontmatter: &str, id: &str) -> String {
    let mut lines = Vec::new();
    let mut saw_id = false;

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some((key, _)) = trimmed.split_once(':')
            && key.trim() == "id"
        {
            if !saw_id {
                lines.push(format!("id: {id}"));
                saw_id = true;
            }
            continue;
        }
        lines.push(line.to_string());
    }

    if !saw_id {
        lines.insert(0, format!("id: {id}"));
    }

    let mut buf = lines.join("\n");
    if !buf.ends_with('\n') {
        buf.push('\n');
    }
    buf
}

fn render_frontmatter(frontmatter: &str, rest: &str) -> String {
    let mut buf = String::new();
    buf.push_str("---\n");
    buf.push_str(frontmatter.trim_end());
    buf.push('\n');
    buf.push_str("---\n\n");
    buf.push_str(rest.trim_start_matches('\n'));
    if !buf.ends_with('\n') {
        buf.push('\n');
    }
    buf
}

/// Split a document into (frontmatter, rest). Returns None if it doesn't look like frontmatter.
fn split_frontmatter(contents: &str) -> Option<(Option<&str>, &str)> {
    if !contents.starts_with("---\n") {
        return None;
    }
    let after = &contents[4..];
    let end = after.find("\n---\n")?;
    let frontmatter = &after[..end];
    let rest = &after[end + 5..];
    Some((Some(frontmatter), rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_id_from_frontmatter() {
        let doc = "---\nid: run-abc123\n---\n\n# Goal\n";
        assert_eq!(parse_frontmatter_id(doc), Some("run-abc123".to_string()));
    }

    #[test]
    fn ensure_goal_id_inserts_frontmatter_when_missing() {
        let doc = "# Goal\n\nHello\n";
        let updated = set_frontmatter_id(doc, "run-1");
        assert!(updated.starts_with("---\nid: run-1\n---"));
        assert!(updated.contains("# Goal"));
    }

    #[test]
    fn set_frontmatter_id_updates_existing_id() {
        let doc = "---\nid: run-old\nfoo: bar\n---\n\n# Goal\n";
        let updated = set_frontmatter_id(doc, "run-new");
        assert!(updated.contains("id: run-new"));
        assert!(updated.contains("foo: bar"));
        assert!(!updated.contains("id: run-old"));
    }

    #[test]
    fn validate_id_rejects_slash() {
        let err = validate_id("bad/id").unwrap_err();
        assert!(err.to_string().contains("must not contain"));
    }
}
