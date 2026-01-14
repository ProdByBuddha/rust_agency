//! Reflection Module
//! 
//! Self-reflection for error analysis and improvement.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;
use std::sync::Arc;
use ollama_rs::Ollama;

use super::{ReActStep, LLMProvider, OllamaProvider, OpenAICompatibleProvider};

/// Result of a reflection analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionResult {
    /// Analysis of what went wrong
    pub analysis: String,
    /// Suggested improvements
    pub suggestions: Vec<String>,
    /// Whether retry is recommended
    pub should_retry: bool,
    /// A refined approach if retry is recommended
    pub refined_approach: Option<String>,
}

/// Reflector for analyzing agent failures and improving performance
#[derive(Clone)]
pub struct Reflector {
    provider: Arc<dyn LLMProvider>,
    model: String,
}

impl Reflector {
    pub fn new(ollama: Ollama) -> Self {
        Self {
            provider: Arc::new(OllamaProvider::new(ollama)),
            model: "deepseek-r1:8b".to_string(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_provider(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.provider = provider;
        self
    }

    pub fn with_provider_url(mut self, url: Option<String>) -> Self {
        if let Some(url_str) = url {
            self.provider = Arc::new(OpenAICompatibleProvider::new(url_str, None));
        }
        self
    }

    /// Analyze a failed execution and provide feedback
    pub async fn analyze_failure(
        &self,
        query: &str,
        steps: &[ReActStep],
        error: Option<&str>,
    ) -> Result<ReflectionResult> {
        let steps_text = steps
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mut step_text = format!("Step {}:\n  Thought: {}", i + 1, s.thought);
                for action in &s.actions {
                    step_text.push_str(&format!("\n  Action: {} with params {:?}", action.name, action.parameters));
                }
                for obs in &s.observations {
                    step_text.push_str(&format!("\n  Observation: {}", obs));
                }
                step_text
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            r#"You are analyzing a failed AI agent execution to help improve it.

## Original Query
{}

## Execution Steps
{}

## Error (if any)
{}

## Analysis Task
Analyze what went wrong and how to improve. Provide:
1. A brief analysis of the failure
2. 2-3 specific suggestions for improvement
3. Whether a retry with a different approach could succeed
4. If retry is recommended, describe the refined approach

Format your response as:
ANALYSIS: [Your analysis]
SUGGESTIONS:
- [Suggestion 1]
- [Suggestion 2]
SHOULD_RETRY: [yes/no]
REFINED_APPROACH: [If yes, describe the new approach]
"#,
            query,
            steps_text,
            error.unwrap_or("Max iterations reached without final answer")
        );

        debug!("Reflection prompt:\n{}", prompt);

        let content = self.provider.generate(&self.model, prompt, None).await?;

        self.parse_reflection(&content)
    }

    fn parse_reflection(&self, response: &str) -> Result<ReflectionResult> {
        let response_upper = response.to_uppercase();
        
        // Extract ANALYSIS
        let analysis = if let Some(analysis_idx) = response_upper.find("ANALYSIS:") {
            let start = analysis_idx + "ANALYSIS:".len();
            let end = response_upper[start..].find("SUGGESTIONS:")
                .or_else(|| response_upper[start..].find("SHOULD_RETRY:"))
                .map(|i| start + i)
                .unwrap_or(response.len());
            response[start..end].trim().to_string()
        } else {
            "Unable to analyze failure".to_string()
        };

        // Extract SUGGESTIONS
        let suggestions = if let Some(sugg_idx) = response_upper.find("SUGGESTIONS:") {
            let start = sugg_idx + "SUGGESTIONS:".len();
            let end = response_upper[start..].find("SHOULD_RETRY:")
                .or_else(|| response_upper[start..].find("REFINED_APPROACH:"))
                .map(|i| start + i)
                .unwrap_or(response.len());
            response[start..end]
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim().trim_start_matches('-').trim();
                    if trimmed.is_empty() || trimmed == "SUGGESTIONS:" {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        // Extract SHOULD_RETRY
        let should_retry = if let Some(retry_idx) = response_upper.find("SHOULD_RETRY:") {
            let start = retry_idx + "SHOULD_RETRY:".len();
            let text = &response[start..].trim();
            text.to_lowercase().starts_with("yes")
        } else {
            false
        };

        // Extract REFINED_APPROACH
        let refined_approach = if let Some(approach_idx) = response_upper.find("REFINED_APPROACH:") {
            let start = approach_idx + "REFINED_APPROACH:".len();
            let text = response[start..].trim().to_string();
            if text.is_empty() { None } else { Some(text) }
        } else {
            None
        };

        Ok(ReflectionResult {
            analysis,
            suggestions,
            should_retry,
            refined_approach,
        })
    }

    /// Review a successful response for hallucinations or missing tool use
    pub async fn review_response(
        &self,
        query: &str,
        answer: &str,
        steps: &[ReActStep],
    ) -> Result<ReflectionResult> {
        let steps_text = steps
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mut step_text = format!("Step {}:\n  Thought: {}", i + 1, s.thought);
                for action in &s.actions {
                    step_text.push_str(&format!("\n  Action: {} with params {:?}", action.name, action.parameters));
                }
                for obs in &s.observations {
                    step_text.push_str(&format!("\n  Observation: {}", obs));
                }
                step_text
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            r#"You are a technical reviewer. Verify the accuracy of the following AI response.

## Original Query
{}

## Assistant Trace
{}

## Final Answer
{}

## Review Task
Check if the answer is grounded in the observations.
CRITICAL RULES:
1. If the agent claims to have DONE something (created a file, forged a tool, ran code), the trace MUST show the corresponding [ACTION] and a successful [OBSERVATION].
2. If the agent says "I have created X" but there is no tool call to create X in the trace, it is a HALLUCINATION.
3. If the query is about the CODEBASE or FILES, the assistant MUST have used 'codebase_explorer' or 'memory_query'. 
4. Be pedantic. Do not accept "I will do X" or "X is ready" if the tools weren't actually called.

Format your response as:
ANALYSIS: [Is the answer verified by observations? Did it actually execute the required tools?]
SUGGESTIONS:
- [Suggestion 1]
SHOULD_RETRY: [yes/no - yes if it hallucinated success, skipped tools, or gave a lazy summary]
REFINED_APPROACH: [If yes, how to fix it. e.g., "You must actually call forge_tool with the code."]
"#,
            query,
            steps_text,
            answer
        );

        debug!("Review prompt:\n{}", prompt);

        let content = self.provider.generate(&self.model, prompt, None).await?;

        self.parse_reflection(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::OllamaProvider;

    #[test]
    fn test_parse_reflection_retry() {
        let reflector = Reflector::new(Ollama::default());
        
        let response = r#"ANALYSIS: The agent tried to use a tool that doesn't exist.
SUGGESTIONS:
- Check available tools before attempting to use them
- Use web_search instead of google_search
SHOULD_RETRY: yes
REFINED_APPROACH: Use the web_search tool with a more specific query."#;

        let result = reflector.parse_reflection(response).unwrap();
        assert_eq!(result.analysis, "The agent tried to use a tool that doesn't exist.");
        assert_eq!(result.suggestions.len(), 2);
        assert!(result.should_retry);
        assert_eq!(result.refined_approach.unwrap(), "Use the web_search tool with a more specific query.");
    }

    #[test]
    fn test_parse_reflection_no_retry() {
        let reflector = Reflector::new(Ollama::default());
        
        let response = r#"ANALYSIS: Everything looks good.
SUGGESTIONS:
None
SHOULD_RETRY: no"#;

        let result = reflector.parse_reflection(response).unwrap();
        assert_eq!(result.analysis, "Everything looks good.");
        assert!(result.suggestions.is_empty() || result.suggestions[0] == "None");
        assert!(!result.should_retry);
        assert!(result.refined_approach.is_none());
    }

    #[test]
    fn test_parse_reflection_malformed() {
        let reflector = Reflector::new(Ollama::default());
        let result = reflector.parse_reflection("Garbage text").unwrap();
        assert_eq!(result.analysis, "Unable to analyze failure");
        assert!(!result.should_retry);
    }
}
