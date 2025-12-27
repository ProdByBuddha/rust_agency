//! Router - Query routing to appropriate agents
//! 
//! Determines which agent should handle a given query.

use anyhow::Result;
use ollama_rs::Ollama;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::agent::{AgentType, LLMProvider, OllamaProvider, OpenAICompatibleProvider};

/// Routing decision for a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// The agent type to route to
    pub agent_type: AgentType,
    /// Whether to search memory for context
    pub should_search_memory: bool,
    /// Confidence in the routing decision (0.0 - 1.0)
    pub confidence: f32,
    /// Reason for the routing decision
    pub reason: String,
}

/// Router for directing queries to appropriate agents
#[derive(Clone)]
pub struct Router {
    provider: Arc<dyn LLMProvider>,
    model: String,
}

impl Router {
    pub fn new(ollama: Ollama) -> Self {
        Self {
            provider: Arc::new(OllamaProvider::new(ollama)),
            model: "llama3.2:3b".to_string(),
        }
    }

    pub fn with_provider(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.provider = provider;
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_provider_url(mut self, url: Option<String>) -> Self {
        if let Some(url_str) = url {
            self.provider = Arc::new(OpenAICompatibleProvider::new(url_str, None));
        }
        self
    }

    /// Route a query to the appropriate agent
    pub async fn route(&self, query: &str) -> Result<RoutingDecision> {
        // Quick heuristics for simple cases
        let q_lower = query.to_lowercase();
        
        // Very short, greeting, or identity messages -> GeneralChat
        if q_lower.len() < 10 || self.is_greeting(&q_lower) || self.is_identity_query(&q_lower) {
            return Ok(RoutingDecision {
                agent_type: AgentType::GeneralChat,
                should_search_memory: false,
                confidence: 0.9,
                reason: "Simple greeting, short message, or identity query".to_string(),
            });
        }

        // Filesystem / Directory heuristics (Fast-Path)
        if self.is_filesystem_related(&q_lower) {
            return Ok(RoutingDecision {
                agent_type: AgentType::Coder,
                should_search_memory: false,
                confidence: 0.95,
                reason: "Direct filesystem query (heuristics fast-path)".to_string(),
            });
        }

        // Knowledge Graph / Relationship heuristics
        if q_lower.contains("graph") || q_lower.contains("relationship") || q_lower.contains("visualize") {
            return Ok(RoutingDecision {
                agent_type: AgentType::Reasoner,
                should_search_memory: true,
                confidence: 0.9,
                reason: "Knowledge graph or relationship query".to_string(),
            });
        }

        // Code-related keywords -> Coder
        if self.is_code_related(&q_lower) && !self.is_complex_query(&q_lower) {
            return Ok(RoutingDecision {
                agent_type: AgentType::Coder,
                should_search_memory: false,
                confidence: 0.85,
                reason: "Query contains code-related keywords".to_string(),
            });
        }

        // Planning keywords -> Planner
        if self.is_planning_related(&q_lower) || self.is_complex_query(&q_lower) {
            return Ok(RoutingDecision {
                agent_type: AgentType::Planner,
                should_search_memory: true,
                confidence: 0.8,
                reason: "Query involves planning or task decomposition".to_string(),
            });
        }

        // Research/search keywords -> Researcher
        if self.is_research_related(&q_lower) {
            return Ok(RoutingDecision {
                agent_type: AgentType::Researcher,
                should_search_memory: true,
                confidence: 0.8,
                reason: "Query requires information gathering".to_string(),
            });
        }

        // Use LLM for complex routing decisions
        self.llm_route(query).await
    }

    fn is_greeting(&self, query: &str) -> bool {
        let greetings = ["hi", "hello", "hey", "howdy", "greetings", "good morning", "good afternoon", "good evening"];
        greetings.iter().any(|g| query.starts_with(g) || query == *g)
    }

    fn is_identity_query(&self, query: &str) -> bool {
        let keywords = ["who are you", "what is your name", "what are you", "your identity", "your name"];
        keywords.iter().any(|k| query.contains(k))
    }

    fn is_filesystem_related(&self, query: &str) -> bool {
        let keywords = [
            "list", "folder", "directory", "file", "ls", "dir", "tree", "structure",
            "show files", "show folders", "what is in", "contents of", "read "
        ];
        keywords.iter().any(|k| query.contains(k))
    }

    fn is_code_related(&self, query: &str) -> bool {
        let keywords = [
            "code", "function", "program", "script", "bug", "error", "compile",
            "debug", "implement", "algorithm", "class", "method", "variable",
            "rust", "python", "javascript", "typescript", "java", "c++", "golang",
            "write a", "create a", "fix the", "refactor"
        ];
        keywords.iter().any(|k| query.contains(k))
    }

    fn is_planning_related(&self, query: &str) -> bool {
        let keywords = [
            "plan", "schedule", "steps", "how to", "break down", "organize",
            "roadmap", "workflow", "process", "strategy", "goal", "milestone"
        ];
        keywords.iter().any(|k| query.contains(k))
    }

    fn is_research_related(&self, query: &str) -> bool {
        let keywords = [
            "search", "find", "look up", "research", "what is", "who is",
            "when did", "where is", "why does", "how does", "latest", "current",
            "news", "information about", "tell me about"
        ];
        keywords.iter().any(|k| query.contains(k))
    }

    fn is_complex_query(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        q.contains(" and ") || q.contains(" then ") || q.contains(", then ") || q.contains(" and finally ")
    }

    async fn llm_route(&self, query: &str) -> Result<RoutingDecision> {
        let prompt = format!(
            r#"q → classify(["general_chat", "reasoner", "coder", "researcher", "planner"]) → agent
q → needs_memory? → memory
→ {{agent, memory, reason: why?}}

q = "{}"
"#,
            query
        );

        let system = Some(super::sns::get_sns_system_prompt());
        let content = self.provider.generate(&self.model, prompt, system).await?;

        self.parse_routing_response(&content)
    }

    fn parse_routing_response(&self, response: &str) -> Result<RoutingDecision> {
        // Try parsing as JSON-like structure first (SNS output)
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_str = &response[start..=end];
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                    let agent_str = v["agent"].as_str()
                        .or_else(|| v["AGENT"].as_str())
                        .map(|s| s.to_lowercase())
                        .unwrap_or_else(|| "reasoner".to_string());

                    let agent_type = match agent_str.as_str() {
                        "general_chat" | "generalchat" | "chat" => AgentType::GeneralChat,
                        "coder" | "programmer" | "developer" => AgentType::Coder,
                        "researcher" | "research" => AgentType::Researcher,
                        "planner" | "planning" => AgentType::Planner,
                        _ => AgentType::Reasoner,
                    };

                    let memory_val = v["memory"].as_str()
                        .or_else(|| v["MEMORY"].as_str())
                        .map(|s| s.to_lowercase())
                        .unwrap_or_else(|| "no".to_string());
                    
                    let should_search_memory = memory_val == "yes" || memory_val == "true";

                    let reason = v["reason"].as_str()
                        .or_else(|| v["REASON"].as_str())
                        .unwrap_or("LLM routing decision")
                        .to_string();

                    return Ok(RoutingDecision {
                        agent_type,
                        should_search_memory,
                        confidence: 0.7,
                        reason,
                    });
                }
            }
        }

        // Fallback to previous Regex parsing
        let agent_re = Regex::new(r"(?i)AGENT:\s*(\w+)")?;
        let memory_re = Regex::new(r"(?i)MEMORY:\s*(yes|no)")?;
        let reason_re = Regex::new(r"(?i)REASON:\s*(.+)")?;

        let agent_str = agent_re
            .captures(response)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_lowercase())
            .unwrap_or_else(|| "reasoner".to_string());

        let agent_type = match agent_str.as_str() {
            "general_chat" | "generalchat" | "chat" => AgentType::GeneralChat,
            "coder" | "programmer" | "developer" => AgentType::Coder,
            "researcher" | "research" => AgentType::Researcher,
            "planner" | "planning" => AgentType::Planner,
            _ => AgentType::Reasoner,
        };

        let should_search_memory = memory_re
            .captures(response)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_lowercase() == "yes")
            .unwrap_or(false);

        let reason = reason_re
            .captures(response)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| "LLM routing decision".to_string());

        Ok(RoutingDecision {
            agent_type,
            should_search_memory,
            confidence: 0.7, // LLM routing is less certain
            reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greeting_detection() {
        let router = Router::new(Ollama::default());
        assert!(router.is_greeting("hi"));
        assert!(router.is_greeting("hello there"));
        assert!(!router.is_greeting("explain how"));
    }

    #[test]
    fn test_code_detection() {
        let router = Router::new(Ollama::default());
        assert!(router.is_code_related("write a python function"));
        assert!(router.is_code_related("debug this rust code"));
        assert!(!router.is_code_related("what is the weather"));
    }
}