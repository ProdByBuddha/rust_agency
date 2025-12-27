//! ReAct Agent Implementation
//! 
//! Implements the Reasoning + Acting framework for intelligent agent behavior.

use anyhow::Result;
use async_trait::async_trait;
use ollama_rs::Ollama;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::{Agent, AgentConfig, AgentType, is_action_query, truncate, LLMProvider, OllamaProvider, OpenAICompatibleProvider};
use crate::memory::Memory;
use crate::tools::{ToolCall, ToolRegistry};

/// A single step in the ReAct loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReActStep {
    /// The agent's thought/reasoning
    pub thought: String,
    /// The actions to take (tool calls)
    pub actions: Vec<ToolCall>,
    /// The observations from the actions
    pub observations: Vec<String>,
    /// Whether this is the final answer
    pub is_final: bool,
    /// The final answer (if is_final is true)
    pub answer: Option<String>,
}

impl ReActStep {
    pub fn thought(thought: impl Into<String>) -> Self {
        Self {
            thought: thought.into(),
            actions: Vec::new(),
            observations: Vec::new(),
            is_final: false,
            answer: None,
        }
    }

    pub fn with_action(mut self, action: ToolCall) -> Self {
        self.actions.push(action);
        self
    }

    pub fn with_actions(mut self, actions: Vec<ToolCall>) -> Self {
        self.actions = actions;
        self
    }

    pub fn final_answer(thought: impl Into<String>, answer: impl Into<String>) -> Self {
        Self {
            thought: thought.into(),
            actions: Vec::new(),
            observations: Vec::new(),
            is_final: true,
            answer: Some(answer.into()),
        }
    }
}

/// Response from an agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// The final answer
    pub answer: String,
    /// All steps taken to reach the answer
    pub steps: Vec<ReActStep>,
    /// The agent type that generated this response
    pub agent_type: AgentType,
    /// Whether the response was successful
    pub success: bool,
    /// Any error message
    pub error: Option<String>,
}

impl AgentResponse {
    pub fn success(answer: impl Into<String>, steps: Vec<ReActStep>, agent_type: AgentType) -> Self {
        Self {
            answer: answer.into(),
            steps,
            agent_type,
            success: true,
            error: None,
        }
    }

    pub fn failure(error: impl Into<String>, steps: Vec<ReActStep>, agent_type: AgentType) -> Self {
        let error = error.into();
        Self {
            answer: format!("I encountered an error: {}", error),
            steps,
            agent_type,
            success: false,
            error: Some(error),
        }
    }
}

/// ReAct Agent with reasoning and tool use capabilities
#[derive(Clone)]
pub struct ReActAgent {
    provider: Arc<dyn LLMProvider>,
    config: AgentConfig,
    tools: Arc<ToolRegistry>,
    memory: Option<Arc<dyn Memory>>,
}

impl ReActAgent {
    pub fn new(
        ollama: Ollama,
        config: AgentConfig,
        tools: Arc<ToolRegistry>,
    ) -> Self {
        let provider = if let Some(ref url) = config.provider_url {
            Arc::new(OpenAICompatibleProvider::new(url.clone(), None)) as Arc<dyn LLMProvider>
        } else {
            Arc::new(OllamaProvider::new(ollama)) as Arc<dyn LLMProvider>
        };

        Self {
            provider,
            config,
            tools,
            memory: None,
        }
    }

    pub fn with_provider(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.provider = provider;
        self
    }

