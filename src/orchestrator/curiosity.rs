//! Curiosity Engine (Minimal)

use std::sync::Arc;
use anyhow::Result;
use tracing::info;
use serde_json::{json, Value};
use crate::agent::{LLMProvider, AgentType};
use crate::memory::Memory;
use crate::orchestrator::queue::TaskQueue;

pub struct CuriosityEngine {
    provider: Arc<dyn LLMProvider>,
    memory: Arc<dyn Memory>,
    queue: Arc<dyn TaskQueue>,
}

impl CuriosityEngine {
    pub fn new(
        provider: Arc<dyn LLMProvider>,
        memory: Arc<dyn Memory>,
        queue: Arc<dyn TaskQueue>,
    ) -> Self {
        Self { provider, memory, queue }
    }

    pub async fn spark_curiosity(&self) -> Result<bool> {
        info!("🧠 Curiosity: Analyzing knowledge gaps...");

        // 1. Fetch recent context to avoid duplication
        let recent = self.memory.get_recent(10).await?;
        let context = recent.iter()
            .map(|m| format!("- {}", m.content))
            .collect::<Vec<_>>()
            .join("\n");

        // 2. Ask the LLM to identify a "blind spot" or exploration target
        let prompt = format!(
            "Your current memory contains:\n{}\n\n\
            Identify ONE thing you don't fully understand about your environment, the codebase, or the system state. \
            Generate an autonomous goal to investigate this gap. \
            \
            Focus areas: Filesystem structure, tool capabilities, optimization opportunities, or environment variables. \
            \
            Return JSON ONLY: {{ \"goal\": \"investigate X\", \"rationale\": \"why\", \"priority\": 0.5 }}",
            context
        );

        let response = self.provider.generate(
            AgentType::Reasoner.default_model(),
            prompt,
            Some("You are the Agency's intrinsic motivation system (CuriosityDrive).".to_string())
        ).await?;

        // 3. Parse and Enqueue
        let cleaned_json = response.trim().trim_matches('`').replace("json", "");
        if let Ok(decision) = serde_json::from_str::<Value>(&cleaned_json) {
            if let Some(goal) = decision["goal"].as_str() {
                info!("🧠 Curiosity: New goal generated: '{}'", goal);
                
                self.queue.enqueue(
                    "autonomous_goal", 
                    json!(goal)
                ).await?;
                
                return Ok(true);
            }
        }

        Ok(false)
    }
}
