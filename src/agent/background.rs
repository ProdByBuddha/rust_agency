use anyhow::Result;
use ollama_rs::Ollama;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error};

use crate::agent::{ContinuousThoughtMachine, LLMCache};
use crate::memory::{Memory, MemoryEntry, entry::MemorySource};
use crate::orchestrator::profile::AgencyProfile;
use crate::tools::ToolRegistry;

/// A machine that thinks in the background without blocking the user
pub struct BackgroundThoughtMachine {
    ctm: ContinuousThoughtMachine,
    memory: Arc<dyn Memory>,
    is_running: bool,
}

impl BackgroundThoughtMachine {
    pub fn new(
        ollama: Ollama, 
        _tools: Arc<ToolRegistry>,
        memory: Arc<dyn Memory>,
        profile: &AgencyProfile
    ) -> Self {
        let ctm = ContinuousThoughtMachine::new(ollama, profile)
            .with_max_cycles(5)
            .with_sync_threshold(0.8);
        
        Self {
            ctm,
            memory,
            is_running: false,
        }
    }

    pub fn with_cache(mut self, cache: Arc<LLMCache>) -> Self {
        self.ctm = self.ctm.with_cache(cache);
        self
    }

    pub async fn start(&mut self) {
        if self.is_running { return; } 
        self.is_running = true;
        
        info!("Background Continuous Thought Machine activated using high-speed temporal unfolding.");
        
        let mut ctm = self.ctm.clone();
        let memory = self.memory.clone();
        
        tokio::spawn(async move {
            loop {
                let query = "Analyze recent interactions and codebase state. What is one technical improvement or architectural insight you can generate right now? Be extremely concise.";
                
                // Get some context from memory to ground the CTM
                let context = match memory.search("recent interactions codebase technical architecture", 5).await {
                    Ok(entries) => {
                        let ctx = entries.iter()
                            .map(|e| format!("[{:?}] {}", e.metadata.source, e.content))
                            .collect::<Vec<_>>()
                            .join("\n");
                        Some(ctx)
                    }
                    Err(_) => None,
                };

                match ctm.unfold(query, context.as_deref()).await {
                    Ok(insight_answer) => {
                        let entry = MemoryEntry::new(
                            format!("BACKGROUND CTM INSIGHT: {}", insight_answer),
                            "BackgroundThoughtMachine",
                            MemorySource::Reflection
                        );
                        
                        if let Err(e) = memory.store(entry).await {
                            error!("Failed to store background insight: {}", e);
                        } else {
                            info!("Background CTM Machine generated a synchronized insight.");
                        }
                    }
                    Err(e) => {
                        error!("Background CTM cycle error: {}", e);
                    }
                }
                
                // Sleep to avoid pegging CPU
                sleep(Duration::from_secs(300)).await; // Every 5 minutes
            }
        });
    }

    pub async fn run_cycle(&mut self) -> Result<()> {
        let query = "Analyze recent interactions and codebase state. What is one technical improvement or architectural insight you can generate right now? Be extremely concise.";
        
        let context = match self.memory.search("recent interactions codebase technical architecture", 5).await {
            Ok(entries) => {
                let ctx = entries.iter()
                    .map(|e| format!("[{:?}] {}", e.metadata.source, e.content))
                    .collect::<Vec<_>>()
                    .join("\n");
                Some(ctx)
            }
            Err(_) => None,
        };

        let insight_answer = self.ctm.unfold(query, context.as_deref()).await?;
        
        let entry = MemoryEntry::new(
            format!("BACKGROUND CTM INSIGHT: {}", insight_answer),
            "BackgroundThoughtMachine",
            MemorySource::Reflection
        );
        
        self.memory.store(entry).await?;
        info!("Background CTM Machine generated a synchronized insight.");
        
        Ok(())
    }
}
