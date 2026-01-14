//! Memory Entry types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Metadata associated with a memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    /// Which agent created this memory
    pub agent: String,
    /// The BoundedContext where this memory belongs (FPF A.1.1)
    pub context: String,
    /// FPF Integration: U.Kind (C.3)
    pub kind: crate::orchestrator::Kind,
    /// FPF Integration: Episteme Slot Graph (C.2.1)
    /// The subject of this knowledge (e.g. "Function handle()")
    pub described_entity: Option<String>,
    /// The physical source of truth (e.g. "file://src/main.rs")
    pub grounding_holon: Option<String>,
    /// The lens used (e.g. "Technical", "Security")
    pub viewpoint: Option<String>,
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
        let content_str = content.into();
        let kind = crate::orchestrator::Kind::detect(&content_str);

        Self {
            id: Uuid::new_v4().to_string(),
            query: None,
            content: content_str,
            metadata: MemoryMetadata {
                agent: agent.into(),
                context: "General".to_string(), // Default FPF Context
                kind,
                described_entity: None,
                grounding_holon: None,
                viewpoint: None,
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
    #[allow(dead_code)]
    pub fn from_interaction(
        query: impl Into<String>,
        response: impl Into<String>,
        agent: impl Into<String>,
    ) -> Self {
        let q = query.into();
        let r = response.into();
        let combined = format!("Query: {}\nResponse: {}", q, r);
        let kind = crate::orchestrator::Kind::detect(&combined);
        
        Self {
            id: Uuid::new_v4().to_string(),
            query: Some(q),
            content: combined,
            metadata: MemoryMetadata {
                agent: agent.into(),
                context: "General".to_string(),
                kind,
                described_entity: None,
                grounding_holon: None,
                viewpoint: None,
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
    #[allow(dead_code)]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.metadata.tags = tags;
        self
    }

    /// Set importance score
    #[allow(dead_code)]
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.metadata.importance = importance.clamp(0.0, 1.0);
        self
    }

    pub fn with_grounding(mut self, entity: impl Into<String>, source: impl Into<String>) -> Self {
        self.metadata.described_entity = Some(entity.into());
        self.metadata.grounding_holon = Some(source.into());
        self
    }

    pub fn with_viewpoint(mut self, viewpoint: impl Into<String>) -> Self {
        self.metadata.viewpoint = Some(viewpoint.into());
        self
    }

    /// Set the BoundedContext for this memory (FPF A.1.1)
    #[allow(dead_code)]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.metadata.context = context.into();
        self
    }
}

impl Default for MemoryMetadata {

    fn default() -> Self {

        Self {

            agent: "unknown".to_string(),

            context: "General".to_string(),

            kind: crate::orchestrator::Kind::Theoretical,

            described_entity: None,

            grounding_holon: None,

            viewpoint: None,

            source: MemorySource::System,

            importance: 0.5,

            access_count: 0,

            tags: Vec::new(),

        }

    }

}
