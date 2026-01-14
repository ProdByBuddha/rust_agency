//! Artifact Management Tool
//! 
//! Allows agents to manage persistent files (artifacts) in a dedicated workspace.
//! This is useful for saving code, documentation, or search results.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

use crate::agent::{AgentResult, AgentError};
use super::{Tool, ToolOutput};

/// Tool for managing persistent artifacts
pub struct ArtifactTool {
    /// Base directory for artifacts
    base_dir: PathBuf,
}

impl ArtifactTool {
    /// Create a new ArtifactTool with the specified base directory
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        let base_dir = base_dir.into();
        Self { base_dir }
    }

    /// Ensure the base directory exists
    async fn ensure_dir(&self) -> AgentResult<()> {
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir).await
                .map_err(|e| AgentError::Io(e))?;
        }
        Ok(())
    }

    /// Resolve a path relative to the base directory and ensure it stays within bounds
    fn resolve_path(&self, filename: &str) -> AgentResult<PathBuf> {
        let path = self.base_dir.join(filename);
        
        // Security check: ensure path is within base_dir
        if !path.starts_with(&self.base_dir) {
            return Err(AgentError::Validation("Access denied: Path is outside the artifacts directory".to_string()));
        }
        
        Ok(path)
    }
}

impl Default for ArtifactTool {
    fn default() -> Self {
        Self::new("artifacts")
    }
}

#[async_trait]
impl Tool for ArtifactTool {
    fn name(&self) -> String {
        "artifact_manager".to_string()
    }

    fn description(&self) -> String {
        "Manage artifacts (files, images, documents) generated or used by agents. \n        Supports 'save', 'load', 'list', and 'delete' operations.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["save", "load", "list", "delete"],
                    "description": "The action to perform"
                },
                "name": {
                    "type": "string",
                    "description": "The name/id of the artifact"
                },
                "content": {
                    "type": "string",
                    "description": "Content to save (if action is 'save')"
                }
            },
            "required": ["action"]
        })
    }

    fn work_scope(&self) -> Value {
        json!({
            "status": "constrained",
            "environment": "local filesystem (artifacts/ directory)",
            "persistence": "permanent",
            "data_types": ["text", "code", "json", "logs"]
        })
    }

    async fn execute(&self, params: Value) -> AgentResult<ToolOutput> {
        self.ensure_dir().await?;

        let action = params["action"]
            .as_str()
            .ok_or_else(|| AgentError::Validation("Missing required parameter: action".to_string()))?;

        match action {
            "save" => {
                let filename = params["name"]
                    .as_str()
                    .ok_or_else(|| AgentError::Validation("Missing required parameter: name".to_string()))?;
                let content = params["content"]
                    .as_str()
                    .ok_or_else(|| AgentError::Validation("Missing required parameter: content".to_string()))?;
                
                let path = self.resolve_path(filename)?;
                fs::write(&path, content).await
                    .map_err(|e| AgentError::Io(e))?;
                
                info!("Artifact written: {}", filename);
                Ok(ToolOutput::success(
                    json!({ "name": filename, "bytes": content.len() }),
                    format!("Successfully saved artifact: {}", filename)
                ))
            }
            "load" => {
                let filename = params["name"]
                    .as_str()
                    .ok_or_else(|| AgentError::Validation("Missing required parameter: name".to_string()))?;
                
                let path = self.resolve_path(filename)?;
                let content = fs::read_to_string(&path).await
                    .map_err(|e| AgentError::Io(e))?;
                
                Ok(ToolOutput::success(
                    json!({ "name": filename, "content": content }),
                    format!("Content of {}:\n\n{}", filename, content)
                ))
            }
            "list" => {
                let mut entries = fs::read_dir(&self.base_dir).await
                    .map_err(|e| AgentError::Io(e))?;
                let mut files = Vec::new();
                
                while let Some(entry) = entries.next_entry().await.map_err(|e| AgentError::Io(e))? {
                    if let Ok(meta) = entry.metadata().await {
                        if meta.is_file() {
                            files.push(entry.file_name().to_string_lossy().to_string());
                        }
                    }
                }
                
                let summary = if files.is_empty() {
                    "No artifacts found.".to_string()
                } else {
                    format!("Artifacts:\n- {}", files.join("\n- "))
                };
                
                Ok(ToolOutput::success(
                    json!({ "files": files }),
                    summary
                ))
            }
            "delete" => {
                let filename = params["name"]
                    .as_str()
                    .ok_or_else(|| AgentError::Validation("Missing required parameter: name".to_string()))?;
                
                let path = self.resolve_path(filename)?;
                fs::remove_file(&path).await
                    .map_err(|e| AgentError::Io(e))?;
                
                info!("Artifact deleted: {}", filename);
                Ok(ToolOutput::success(
                    json!({ "name": filename }),
                    format!("Successfully deleted artifact: {}", filename)
                ))
            }
            _ => Ok(ToolOutput::failure(format!("Unknown action: {}", action)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_artifact_write_read() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let tool = ArtifactTool::new(temp_dir.path());
        
        let filename = "test.txt";
        let content = "hello artifact";
        
        // Write
        tool.execute(json!({
            "action": "save",
            "name": filename,
            "content": content
        })).await.expect("Tool execution failed");
        
        // Read
        let res = tool.execute(json!({
            "action": "load",
            "name": filename
        })).await.expect("Tool execution failed");
        
        assert!(res.success);
        assert_eq!(res.data["content"].as_str().expect("No content in data"), content);
    }

    #[tokio::test]
    async fn test_artifact_list_delete() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let tool = ArtifactTool::new(temp_dir.path());
        
        tool.execute(json!({
            "action": "save",
            "name": "f1.txt",
            "content": "c1"
        })).await.expect("Tool execution failed");
        
        // List
        let res_list = tool.execute(json!({"action": "list"})).await.expect("Tool execution failed");
        assert!(res_list.data["files"].as_array().expect("No files in data").len() >= 1);
        
        // Delete
        tool.execute(json!({
            "action": "delete",
            "name": "f1.txt"
        })).await.expect("Tool execution failed");
        
        let res_list_after = tool.execute(json!({"action": "list"})).await.expect("Tool execution failed");
        assert_eq!(res_list_after.data["files"].as_array().expect("No files in data").len(), 0);
    }
}
