//! Memory Query Tool
//! 
//! Allows agents to search their own memory.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::debug;

use super::{Tool, ToolOutput};
use crate::memory::Memory;

/// Tool for querying the memory system
pub struct MemoryQueryTool {
    memory: Arc<dyn Memory>,
}

impl MemoryQueryTool {
    pub fn new(memory: Arc<dyn Memory>) -> Self {
        Self { memory }
    }
}

#[async_trait]
impl Tool for MemoryQueryTool {
    fn name(&self) -> &str {
        "memory_query"
    }

    fn description(&self) -> &str {
        "Search your memory for past interactions, learned information, or context. \
         Use this when you need to recall previous conversations or find relevant information \
         from past interactions."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to find relevant memories"
                },
                "top_k": {
                    "type": "integer",
                    "description": "Number of results to return (default: 3, max: 10)",
                    "default": 3
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let query = params["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;
        
        let top_k = params["top_k"]
            .as_u64()
            .unwrap_or(3)
            .min(10) as usize;

        debug!("Querying memory for: {} (top {})", query, top_k);

        match self.memory.search(query, top_k).await {
            Ok(entries) => {
                if entries.is_empty() {
                    return Ok(ToolOutput::success_str(
                        "No relevant memories found for this query."
                    ));
                }

                let formatted = entries
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let similarity = e.similarity
                            .map(|s| format!(" (relevance: {:.2})", s))
                            .unwrap_or_default();
                        
                        format!(
                            "{}. [{}{}]\n   {}\n   Time: {}",
                            i + 1,
                            e.metadata.agent,
                            similarity,
                            e.content.chars().take(500).collect::<String>(),
                            e.timestamp.format("%Y-%m-%d %H:%M")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n");

                let summary = format!(
                    "Found {} relevant memories:\n\n{}",
                    entries.len(),
                    formatted
                );

                Ok(ToolOutput::success(
                    json!({
                        "query": query,
                        "num_results": entries.len(),
                        "memories": entries.iter().map(|e| json!({
                            "id": e.id,
                            "content": e.content,
                            "agent": e.metadata.agent,
                            "timestamp": e.timestamp.to_rfc3339(),
                            "similarity": e.similarity
                        })).collect::<Vec<_>>()
                    }),
                    summary
                ))
            }
            Err(e) => {
                Ok(ToolOutput::failure(format!("Memory search failed: {}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{VectorMemory, MemoryEntry, entry::MemorySource};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_memory_query_tool_execute() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("memory.json");
        let memory = Arc::new(VectorMemory::new(path).unwrap());
        
        // Seed some memory
        memory.store(MemoryEntry::new("Rust is a systems programming language", "test", MemorySource::User)).await.unwrap();
        
        let tool = MemoryQueryTool::new(memory);
        let res = tool.execute(json!({"query": "what is rust?"})).await.unwrap();
        
        assert!(res.success);
        assert!(res.summary.contains("Found 1 relevant memories"));
        assert!(res.summary.contains("Rust is a systems programming language"));
    }

    #[tokio::test]
    async fn test_memory_query_tool_empty() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("memory.json");
        let memory = Arc::new(VectorMemory::new(path).unwrap());
        
        let tool = MemoryQueryTool::new(memory);
        let res = tool.execute(json!({"query": "anything?"})).await.unwrap();
        
        assert!(res.success);
        assert!(res.summary.contains("No relevant memories found"));
    }
}
