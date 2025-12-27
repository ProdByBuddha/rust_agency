//! Continuous Thought Machine (CTM) Implementation
//! 
//! Inspired by SakanaAI's Continuous Thought Machines, this module implements 
//! an agentic version of internal temporal unfolding and neural synchronization.
//! 
//! Instead of a single reasoning step, the CTM unfolds its internal state over 
//! multiple "temporal cycles" before producing a synchronized output.

use anyhow::Result;
use ollama_rs::Ollama;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use std::sync::Arc;

use crate::agent::{AgentConfig, AgentType, SimpleAgent, LLMProvider, OllamaProvider, LLMCache, CachedProvider};
use crate::orchestrator::profile::AgencyProfile;

/// A single step in the internal temporal unfolding of a thought
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalStep {
    pub cycle: usize,
    pub internal_thought: String,
    pub confidence: f32,
}

/// The Continuous Thought Machine
#[derive(Clone)]
pub struct ContinuousThoughtMachine {
    agent: SimpleAgent,
    thought_buffer: Vec<TemporalStep>,
    max_cycles: usize,
    sync_threshold: f32,
}

impl ContinuousThoughtMachine {
    pub fn new(
        ollama: Ollama,
        profile: &AgencyProfile,
    ) -> Self {
        let config = AgentConfig::new(AgentType::BitNet, profile);
        let agent = SimpleAgent::new(ollama, config);
        
        Self {
            agent,
            thought_buffer: Vec::new(),
            max_cycles: 10,
            sync_threshold: 0.85,
        }
    }

    pub fn with_cache(mut self, cache: Arc<LLMCache>) -> Self {
        let ollama = Ollama::default(); 
        let provider = Arc::new(OllamaProvider::new(ollama)) as Arc<dyn LLMProvider>;
        let cached_provider = Arc::new(CachedProvider::new(provider, cache));
        self.agent = self.agent.with_provider(cached_provider);
        self
    }

    pub fn with_provider(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.agent = self.agent.with_provider(provider);
        self
    }

    pub fn with_max_cycles(mut self, cycles: usize) -> Self {
        self.max_cycles = cycles;
        self
    }

    pub fn with_sync_threshold(mut self, threshold: f32) -> Self {
        self.sync_threshold = threshold;
        self
    }

    /// Unfold the internal thought process over multiple temporal cycles
    pub async fn unfold(&mut self, query: &str, context: Option<&str>) -> Result<String> {
        self.thought_buffer.clear();
        info!("CTM unfolding thought process for: '{}'", query);

        for cycle in 1..=self.max_cycles {
            let thought = self.execute_cycle(query, cycle, context).await?;
            
            let confidence = self.evaluate_synchronization(&thought).await?;
            
            let step = TemporalStep {
                cycle,
                internal_thought: thought.clone(),
                confidence,
            };
            
            self.thought_buffer.push(step);
            
            debug!("Cycle {} Confidence: {:.2}", cycle, confidence);

            if confidence >= self.sync_threshold && cycle >= 3 {
                info!("CTM synchronized at cycle {} with confidence {:.2}", cycle, confidence);
                break;
            }
        }

        self.produce_synchronized_output(query).await
    }

    async fn execute_cycle(&self, query: &str, cycle: usize, context: Option<&str>) -> Result<String> {
        let history = self.thought_buffer
            .iter()
            .map(|s| format!("Cycle {}: {}", s.cycle, s.internal_thought))
            .collect::<Vec<_>>()
            .join("\n");

        let context_str = context.map(|c| format!("\nCONTEXT:\n{}\n", c)).unwrap_or_default();

        let prompt = format!(
            r###"INTERNAL TEMPORAL AXIS - CYCLE {} 
TASK: {} 
{}
PREVIOUS INTERNAL STATES:
{} 

INSTRUCTIONS:
Refine the internal representation of the solution. 
If this is an early cycle, explore possibilities. 
If this is a later cycle, converge on the most optimal and accurate thought.
BE EXTREMELY CONCISE. This is an internal thought, not a final answer.
"###,
            cycle, query, context_str, history
        );

        let response = self.agent.execute_simple(&prompt, None).await?;
        Ok(response.answer)
    }

    async fn evaluate_synchronization(&self, current_thought: &str) -> Result<f32> {
        if self.thought_buffer.is_empty() {
            return Ok(0.5); // Initial confidence
        }

        let last_thought = &self.thought_buffer.last().unwrap().internal_thought;
        
        // Use a very fast LLM call to judge "synchronization" (convergence)
        let prompt = format!(
            r###"Compare these two sequential internal thoughts. 
Are they converging on a stable conclusion?
Provide a synchronization score between 0.0 and 1.0.
0.0 = Completely different/divergent
1.0 = Perfectly synchronized/stable

THOUGHT A: {} 
THOUGHT B: {} 

SCORE:"###,
            last_thought, current_thought
        );

        let response = self.agent.execute_simple(&prompt, None).await?;
        let score_text = response.answer.trim();
        
        // Simple parsing of the score
        let score = score_text.split_whitespace()
            .find_map(|s| s.parse::<f32>().ok())
            .unwrap_or(0.7);

        Ok(score)
    }

    async fn produce_synchronized_output(&self, query: &str) -> Result<String> {
        let final_thought = self.thought_buffer.last() 
            .map(|s| s.internal_thought.as_str())
            .unwrap_or("No internal state reached.");

        let prompt = format!(
            r###"Based on the following internal temporal unfolding, provide the final synchronized answer to the query.

QUERY: {} 

SYNCHRONIZED INTERNAL STATE:
{} 

FINAL ANSWER:"###,
            query, final_thought
        );

        let response = self.agent.execute_simple(&prompt, None).await?;
        Ok(response.answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockCTMProvider;

    #[async_trait]
    impl LLMProvider for MockCTMProvider {
        async fn generate(&self, _model: &str, prompt: String, _system: Option<String>) -> Result<String> {
            if prompt.contains("SCORE:") {
                Ok("0.9".to_string())
            } else if prompt.contains("FINAL ANSWER:") {
                Ok("The synchronized answer is 42.".to_string())
            } else {
                Ok("Thinking about the problem...".to_string())
            }
        }
    }

    #[tokio::test]
    async fn test_ctm_unfold() {
        let profile = AgencyProfile::default();
        let mut ctm = ContinuousThoughtMachine::new(Ollama::default(), &profile)
            .with_provider(Arc::new(MockCTMProvider))
            .with_max_cycles(3)
            .with_sync_threshold(0.8);
            
        let result = ctm.unfold("What is the meaning of life?", None).await.unwrap();
        assert_eq!(result, "The synchronized answer is 42.");
        assert!(ctm.thought_buffer.len() >= 1);
    }
}
