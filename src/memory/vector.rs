//! Vector Memory Implementation using fastembed
//! 
//! Provides semantic search over stored memories using vector embeddings.
//! Uses file-based persistence with an in-memory cache for high performance.

use anyhow::{Context, Result};
use async_trait::async_trait;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::{Memory, MemoryEntry};

/// Vector memory backed by file storage with semantic search
pub struct VectorMemory {
    path: PathBuf,
    embedder: Arc<RwLock<TextEmbedding>>,
    /// In-memory cache of entries to avoid redundant disk I/O
    cache: Arc<RwLock<Vec<MemoryEntry>>>,
}

impl VectorMemory {
    /// Create a new VectorMemory instance
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        info!("Initializing VectorMemory at {:?}", path);
        
        let embedder = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(true)
        ).context("Failed to initialize embedding model")?;

        let mut memory = Self {
            path,
            embedder: Arc::new(RwLock::new(embedder)),
            cache: Arc::new(RwLock::new(Vec::new())),
        };

        // Warm up the cache from disk
        memory.load_to_cache_sync()?;
        
        Ok(memory)
    }

    /// Load entries from disk into the in-memory cache
    fn load_to_cache_sync(&mut self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }
        let content = std::fs::read_to_string(&self.path)?;
        let entries: Vec<MemoryEntry> = serde_json::from_str(&content)?;
        
        // We use a sync block here only during initialization
        let mut cache = self.cache.try_write()
            .map_err(|_| anyhow::anyhow!("Failed to lock cache during init"))?;
        *cache = entries;
        info!("Loaded {} entries into memory cache", cache.len());
        Ok(())
    }

    /// Generate embeddings for text and normalize them
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let embedder = self.embedder.read().await;
        let mut embeddings = embedder.embed(texts.to_vec(), None)
            .context("Failed to generate embeddings")?;
        
        // Normalize embeddings for faster dot-product similarity
        for emb in &mut embeddings {
            Self::normalize(emb);
        }
        
        Ok(embeddings)
    }

    fn normalize(vec: &mut Vec<f32>) {
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in vec {
                *x /= norm;
            }
        }
    }

    /// Persist the current cache to disk (async)
    async fn persist(&self) -> Result<()> {
        let cache = self.cache.read().await;
        let content = serde_json::to_string_pretty(&*cache)?;
        tokio::fs::write(&self.path, content).await?;
        Ok(())
    }

    /// Dot product for normalized vectors (equivalent to cosine similarity)
    fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }
}

#[async_trait]
impl Memory for VectorMemory {
    /// Store a new memory entry
    async fn store(&self, mut entry: MemoryEntry) -> Result<String> {
        let start = std::time::Instant::now();
        // Generate embedding if missing
        if entry.embedding.is_none() {
            debug!("Generating embedding for entry: {}", entry.id);
            let embeddings = self.embed(&[entry.content.clone()]).await?;
            entry.embedding = Some(embeddings[0].clone());
        }

        let mut cache = self.cache.write().await;
        
        // Deduplication logic: If this is a codebase file, remove the old version first
        if let Some(ref query) = entry.query {
            if entry.metadata.agent == "CodebaseIndexer" {
                cache.retain(|e| e.query.as_ref() != Some(query));
            }
        }

        let id = entry.id.clone();
        cache.push(entry);
        
        let elapsed = start.elapsed();
        debug!("Memory entry {} stored in-memory in {:?}", id, elapsed);
        
        Ok(id)
    }

    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<MemoryEntry>> {
        let start = std::time::Instant::now();
        debug!("Searching memory for: {} (top {})", query, top_k);
        
        let query_embedding = self.embed(&[query.to_string()]).await?
            .into_iter().next()
            .context("No query embedding generated")?;
        
        let cache = self.cache.read().await;
        
        // 1. Score entries without cloning
        let mut scored: Vec<(f32, usize)> = cache
            .iter()
            .enumerate()
            .filter_map(|(idx, e)| {
                e.embedding.as_ref().map(|emb| {
                    let sim = Self::dot_product(&query_embedding, emb);
                    (sim, idx)
                })
            })
            .collect();
        
        // 2. Sort by score
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // 3. Clone only the top K results
        let results: Vec<MemoryEntry> = scored.into_iter()
            .take(top_k)
            .map(|(score, idx)| {
                let mut entry = cache[idx].clone();
                entry.similarity = Some(score);
                entry
            })
            .collect();
            
        debug!("Found {} memory entries in {:?}", results.len(), start.elapsed());
        
        Ok(results)
    }

    async fn count(&self) -> Result<usize> {
        let cache = self.cache.read().await;
        Ok(cache.len())
    }

    async fn persist(&self) -> Result<()> {
        self.persist().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_entry_creation() {
        let entry = MemoryEntry::new("Test content", "TestAgent", super::super::entry::MemorySource::Agent);
        assert!(!entry.id.is_empty());
        assert_eq!(entry.content, "Test content");
        assert_eq!(entry.metadata.agent, "TestAgent");
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((VectorMemory::dot_product(&a, &b) - 1.0).abs() < 0.001);
        
        let c = vec![0.0, 1.0, 0.0];
        assert!(VectorMemory::dot_product(&a, &c).abs() < 0.001);
    }
}