//! Memory Entry types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Metadata associated with a memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    /// Which agent created this memory
    pub agent: String,
    /// Source of the information (user, tool, reflection)
    pub source: MemorySource,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
    /// Number of times this memory was accessed
    pub access_count: u32,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Source of a memory entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MemorySource {
    User,
    Agent,
    Tool,
    Reflection,
    System,
    Codebase,
}

/// A single memory entry with content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier
    pub id: String,
    /// The user's original query (if applicable)
    pub query: Option<String>,
    /// The content/response to store
    pub content: String,
    /// Associated metadata
    pub metadata: MemoryMetadata,
    /// When this memory was created
    pub timestamp: DateTime<Utc>,
    /// Optional embedding (populated on retrieval)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    /// Similarity score (only set during search results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity: Option<f32>,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(content: impl Into<String>, agent: impl Into<String>, source: MemorySource) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            query: None,
            content: content.into(),
            metadata: MemoryMetadata {
                agent: agent.into(),
                source,
                importance: 0.5,
                access_count: 0,
                tags: Vec::new(),
            },
            timestamp: Utc::now(),
            embedding: None,
            similarity: None,
        }
    }

    /// Create an entry from a user query and agent response
    pub fn from_interaction(
        query: impl Into<String>,
        response: impl Into<String>,
        agent: impl Into<String>,
    ) -> Self {
        let q = query.into();
        let r = response.into();
        let combined = format!("Query: {}\nResponse: {}", q, r);
        
        Self {
            id: Uuid::new_v4().to_string(),
            query: Some(q),
            content: combined,
            metadata: MemoryMetadata {
                agent: agent.into(),
                source: MemorySource::Agent,
                importance: 0.5,
                access_count: 0,
                tags: Vec::new(),
            },
            timestamp: Utc::now(),
            embedding: None,
            similarity: None,
        }
    }

    /// Add tags to this entry
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.metadata.tags = tags;
        self
    }

    /// Set importance score
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.metadata.importance = importance.clamp(0.0, 1.0);
        self
    }
}

impl Default for MemoryMetadata {
    fn default() -> Self {
        Self {
            agent: "unknown".to_string(),
            source: MemorySource::System,
            importance: 0.5,
            access_count: 0,
            tags: Vec::new(),
        }
    }
}
