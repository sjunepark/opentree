//! Agent abstractions for tree planning and execution.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

pub mod executor;
pub mod tree;

pub(crate) fn write_output_schema(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create schema dir {}", parent.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("write schema {}", path.display()))
}
