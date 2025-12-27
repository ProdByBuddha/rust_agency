//! Web Search Tool
//! 
//! Performs web searches using DuckDuckGo HTML API (no API key required).

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::{debug, warn};

use super::{Tool, ToolOutput};

/// Web search tool using DuckDuckGo
pub struct WebSearchTool {
    client: Client,
}

impl WebSearchTool {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
                .build()
                .unwrap_or_default(),
        }
    }

    async fn search_ddg(&self, query: &str, num_results: usize) -> Result<Vec<SearchResult>> {
        // Use DuckDuckGo HTML search (more reliable than API for simple uses)
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );

        debug!("Searching DuckDuckGo: {}", query);

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send search request")?;

        let html = response.text().await.context("Failed to read response")?;
        
        // Parse results from HTML (simple extraction)
        let results = self.parse_ddg_html(&html, num_results);
        
        Ok(results)
    }

    fn parse_ddg_html(&self, html: &str, max_results: usize) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        // Simple HTML parsing - look for result snippets
        // DuckDuckGo HTML uses class="result__snippet" for snippets
        // and class="result__a" for links
        
        let snippet_pattern = regex::Regex::new(r#"class="result__snippet"[^>]*>([^<]+)"#).ok();
        let title_pattern = regex::Regex::new(r#"class="result__a"[^>]*>([^<]+)"#).ok();
        let url_pattern = regex::Regex::new(r#"class="result__url"[^>]*>([^<]+)"#).ok();

        if let (Some(snippet_re), Some(title_re), Some(url_re)) = 
            (snippet_pattern, title_pattern, url_pattern) 
        {
            let snippets: Vec<_> = snippet_re.captures_iter(html).collect();
            let titles: Vec<_> = title_re.captures_iter(html).collect();
            let urls: Vec<_> = url_re.captures_iter(html).collect();

            let count = snippets.len().min(titles.len()).min(max_results);
            
            for i in 0..count {
                let title = titles.get(i)
                    .and_then(|c| c.get(1))
                    .map(|m| html_escape::decode_html_entities(m.as_str()).to_string())
                    .unwrap_or_default();
                
                let snippet = snippets.get(i)
                    .and_then(|c| c.get(1))
                    .map(|m| html_escape::decode_html_entities(m.as_str()).to_string())
                    .unwrap_or_default();
                
                let url = urls.get(i)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default();

                if !title.is_empty() && !snippet.is_empty() {
                    results.push(SearchResult { title, snippet, url });
                }
            }
        }

        // If HTML parsing failed, return a simple acknowledgment
        if results.is_empty() {
            warn!("Could not parse DuckDuckGo results, returning placeholder");
            results.push(SearchResult {
                title: "Search Completed".to_string(),
                snippet: format!("Search was performed but results could not be parsed. Query: {}", 
                    html.len()),
                url: String::new(),
            });
        }

        results
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct SearchResult {
    title: String,
    snippet: String,
    url: String,
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information. Use this when you need to find current information, \
         facts, or data that may not be in your training data."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to look up"
                },
                "num_results": {
                    "type": "integer",
                    "description": "Number of results to return (default: 5, max: 10)",
                    "default": 5
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let query = params["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;
        
        let num_results = params["num_results"]
            .as_u64()
            .unwrap_or(5)
            .min(10) as usize;

        match self.search_ddg(query, num_results).await {
            Ok(results) => {
                let formatted = results
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        format!(
                            "{}. **{}**\n   {}\n   URL: {}",
                            i + 1,
                            r.title,
                            r.snippet,
                            r.url
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n");

                let summary = format!(
                    "Found {} results for '{}'\n\n{}",
                    results.len(),
                    query,
                    formatted
                );

                Ok(ToolOutput::success(
                    json!({
                        "query": query,
                        "num_results": results.len(),
                        "results": results.iter().map(|r| json!({
                            "title": r.title,
                            "snippet": r.snippet,
                            "url": r.url
                        })).collect::<Vec<_>>()
                    }),
                    summary
                ))
            }
            Err(e) => {
                warn!("Web search failed: {}", e);
                Ok(ToolOutput::failure(format!("Search failed: {}", e)))
            }
        }
    }
}