    pub fn with_memory(mut self, memory: Arc<dyn Memory>) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Build the ReAct prompt
    async fn build_react_prompt(&self, query: &str, steps: &[ReActStep], context: Option<&str>) -> String {
        let mut prompt = format!(
            "{}\n\n",
            self.config.system_prompt
        );

        if let Some(ctx) = context {
            prompt.push_str(&format!("## Context\n{}\n\n", ctx));
        }

        prompt.push_str("## Available Tools\n");
        prompt.push_str(&self.tools.generate_filtered_tools_prompt(&self.config.allowed_tools).await);
        prompt.push_str("\n");

        prompt.push_str(r###"## Response Format

Strictly follow this format:

[THOUGHT]
Your reasoning.

[ACTION]
{{"name": "tool", "parameters": {{...}}}}
(Provide multiple [ACTION] for parallel execution.)

[ANSWER]
Your final response.

RULES:
1. No [ANSWER] if using [ACTION].
2. NEVER generate [OBSERVATION].
3. For codebase queries, use tools first.
4. Parallel execution is encouraged.

"###);

        prompt.push_str(&format!("## User Query\n{}\n\n", query));

        if !steps.is_empty() {
            prompt.push_str("## Trace\n");
            for step in steps {
                prompt.push_str(&format!("[THOUGHT]\n{}\n", step.thought));
                for action in &step.actions {
                    let action_json = serde_json::to_string(action).unwrap_or_default();
                    prompt.push_str(&format!("[ACTION]\n{}\n", action_json));
                }
                for obs in &step.observations {
                    prompt.push_str(&format!("[OBSERVATION]\n{}\n", obs));
                }
                prompt.push_str("\n");
            }
        }

        prompt.push_str("Continue:\n");
        prompt
    }

    /// Parse the LLM response using strict Tags
    fn parse_response(&self, response: &str, query: &str) -> Result<ReActStep> {
        debug!("Raw LLM Response for parsing:\n{}", response);

        let thought = self.extract_tag(response, "[THOUGHT]")
            .unwrap_or_else(|| "Thinking...".to_string());

        // 1. Check for Actions (even if Answer is present, Actions take priority to prevent laziness)
        let actions = self.extract_all_tags(response, "[ACTION]");
        let mut tool_calls = Vec::new();
        
        if !actions.is_empty() {
            for action_str in actions {
                if let Some(call) = self.parse_json_tool_call(&action_str) {
                    tool_calls.push(call);
                }
            }
        } else {
            // FALLBACK: Try to find JSON even without the tag if it looks like a tool call is intended
            if let Some(call) = self.parse_json_tool_call(response) {
                debug!("Fallback: Found valid JSON tool call without [ACTION] tag");
                tool_calls.push(call);
            }
        }

        if !tool_calls.is_empty() {
            return Ok(ReActStep::thought(thought).with_actions(tool_calls));
        }

        // 2. Check for Answer
        if let Some(answer) = self.extract_tag(response, "[ANSWER]") {
            return Ok(ReActStep::final_answer(thought, answer));
        }

        // 3. Robustness Fallback: If no tags were found, and it's NOT an action query (e.g. "Hello"), 
        // treat the whole response as the answer.
        if !is_action_query(query) && !response.is_empty() {
            warn!("No tags found in conversational response. Falling back to treating content as answer.");
            return Ok(ReActStep::final_answer("Conversational response", response));
        }

        Err(anyhow::anyhow!("Response failed to follow [TAG] format. You MUST provide [THOUGHT] and then either [ACTION] or [ANSWER]."))
    }

    fn parse_json_tool_call(&self, text: &str) -> Option<ToolCall> {
        if let Some(json_start) = text.find('{') {
            let json_text = &text[json_start..];
            // Find matching brace
            let mut depth = 0;
            let mut json_end = 0;
            for (i, c) in json_text.chars().enumerate() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            json_end = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if json_end > 0 {
                let action_json = &json_text[..json_end];
                if let Ok(call) = serde_json::from_str::<ToolCall>(action_json) {
                    return Some(call);
                }
            }
        }
        None
    }

    fn extract_tag(&self, text: &str, tag: &str) -> Option<String> {
        // Case-insensitive search for the tag
        let tag_lower = tag.to_lowercase();
        let text_lower = text.to_lowercase();
        
        if let Some(start_idx) = text_lower.find(&tag_lower) {
            let start = start_idx + tag.len();
            
            // Look for the next tag to find the end
            let tags = ["[THOUGHT]", "[ACTION]", "[ANSWER]", "[OBSERVATION]"];
            let mut end = text.len();
            
            for t in tags {
                if let Some(next_idx) = text_lower[start..].find(&t.to_lowercase()) {
                    let abs_next_idx = start + next_idx;
                    if abs_next_idx < end {
                        end = abs_next_idx;
                    }
                }
            }
            
            let result = text[start..end].trim().to_string();
            if result.is_empty() { None } else { Some(result) }
        } else {
            None
        }
    }

    fn extract_all_tags(&self, text: &str, tag: &str) -> Vec<String> {
        let mut results = Vec::new();
        let tag_lower = tag.to_lowercase();
        let text_lower = text.to_lowercase();
        
        let mut current_pos = 0;
        while let Some(start_idx) = text_lower[current_pos..].find(&tag_lower) {
            let start = current_pos + start_idx + tag.len();
            
            // Find end (next tag or end of string)
            let tags = ["[THOUGHT]", "[ACTION]", "[ANSWER]", "[OBSERVATION]"];
            let mut end = text.len();
            
            for t in tags {
                if let Some(next_idx) = text_lower[start..].find(&t.to_lowercase()) {
                    let abs_next_idx = start + next_idx;
                    if abs_next_idx < end {
                        end = abs_next_idx;
                    }
                }
            }
            
            let result = text[start..end].trim().to_string();
            if !result.is_empty() {
                results.push(result);
            }
            current_pos = end;
        }
        results
    }

    /// Execute a single step of the ReAct loop
    pub async fn step(&self, query: &str, steps: &[ReActStep], context: Option<&str>) -> Result<ReActStep> {
        let prompt = self.build_react_prompt(query, steps, context).await;
        
        debug!("ReAct prompt:\n{}", prompt);

        let content = self.provider.generate(&self.config.model, prompt, None).await?;

        debug!("LLM response:\n{}", content);

        self.parse_response(&content, query)
    }
}

#[async_trait]
impl Agent for ReActAgent {
    fn agent_type(&self) -> AgentType {
        self.config.agent_type
    }

