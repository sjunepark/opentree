//! Result caching with content-addressed storage.
//!
//! Results are stored at `results/{agent}/{prompt_hash}/{input_id}.json`.
//! The prompt hash is derived from the template content, enabling cache invalidation
//! when prompts change.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use tracing::debug;

use crate::runner::CombinationResult;

/// Compute a short hash of content for cache keys.
pub fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8]) // First 8 bytes = 16 hex chars
}

/// Cache for storing and retrieving combination results.
pub struct ResultCache {
    results_dir: PathBuf,
}

impl ResultCache {
    pub fn new(lab_root: &Path) -> Self {
        Self {
            results_dir: lab_root.join("results"),
        }
    }

    /// Get the cache path for a combination.
    pub fn cache_path(&self, agent: &str, prompt_hash: &str, input_id: &str) -> PathBuf {
        self.results_dir
            .join(agent)
            .join(prompt_hash)
            .join(format!("{}.json", input_id))
    }

    /// Check if a cached result exists.
    pub fn has_cached(&self, agent: &str, prompt_hash: &str, input_id: &str) -> bool {
        self.cache_path(agent, prompt_hash, input_id).exists()
    }

    /// Load a cached result if it exists.
    #[allow(dead_code)]
    pub fn get_cached(
        &self,
        agent: &str,
        prompt_hash: &str,
        input_id: &str,
    ) -> Result<Option<CombinationResult>> {
        let path = self.cache_path(agent, prompt_hash, input_id);
        if !path.exists() {
            return Ok(None);
        }

        debug!(path = %path.display(), "loading cached result");
        let content =
            fs::read_to_string(&path).with_context(|| format!("read cache {}", path.display()))?;
        let result: CombinationResult = serde_json::from_str(&content)
            .with_context(|| format!("parse cache {}", path.display()))?;
        Ok(Some(result))
    }

    /// Save a result to the cache.
    pub fn save_result(
        &self,
        agent: &str,
        prompt_hash: &str,
        input_id: &str,
        result: &CombinationResult,
    ) -> Result<()> {
        let path = self.cache_path(agent, prompt_hash, input_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create cache dir {}", parent.display()))?;
        }

        debug!(path = %path.display(), "saving result to cache");
        let content = serde_json::to_string_pretty(result)?;
        fs::write(&path, content).with_context(|| format!("write cache {}", path.display()))?;
        Ok(())
    }

    /// List all cached results for an agent.
    pub fn list_results(&self, agent: &str) -> Result<Vec<CombinationResult>> {
        let agent_dir = self.results_dir.join(agent);
        if !agent_dir.exists() {
            return Ok(vec![]);
        }

        let mut results = Vec::new();
        for prompt_entry in fs::read_dir(&agent_dir)? {
            let prompt_entry = prompt_entry?;
            if !prompt_entry.file_type()?.is_dir() {
                continue;
            }

            // Skip index.json
            let path = prompt_entry.path();
            if path.file_name().is_some_and(|n| n == "index.json") {
                continue;
            }

            for input_entry in fs::read_dir(prompt_entry.path())? {
                let input_entry = input_entry?;
                let path = input_entry.path();
                if path.extension().is_some_and(|e| e == "json") {
                    let content = fs::read_to_string(&path)?;
                    if let Ok(result) = serde_json::from_str::<CombinationResult>(&content) {
                        results.push(result);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Generate an index.json for the dashboard.
    pub fn generate_index(&self, agent: &str) -> Result<()> {
        let results = self.list_results(agent)?;
        if results.is_empty() {
            return Ok(());
        }

        let prompts: Vec<String> = {
            let mut set: std::collections::HashSet<_> =
                results.iter().map(|r| r.prompt_name.clone()).collect();
            let mut v: Vec<_> = set.drain().collect();
            v.sort();
            v
        };

        let inputs: Vec<String> = {
            let mut set: std::collections::HashSet<_> =
                results.iter().map(|r| r.input_id.clone()).collect();
            let mut v: Vec<_> = set.drain().collect();
            v.sort();
            v
        };

        let index = serde_json::json!({
            "prompts": prompts,
            "inputs": inputs,
            "results": results,
        });

        let index_path = self.results_dir.join(agent).join("index.json");
        fs::create_dir_all(index_path.parent().unwrap())?;
        fs::write(&index_path, serde_json::to_string_pretty(&index)?)?;
        debug!(path = %index_path.display(), "wrote index.json");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_deterministic() {
        let content = "Hello, world!";
        let hash1 = content_hash(content);
        let hash2 = content_hash(content);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 16); // 8 bytes = 16 hex chars
    }

    #[test]
    fn test_content_hash_different_inputs() {
        let hash1 = content_hash("Hello");
        let hash2 = content_hash("World");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_cache_path() {
        let temp = tempfile::tempdir().unwrap();
        let cache = ResultCache::new(temp.path());
        let path = cache.cache_path("decomposer", "abc123", "test_input");
        assert!(path.ends_with("results/decomposer/abc123/test_input.json"));
    }
}
