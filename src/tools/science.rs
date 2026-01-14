//! Science Tool - Interfaces with Sciencepedia knowledge base (Remote)
//! 
//! Provides tools to browse and query the hierarchical scientific encyclopedia
//! hosted at https://github.com/deepmodeling/sciencepedia

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::debug;

use crate::agent::{AgentResult, AgentError};
use super::{Tool, ToolOutput};

#[derive(Debug, Deserialize)]
struct GithubContent {
    name: String,
    path: String,
    #[serde(rename = "type")]
    content_type: String,
    #[allow(dead_code)]
    download_url: Option<String>,
}

/// Tool for accessing Sciencepedia content remotely
pub struct ScienceTool {
    client: Client,
    api_base: String,
    raw_base: String,
}

impl ScienceTool {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("rust_agency/0.2.0")
                .build()
                .unwrap_or_default(),
            api_base: "https://api.github.com/repos/deepmodeling/sciencepedia/contents".to_string(),
            raw_base: "https://raw.githubusercontent.com/deepmodeling/sciencepedia/master".to_string(),
        }
    }

    async fn fetch_github_dir(&self, path: &str) -> AgentResult<Vec<GithubContent>> {
        let url = if path.is_empty() {
            self.api_base.clone()
        } else {
            format!("{}/{}", self.api_base, path)
        };

        debug!("Fetching GitHub directory: {}", url);
        let response = self.client.get(&url).send().await
            .map_err(|e| AgentError::Tool(format!("GitHub request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AgentError::Tool(format!("GitHub API error: {}", response.status())));
        }

        let contents: Vec<GithubContent> = response.json().await
            .map_err(|e| AgentError::Tool(format!("Failed to parse GitHub response: {}", e)))?;
        Ok(contents)
    }

    async fn fetch_raw_content(&self, path: &str) -> AgentResult<String> {
        let url = format!("{}/{}", self.raw_base, path);
        debug!("Fetching raw content: {}", url);
        
        let response = self.client.get(&url).send().await
            .map_err(|e| AgentError::Tool(format!("GitHub request failed: {}", e)))?;
        if !response.status().is_success() {
            return Err(AgentError::Tool(format!("Failed to fetch raw content: {}", response.status())));
        }

        Ok(response.text().await.map_err(|e| AgentError::Tool(e.to_string()))?)
    }
}

impl Default for ScienceTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ScienceTool {
    fn name(&self) -> String {
        "science_query".to_string()
    }

    fn description(&self) -> String {
        "Browse and read Sciencepedia, a structured scientific encyclopedia hosted on GitHub. Supports 'list_categories', 'explore_subject', and 'read_article'.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list_categories", "explore_subject", "read_article"],
                    "description": "The action to perform"
                },
                "path": {
                    "type": "string",
                    "description": "Relative path within sciencepedia (e.g., 'Applied Mathematics@@619990/undergraduate@@619991')"
                }
            },
            "required": ["action"]
        })
    }

    fn work_scope(&self) -> Value {
        json!({
            "status": "constrained",
            "environment": "external GitHub repository",
            "network": "required",
            "access": "read-only",
            "data_scope": "scientific knowledge base"
        })
    }

    async fn execute(&self, params: Value) -> AgentResult<ToolOutput> {
        let action = params["action"].as_str().unwrap_or("list_categories");

        match action {
            "list_categories" => {
                match self.fetch_github_dir("").await {
                    Ok(contents) => {
                        let categories: Vec<String> = contents
                            .into_iter()
                            .filter(|c| c.content_type == "dir" && !c.name.starts_with('.'))
                            .map(|c| c.name)
                            .collect();
                        
                        let summary = format!("Sciencepedia Categories:\n{}", 
                            categories.iter().map(|c| format!("- {}", c)).collect::<Vec<_>>().join("\n"));
                        
                        Ok(ToolOutput::success(json!({ "categories": categories }), summary))
                    },
                    Err(e) => Ok(ToolOutput::failure(format!("Failed to list categories from GitHub: {}", e)))
                }
            },
            "explore_subject" => {
                let rel_path = params["path"].as_str().unwrap_or("");
                match self.fetch_github_dir(rel_path).await {
                    Ok(contents) => {
                        let mut sub_items = Vec::new();
                        let mut files = Vec::new();

                        for item in contents {
                            if item.name.starts_with('.') { continue; }
                            if item.content_type == "dir" {
                                sub_items.push(item.name);
                            } else {
                                files.push(item.name);
                            }
                        }

                        let summary = format!(
                            "Contents of {}:\n\nSub-categories:\n{}\n\nFiles:\n{}",
                            rel_path,
                            sub_items.iter().map(|s| format!("- [DIR] {}", s)).collect::<Vec<_>>().join("\n"),
                            files.iter().map(|f| format!("- [FILE] {}", f)).collect::<Vec<_>>().join("\n")
                        );

                        Ok(ToolOutput::success(json!({ "directories": sub_items, "files": files }), summary))
                    },
                    Err(e) => Ok(ToolOutput::failure(format!("Failed to explore subject '{}' on GitHub: {}", rel_path, e)))
                }
            },
            "read_article" => {
                let rel_path = params["path"].as_str().ok_or_else(|| AgentError::Validation("Missing path".to_string()))?;
                
                // Try MainContent.md first
                let main_path = format!("{}/MainContent.md", rel_path);
                match self.fetch_raw_content(&main_path).await {
                    Ok(content) => {
                        let summary = format!("Article Content from {}:\n\n{}", rel_path, content);
                        Ok(ToolOutput::success(json!({ "path": rel_path, "content": content }), summary))
                    },
                    Err(_) => {
                        // Fallback to Applications.md
                        let app_path = format!("{}/Applications.md", rel_path);
                        match self.fetch_raw_content(&app_path).await {
                            Ok(content) => {
                                let summary = format!("Article Content (Applications) from {}:\n\n{}", rel_path, content);
                                Ok(ToolOutput::success(json!({ "path": rel_path, "content": content }), summary))
                            },
                            Err(e) => Ok(ToolOutput::failure(format!("No article content found in {} on GitHub: {}", rel_path, e)))
                        }
                    }
                }
            },
            _ => Ok(ToolOutput::failure("Unsupported science action"))
        }
    }
}