    fn name(&self) -> &str {
        match self.config.agent_type {
            AgentType::GeneralChat => "GeneralChat",
            AgentType::Reasoner => "Reasoner",
            AgentType::Coder => "Coder",
            AgentType::Researcher => "Researcher",
            AgentType::Planner => "Planner",
            AgentType::Reviewer => "Reviewer",
            AgentType::BitNet => "BitNet",
        }
    }

    fn system_prompt(&self) -> &str {
        &self.config.system_prompt
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    async fn execute(&self, query: &str, context: Option<&str>) -> Result<AgentResponse> {
        info!("ReAct agent starting execution for query: {}", query);
        
        let mut steps = Vec::new();
        
        for iteration in 0..self.config.max_iterations {
            debug!("ReAct iteration {}", iteration + 1);
            
            let mut step = match self.step(query, &steps, context).await {
                Ok(s) => {
                    // Display real-time thought process
                    println!("   💭 {}", truncate(&s.thought, 100));
                    for action in &s.actions {
                        println!("      🔧 Using Tool: {}...", action.name);
                    }
                    s
                },
                Err(e) => {
                    warn!("ReAct step parsing failed: {}", e);
                    steps.push(ReActStep::thought(format!("Parsing error: {}", e)));
                    return Ok(AgentResponse::failure(e.to_string(), steps, self.config.agent_type));
                }
            };

            // LAZINESS FILTER: Detect finishing without action for complex queries
            if step.is_final && steps.is_empty() && is_action_query(query) {
                warn!("Laziness detected: Agent tried to finish without any tool calls for an action query.");
                let hint = "SYSTEM HINT: Your query requires ACTION (creating, analyzing, searching). You MUST use tools first. Do NOT provide a final answer until you have observations from the required tools (e.g., forge_tool, code_exec, codebase_explorer).";
                
                // Convert current final answer back to a thought and continue
                step.is_final = false;
                step.thought = format!("{} [REJECTED: No tool used. I must use tools.]", step.thought);
                
                let mut hint_step = step.clone();
                hint_step.observations.push(hint.to_string());
                steps.push(hint_step);
                continue;
            }
            
            if step.is_final {
                let answer = step.answer.clone().unwrap_or_else(|| step.thought.clone());
                steps.push(step);
                
                info!("ReAct agent completed in {} iterations", iteration + 1);
                return Ok(AgentResponse::success(answer, steps, self.config.agent_type));
            }

            if !step.actions.is_empty() {
                // Loop Guard: Check for redundant tool calls
                if let Some(last_step) = steps.last() {
                    if last_step.actions == step.actions {
                        warn!("Redundant tool calls detected. Injecting loop guard hint.");
                        let mut loop_guard_step = step.clone();
                        loop_guard_step.observations = vec!["SYSTEM HINT: You just called these tools with the same parameters and got the same result. Do NOT repeat yourself. Analyze the previous observations and try DIFFERENT tools, DIFFERENT parameters, or provide your FINAL_ANSWER based on what you already know.".to_string()];
                        steps.push(loop_guard_step);
                        continue;
                    }
                }

                debug!("Executing {} tools in parallel", step.actions.len());
                let results = self.tools.execute_parallel(&step.actions).await;
                
                let mut observations = Vec::new();
                for res in results {
                    let mut obs = match res {
                        Ok(output) => output.summary,
                        Err(e) => format!("Tool execution failed: {}", e),
                    };
                    
                    // Context Compression: Truncate tool outputs if they are too long
                    if obs.len() > 1500 {
                        obs = format!("{}... [Output truncated for memory optimization]", &obs[..1500]);
                    }
                    observations.push(obs);
                }

                let step_with_obs = ReActStep {
                    thought: step.thought.clone(),
                    actions: step.actions.clone(),
                    observations,
                    is_final: false,
                    answer: None,
                };
                steps.push(step_with_obs);
            } else {
                steps.push(step);
            }
        }

        Ok(AgentResponse::failure(
            format!("Reached maximum iterations ({})", self.config.max_iterations),
            steps,
            self.config.agent_type,
        ))
    }
}

/// Simple agent for direct conversation without ReAct loop
#[derive(Clone)]
pub struct SimpleAgent {
    provider: Arc<dyn LLMProvider>,
    config: AgentConfig,
}

impl SimpleAgent {
    pub fn new(ollama: Ollama, config: AgentConfig) -> Self {
        let provider = if let Some(ref url) = config.provider_url {
            Arc::new(OpenAICompatibleProvider::new(url.clone(), None)) as Arc<dyn LLMProvider>
        } else {
            Arc::new(OllamaProvider::new(ollama)) as Arc<dyn LLMProvider>
        };

        Self { provider, config }
    }

