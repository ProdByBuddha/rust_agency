//! Artifact Management Tool
//! 
//! Allows agents to manage persistent files (artifacts) in a dedicated workspace.
//! This is useful for saving code, documentation, or search results.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

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
    async fn ensure_dir(&self) -> Result<()> {
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir).await
                .context("Failed to create artifacts directory")?;
        }
        Ok(())
    }

    /// Resolve a path relative to the base directory and ensure it stays within bounds
    fn resolve_path(&self, filename: &str) -> Result<PathBuf> {
        let path = self.base_dir.join(filename);
        
        // Security check: ensure path is within base_dir
        if !path.starts_with(&self.base_dir) {
            return Err(anyhow::anyhow!("Access denied: Path is outside the artifacts directory"));
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

    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        self.ensure_dir().await?;

        let action = params["action"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: action"))?;

        match action {
            "save" => {
                let filename = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
                let content = params["content"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;
                
                let path = self.resolve_path(filename)?;
                fs::write(&path, content).await
                    .context(format!("Failed to write artifact: {}", filename))?;
                
                info!("Artifact written: {}", filename);
                Ok(ToolOutput::success(
                    json!({ "name": filename, "bytes": content.len() }),
                    format!("Successfully saved artifact: {}", filename)
                ))
            }
            "load" => {
                let filename = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
                
                let path = self.resolve_path(filename)?;
                let content = fs::read_to_string(&path).await
                    .context(format!("Failed to read artifact: {}", filename))?;
                
                Ok(ToolOutput::success(
                    json!({ "name": filename, "content": content }),
                    format!("Content of {}:\n\n{}", filename, content)
                ))
            }
            "list" => {
                let mut entries = fs::read_dir(&self.base_dir).await?;
                let mut files = Vec::new();
                
                while let Some(entry) = entries.next_entry().await? {
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
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
                
                let path = self.resolve_path(filename)?;
                fs::remove_file(&path).await
                    .context(format!("Failed to delete artifact: {}", filename))?;
                
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
        let temp_dir = tempdir().unwrap();
        let tool = ArtifactTool::new(temp_dir.path());
        
        let filename = "test.txt";
        let content = "hello artifact";
        
        // Write
        tool.execute(json!({
            "action": "save",
            "name": filename,
            "content": content
        })).await.unwrap();
        
        // Read
        let res = tool.execute(json!({
            "action": "load",
            "name": filename
        })).await.unwrap();
        
        assert!(res.success);
        assert_eq!(res.data["content"].as_str().unwrap(), content);
    }

    #[tokio::test]
    async fn test_artifact_list_delete() {
        let temp_dir = tempdir().unwrap();
        let tool = ArtifactTool::new(temp_dir.path());
        
        tool.execute(json!({
            "action": "save",
            "name": "f1.txt",
            "content": "c1"
        })).await.unwrap();
        
        // List
        let res_list = tool.execute(json!({"action": "list"})).await.unwrap();
        assert!(res_list.data["files"].as_array().unwrap().len() >= 1);
        
        // Delete
        tool.execute(json!({
            "action": "delete",
            "name": "f1.txt"
        })).await.unwrap();
        
        let res_list_after = tool.execute(json!({"action": "list"})).await.unwrap();
        assert_eq!(res_list_after.data["files"].as_array().unwrap().len(), 0);
    }
}
