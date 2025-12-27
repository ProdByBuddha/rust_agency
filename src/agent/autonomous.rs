use anyhow::Result;
use ollama_rs::Ollama;
use std::sync::Arc;
use tracing::info;

use crate::agent::{ReActAgent, ReActStep, AgentConfig, AgentType, AgentResponse, Agent, LLMCache, LLMProvider, OllamaProvider, CachedProvider};
use crate::orchestrator::profile::AgencyProfile;
use crate::tools::ToolRegistry;

/// A machine that thinks continuously towards a goal
pub struct AutonomousMachine {
    agent: ReActAgent,
    goal: String,
    steps: Vec<ReActStep>,
}

impl AutonomousMachine {
    pub fn new(ollama: Ollama, tools: Arc<ToolRegistry>, profile: &AgencyProfile, goal: String) -> Self {
        let config = AgentConfig::new(AgentType::Coder, profile);
        let agent = ReActAgent::new(ollama, config, tools);
        Self {
            agent,
            goal,
            steps: Vec::new(),
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

    pub async fn run_iteration(&mut self) -> Result<AgentResponse> {
        info!("Autonomous Machine thinking: {}", self.goal);
        let response = self.agent.execute(&self.goal, None).await?;
        self.steps.extend(response.steps.clone());
        Ok(response)
    }
}