    pub fn with_provider(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.provider = provider;
        self
    }

    pub async fn execute_simple(&self, query: &str, context: Option<&str>) -> Result<AgentResponse> {
        let mut prompt = String::new();
        if let Some(ctx) = context {
            prompt.push_str(&format!("## Context\n{}\n\n", ctx));
        }
        prompt.push_str(&format!("## User Query\n{}\n", query));

        debug!("Simple conversational prompt:\n{}", prompt);

        let answer = self.provider.generate(&self.config.model, prompt, Some(self.config.system_prompt.clone())).await?;

        let step = ReActStep::final_answer("Direct response", &answer);

        Ok(AgentResponse::success(answer, vec![step], self.config.agent_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::AgencyProfile;

    #[test]
    fn test_extract_tag() {
        let profile = AgencyProfile::default();
        let config = AgentConfig::new(AgentType::GeneralChat, &profile);
        let agent = ReActAgent::new(Ollama::default(), config, Arc::new(ToolRegistry::new()));
        
        let response = "[THOUGHT]\nI should check the weather.\n[ACTION]\n{\"name\": \"get_weather\", \"parameters\": {\"location\": \"Seattle\"}}\n";
        
        let thought = agent.extract_tag(response, "[THOUGHT]");
        assert_eq!(thought.unwrap(), "I should check the weather.");
        
        let action = agent.extract_tag(response, "[ACTION]");
        assert_eq!(action.unwrap(), "{\"name\": \"get_weather\", \"parameters\": {\"location\": \"Seattle\"}}");
    }

    #[test]
    fn test_parse_response_final_answer() {
        let profile = AgencyProfile::default();
        let config = AgentConfig::new(AgentType::GeneralChat, &profile);
        let agent = ReActAgent::new(Ollama::default(), config, Arc::new(ToolRegistry::new()));
        
        let response = "[THOUGHT]\nI have finished.\n[ANSWER]\nThe weather is sunny.\n";
        let step = agent.parse_response(response, "What is the weather?").unwrap();
        
        assert!(step.is_final);
        assert_eq!(step.answer.unwrap(), "The weather is sunny.");
        assert_eq!(step.thought, "I have finished.");
    }

    #[test]
    fn test_parse_response_action() {
        let profile = AgencyProfile::default();
        let config = AgentConfig::new(AgentType::GeneralChat, &profile);
        let agent = ReActAgent::new(Ollama::default(), config, Arc::new(ToolRegistry::new()));
        
        let response = "[THOUGHT]\nI need to search.\n[ACTION]\n{\"name\": \"search\", \"parameters\": {\"query\": \"rust\"}}\n";
        let step = agent.parse_response(response, "Search for rust").unwrap();
        
        assert!(!step.is_final);
        assert_eq!(step.actions.len(), 1);
        assert_eq!(step.actions[0].name, "search");
        assert_eq!(step.thought, "I need to search.");
    }
}
