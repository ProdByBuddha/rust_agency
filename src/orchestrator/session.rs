//! Session Manager - Handles persistence of episodic memory and plans
//! 
//! Inspired by the MemoryController patterns, this module ensures
//! conversation state survives process restarts.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::memory::EpisodicMemory;
use crate::orchestrator::Plan;

/// Persistent session state
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct SessionState {
    /// The conversation history
    pub episodic_memory: EpisodicMemory,
    /// The last executed plan (if any)
    pub last_plan: Option<Plan>,
}

pub struct SessionManager {
    path: PathBuf,
}

impl SessionManager {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Save the current state to disk
    pub async fn save(&self, memory: &EpisodicMemory, plan: Option<&Plan>) -> Result<()> {
        let state = SessionState {
            episodic_memory: memory.clone(),
            last_plan: plan.cloned(),
        };
        
        let json = serde_json::to_string_pretty(&state)
            .context("Failed to serialize session state")?;
        
        fs::write(&self.path, json).await
            .context("Failed to write session file")?;
        
        Ok(())
    }

    /// Load state from disk
    pub async fn load(&self) -> Result<SessionState> {
        if !self.path.exists() {
            return Ok(SessionState::default());
        }

        let json = fs::read_to_string(&self.path).await
            .context("Failed to read session file")?;
        
        let state = serde_json::from_str(&json)
            .context("Failed to deserialize session state")?;
        
        Ok(state)
    }

    /// Clear the session file
    pub async fn clear(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_session_save_load() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("session.json");
        let manager = SessionManager::new(path);
        
        let mut memory = EpisodicMemory::default();
        memory.add_user("Hello");
        memory.add_assistant("Hi there!", None);
        
        let plan = Plan::new("Test goal");
        
        manager.save(&memory, Some(&plan)).await.unwrap();
        let loaded = manager.load().await.unwrap();
        
        assert_eq!(loaded.episodic_memory.len(), 2);
        assert_eq!(loaded.last_plan.unwrap().goal, "Test goal");
    }

    #[tokio::test]
    async fn test_session_clear() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("session.json");
        let manager = SessionManager::new(path.clone());
        
        manager.save(&EpisodicMemory::default(), None).await.unwrap();
        assert!(path.exists());
        
        manager.clear().await.unwrap();
        assert!(!path.exists());
    }
}
