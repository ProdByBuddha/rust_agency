//! Codebase Indexer Tool
//! 
//! Allows agents to search and read the project's own source code.
//! This helps agents understand their own capabilities and tool definitions.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::fs;

use super::{Tool, ToolOutput};

/// Tool for exploring the agency's own codebase
pub struct CodebaseTool {
    src_dir: PathBuf,
}

impl CodebaseTool {
    pub fn new(src_dir: impl Into<PathBuf>) -> Self {
        let path = src_dir.into();
        // Try to get absolute path if possible for better safety checks
        let src_dir = std::fs::canonicalize(&path).unwrap_or(path);
        Self { src_dir }
    }

    fn is_safe_path(&self, path: &Path) -> bool {
        // Canonicalize the input path to resolve ".." and symlinks
        let canonical = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => return false, // If it doesn't exist or can't be resolved, it's not safe to read
        };

        let path_str = canonical.to_string_lossy();
        
        canonical.starts_with(&self.src_dir) || 
        path_str.contains("rust_agency/src") ||
        path_str.ends_with("Cargo.toml") ||
        path_str.ends_with("Cargo.lock") ||
        path_str.ends_with(".gitignore")
    }
}

impl Default for CodebaseTool {
    fn default() -> Self {
        Self::new("src")
    }
}

#[async_trait]
impl Tool for CodebaseTool {
    fn name(&self) -> String {
        "codebase_explorer".to_string()
    }

    fn description(&self) -> String {
        "Explore and analyze the current project's codebase. \n        Supports 'list_files', 'read_file', and 'search' operations.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list_files", "read_file", "search"],
                    "description": "The action to perform"
                },
                "path": {
                    "type": "string",
                    "description": "File path (if action is 'read_file')"
                },
                "query": {
                    "type": "string",
                    "description": "Search query (if action is 'search')"
                }
            },
            "required": ["action"]
        })
    }

    fn work_scope(&self) -> Value {
        json!({
            "status": "constrained",
            "environment": "local project root",
            "access": "read-only",
            "data_scope": "source code and configuration"
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let action = params["action"].as_str().unwrap_or("list_files");

        match action {
            "list_files" => {
                let mut files = Vec::new();
                let mut dirs = vec![self.src_dir.clone()];
                
                while let Some(dir) = dirs.pop() {
                    let mut entries = match fs::read_dir(dir).await {
                        Ok(e) => e,
                        Err(e) => return Ok(ToolOutput::failure(format!("Failed to read directory: {}", e))),
                    };
                    while let Some(entry) = entries.next_entry().await? {
                        let path = entry.path();
                        if path.is_dir() {
                            let path_str = path.to_string_lossy();
                            if !path_str.contains("target") && !path_str.contains(".git") {
                                dirs.push(path);
                            }
                        } else {
                            files.push(path.to_string_lossy().to_string());
                        }
                    }
                }
                
                let mut tree_summary = String::from("Codebase Files:\n");
                for f in &files {
                    tree_summary.push_str(&format!("- {}\n", f));
                }
                
                Ok(ToolOutput::success(json!({ "files": files }), tree_summary))
            },
            "read_file" => {
                let rel_path = params["path"].as_str().context("Missing path")?;
                
                // Construct the path carefully
                let path = self.src_dir.join(rel_path);
                
                // SAFETY CHECK FIRST
                if !self.is_safe_path(&path) {
                    return Ok(ToolOutput::failure("Access denied: Path outside allowed areas"));
                }

                // Check existence after safety check
                if !path.exists() {
                    return Ok(ToolOutput::failure(format!("File not found: {}", rel_path)));
                }

                let content = match fs::read_to_string(&path).await {
                    Ok(c) => c,
                    Err(e) => return Ok(ToolOutput::failure(format!("Failed to read {}: {}", rel_path, e))),
                };
                
                Ok(ToolOutput::success(
                    json!({ "path": rel_path, "content": content }),
                    format!("Content of {}:\n\n{}", rel_path, content)
                ))
            },
            _ => Ok(ToolOutput::failure("Unsupported codebase action"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_codebase_list_files() {
        let dir = tempdir().unwrap();
        let src_path = dir.path().join("src");
        fs::create_dir(&src_path).await.unwrap();
        
        let file_path = src_path.join("lib.rs");
        let mut file = File::create(file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();
        
        // We need real absolute paths for canonicalize to work in tests
        let tool = CodebaseTool::new(src_path);
        let res = tool.execute(json!({"action": "list_files"})).await.unwrap();
        
        assert!(res.success);
        let files = res.data["files"].as_array().unwrap();
        assert!(files.iter().any(|f| f.as_str().unwrap().contains("lib.rs")));
    }

    #[tokio::test]
    async fn test_codebase_read_file() {
        let dir = tempdir().unwrap();
        let src_path = dir.path().join("src");
        fs::create_dir(&src_path).await.unwrap();
        
        let file_path = src_path.join("lib.rs");
        let mut file = File::create(&file_path).unwrap();
        let content = "pub fn hello() {}";
        writeln!(file, "{}", content).unwrap();
        
        let tool = CodebaseTool::new(&src_path);
        let res = tool.execute(json!({
            "action": "read_file",
            "path": "lib.rs"
        })).await.unwrap();
        
        assert!(res.success);
        assert!(res.data["content"].as_str().unwrap().contains(content));
    }

    #[tokio::test]
    async fn test_codebase_safety() {
        let dir = tempdir().unwrap();
        let src_path = dir.path().join("src");
        fs::create_dir(&src_path).await.unwrap();
        
        let tool = CodebaseTool::new(&src_path);
        
        // Attempt to read something outside using path traversal
        let res = tool.execute(json!({
            "action": "read_file",
            "path": "../secret.txt"
        })).await.unwrap();
        
        assert!(!res.success);
        assert!(res.summary.contains("Access denied"));
    }
}
