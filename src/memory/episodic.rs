//! Episodic Memory - Sliding window conversation history
//! 
//! Maintains recent conversation turns for context injection.

use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single conversation turn
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationTurn {
    pub role: Role,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub agent: Option<String>,
}

/// Role in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

/// Sliding window episodic memory for conversation history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EpisodicMemory {
    turns: VecDeque<ConversationTurn>,
    max_turns: usize,
    max_tokens_estimate: usize,
}

impl EpisodicMemory {
    /// Create a new episodic memory with specified limits
    pub fn new(max_turns: usize, max_tokens_estimate: usize) -> Self {
        Self {
            turns: VecDeque::new(),
            max_turns,
            max_tokens_estimate,
        }
    }

    /// Add a user message
    pub fn add_user(&mut self, content: impl Into<String>) {
        self.add_turn(ConversationTurn {
            role: Role::User,
            content: content.into(),
            timestamp: Utc::now(),
            agent: None,
        });
    }

    /// Add an assistant/agent response
    pub fn add_assistant(&mut self, content: impl Into<String>, agent: Option<String>) {
        self.add_turn(ConversationTurn {
            role: Role::Assistant,
            content: content.into(),
            timestamp: Utc::now(),
            agent,
        });
    }

    /// Add a system message
    #[allow(dead_code)]
    pub fn add_system(&mut self, content: impl Into<String>) {
        self.add_turn(ConversationTurn {
            role: Role::System,
            content: content.into(),
            timestamp: Utc::now(),
            agent: None,
        });
    }

    /// Add a tool output
    #[allow(dead_code)]
    pub fn add_tool(&mut self, tool_name: impl Into<String>, output: impl Into<String>) {
        self.add_turn(ConversationTurn {
            role: Role::Tool,
            content: format!("[{}]: {}", tool_name.into(), output.into()),
            timestamp: Utc::now(),
            agent: None,
        });
    }

    fn add_turn(&mut self, turn: ConversationTurn) {
        self.turns.push_back(turn);
        self.trim_to_limits();
    }

    fn trim_to_limits(&mut self) {
        // Remove old turns if we exceed max_turns
        while self.turns.len() > self.max_turns {
            self.turns.pop_front();
        }

        // Estimate tokens and trim if needed
        while self.estimate_tokens() > self.max_tokens_estimate && self.turns.len() > 1 {
            self.turns.pop_front();
        }
    }

    fn estimate_tokens(&self) -> usize {
        // Rough estimate: ~4 characters per token
        self.turns.iter().map(|t| t.content.len() / 4).sum()
    }

    /// Get all turns as a slice
    pub fn turns(&self) -> Vec<&ConversationTurn> {
        self.turns.iter().collect()
    }

    /// Format history for prompt injection
    pub fn format_for_prompt(&self) -> String {
        self.turns
            .iter()
            .map(|turn| {
                let role_str = match turn.role {
                    Role::User => "User",
                    Role::Assistant => {
                        if let Some(ref agent) = turn.agent {
                            return format!("{}: {}", agent, turn.content);
                        }
                        "Assistant"
                    }
                    Role::System => "System",
                    Role::Tool => "Tool",
                };
                format!("{}: {}", role_str, turn.content)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Format history as ChatML for native model compatibility
    pub fn format_as_chatml(&self) -> String {
        self.turns
            .iter()
            .map(|turn| {
                let role = match turn.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::System => "system",
                    Role::Tool => "tool",
                };
                let content = if let Role::Assistant = turn.role {
                    if let Some(ref agent) = turn.agent {
                        format!("[{}]: {}", agent, turn.content)
                    } else {
                        turn.content.clone()
                    }
                } else {
                    turn.content.clone()
                };
                format!("<|im_start|>{}\n{}<|im_end|>", role, content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get the last N turns
    pub fn last_n(&self, n: usize) -> Vec<&ConversationTurn> {
        self.turns.iter().rev().take(n).rev().collect()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.turns.clear();
    }

    /// Get the number of turns
    pub fn len(&self) -> usize {
        self.turns.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }

    /// Estimate total tokens in history
    pub fn estimate_total_tokens(&self) -> usize {
        self.turns.iter().map(|t| t.content.len() / 4).sum()
    }

    /// Get all turns for processing
    pub fn get_turns(&self) -> Vec<ConversationTurn> {
        self.turns.iter().cloned().collect()
    }

    /// Replace all turns (used after compaction)
    pub fn replace_turns(&mut self, new_turns: Vec<ConversationTurn>) {
        self.turns = new_turns.into();
        self.trim_to_limits();
    }

    /// Get the last user message
    #[allow(dead_code)]
    pub fn last_user_message(&self) -> Option<&str> {
        self.turns
            .iter()
            .rev()
            .find(|t| t.role == Role::User)
            .map(|t| t.content.as_str())
    }

    /// Serialize to JSON for persistence
    #[allow(dead_code)]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.turns.iter().collect::<Vec<_>>())
    }

    /// Load from JSON
    #[allow(dead_code)]
    pub fn from_json(json: &str, max_turns: usize, max_tokens_estimate: usize) -> serde_json::Result<Self> {
        let turns: Vec<ConversationTurn> = serde_json::from_str(json)?;
        let mut memory = Self::new(max_turns, max_tokens_estimate);
        for turn in turns {
            memory.turns.push_back(turn);
        }
        memory.trim_to_limits();
        Ok(memory)
    }
}

impl Default for EpisodicMemory {
    fn default() -> Self {
        Self::new(20, 4000) // Default: 20 turns, ~4000 tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episodic_memory() {
        let mut memory = EpisodicMemory::new(5, 10000);
        
        memory.add_user("Hello");
        memory.add_assistant("Hi there!", Some("GeneralChat".to_string()));
        memory.add_user("How are you?");
        memory.add_assistant("I'm doing well!", None);
        
        assert_eq!(memory.len(), 4);
        assert_eq!(memory.last_user_message(), Some("How are you?"));
    }

    #[test]
    fn test_sliding_window() {
        let mut memory = EpisodicMemory::new(3, 10000);
        
        for i in 0..5 {
            memory.add_user(format!("Message {}", i));
        }
        
        assert_eq!(memory.len(), 3);
        // Should have messages 2, 3, 4
        let formatted = memory.format_for_prompt();
        assert!(formatted.contains("Message 2"));
        assert!(formatted.contains("Message 4"));
        assert!(!formatted.contains("Message 0"));
    }
}
