//! Memory System Module
//! 
//! Provides semantic vector memory using fastembed,
//! plus episodic conversation history.

pub mod vector;
pub mod episodic;
pub mod entry;
pub mod manager;
pub mod indexer;

pub use vector::VectorMemory;
pub use episodic::EpisodicMemory;
pub use entry::MemoryEntry;
pub use manager::MemoryManager;
pub use indexer::CodebaseIndexer;

use anyhow::Result;
use async_trait::async_trait;

/// Trait for memory systems that can store and retrieve entries
#[async_trait]
pub trait Memory: Send + Sync {
    /// Store a new memory entry
    async fn store(&self, entry: MemoryEntry) -> Result<String>;
    
    /// Search for relevant memories given a query
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<MemoryEntry>>;
    
    /// Get total number of entries
    async fn count(&self) -> Result<usize>;

    /// Persist memory to storage
    async fn persist(&self) -> Result<()>;
}
