//! High-Fidelity Context Compaction
//! 
//! Provides logic to summarize and compress long conversation histories
//! while preserving the core objective and recent context.

use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn};

use crate::agent::{LLMProvider, SimpleAgent, AgentConfig, AgentType};
use crate::memory::episodic::{EpisodicMemory, Role};
use crate::orchestrator::profile::AgencyProfile;

pub struct ContextCompactor;

impl ContextCompactor {
    /// Compacts the episodic memory if it exceeds the specified token limit.
    pub async fn compact_if_needed(
        memory: &mut EpisodicMemory,
        provider: Arc<dyn LLMProvider>,
        profile: &AgencyProfile,
        max_tokens: usize,
    ) -> Result<bool> {
        let current_tokens = memory.estimate_total_tokens();
        
        if current_tokens < max_tokens {
            return Ok(false);
        }

        info!("Triggering context compaction (current: {} tokens, limit: {})", current_tokens, max_tokens);

        let turns = memory.get_turns();
        if turns.len() < 10 {
            warn!("Memory is too small to compact effectively, skipping.");
            return Ok(false);
        }

        // 1. Identify parts to keep
        let first_message = turns.first().cloned().unwrap(); // Usually the goal
        let last_n_turns = 5;
        let recent_turns = turns.iter().rev().take(last_n_turns).rev().cloned().collect::<Vec<_>>();
        
        // 2. Identify turns to summarize (everything in between)
        let middle_turns = turns[1..turns.len() - last_n_turns].to_vec();
        let middle_text = middle_turns.iter()
            .map(|t| format!("{}: {}", match t.role { Role::User => "User", _ => "Assistant" }, t.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        // 3. Perform summarization
        let mut config = AgentConfig::new(AgentType::GeneralChat, profile);
        config.model = "qwen2.5:3b-q4".to_string(); // Use a fast model for summary
        let summarizer = SimpleAgent::new_with_provider(provider, config);

        let prompt = format!(
            "Please provide a concise technical summary of the following conversation history. \nFocus on key decisions made, tools used, and the current progress toward the goal. \nKEEP IT UNDER 500 CHARACTERS.\n\n### History to Summarize:\n{}"
            , 
            middle_text
        );

        let summary_response = summarizer.execute_simple(&prompt, None).await?;
        let summary_content = summary_response.answer;

        // 4. Construct new memory state
        let mut new_turns = Vec::new();
        new_turns.push(first_message);
        
        new_turns.push(crate::memory::episodic::ConversationTurn {
            role: crate::memory::episodic::Role::System,
            content: format!("[CONTEXT COMPACTED]: Previous turns summarized here: {}", summary_content),
            timestamp: chrono::Utc::now(),
            agent: Some("SystemCompactor".to_string()),
        });
        
        new_turns.extend(recent_turns);

        memory.replace_turns(new_turns);
        info!("Compaction complete. New token count: {}", memory.estimate_total_tokens());

        Ok(true)
    }
}