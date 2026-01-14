//! Codebase Indexer - Indexes source code into Vector Memory
//! 
//! Provides functionality to crawl the project's source directory
//! and store semantic embeddings of code files.
//! Includes Hash-based deduplication to prevent redundant indexing.

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use tracing::{info, debug};
use sha2::{Sha256, Digest};

use crate::memory::{Memory, MemoryEntry};
use crate::memory::entry::MemorySource;

/// Indexer for codebase semantic search
pub struct CodebaseIndexer {
    src_dir: PathBuf,
    memory: Arc<dyn Memory>,
    /// Cache of file hashes to prevent redundant indexing
    hash_cache: Arc<Mutex<HashMap<String, String>>>,
}

impl CodebaseIndexer {
    pub fn new(src_dir: impl Into<PathBuf>, memory: Arc<dyn Memory>) -> Self {
        Self {
            src_dir: src_dir.into(),
            memory,
            hash_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Recursively index the source directory
    pub async fn index_all(&self) -> Result<usize> {
        let start = std::time::Instant::now();
        info!("Indexing codebase at {:?}", self.src_dir);
        let mut count = 0;
        let mut skipped = 0;
        let mut dirs = vec![self.src_dir.clone()];

        while let Some(dir) = dirs.pop() {
            let mut entries = fs::read_dir(dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    let path_str = path.to_string_lossy();
                    if !path_str.contains("target") && !path_str.contains(".git") && !path_str.contains(".fastembed_cache") {
                        dirs.push(path);
                    }
                } else if self.is_source_file(&path) {
                    // Throttle indexing to prevent hardware saturation
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    match self.index_file(&path).await? {
                        true => count += 1,
                        false => skipped += 1,
                    }
                }
            }
        }

        if count > 0 {
            debug!("Persisting vector memory to disk...");
            self.memory.persist().await?;
        }

        let elapsed = start.elapsed();
        info!("Indexing complete: {} new/updated, {} skipped (unchanged) in {:?}", count, skipped, elapsed);
        Ok(count)
    }

    fn is_source_file(&self, path: &Path) -> bool {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        matches!(ext, "rs" | "py" | "js" | "sh" | "toml" | "md")
    }

    async fn index_file(&self, path: &Path) -> Result<bool> {
        let content = fs::read_to_string(path).await?;
        if content.trim().is_empty() {
            return Ok(false);
        }

        // Calculate hash
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        let rel_path = path.strip_prefix(&self.src_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Check if hash changed
        let mut cache = self.hash_cache.lock().await;
        if let Some(old_hash) = cache.get(&rel_path) {
            if old_hash == &hash {
                return Ok(false); // Unchanged
            }
        }
        
        debug!("Indexing file: {} (hash changed)", rel_path);
        cache.insert(rel_path.clone(), hash);

        let mut entry = MemoryEntry::new(
            format!("File: {}\n\nContent:\n{}", rel_path, content),
            "CodebaseIndexer",
            MemorySource::Codebase
        );
        entry.query = Some(format!("Source code for {}", rel_path));
        
        self.memory.store(entry).await?;
        Ok(true)
    }
}
