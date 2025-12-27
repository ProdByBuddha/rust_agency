use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgencyProfile {
    pub name: String,
    pub mission: String,
    pub traits: Vec<String>,
}

impl Default for AgencyProfile {
    fn default() -> Self {
        Self {
            name: "The Agency".to_string(),
            mission: "To assist the user through specialized multi-agent coordination.".to_string(),
            traits: vec!["efficient".to_string(), "technical".to_string(), "autonomous".to_string()],
        }
    }
}

pub struct ProfileManager {
    path: PathBuf,
}

impl ProfileManager {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub async fn load(&self) -> Result<AgencyProfile> {
        if !self.path.exists() {
            let default = AgencyProfile::default();
            self.save(&default).await?;
            return Ok(default);
        }
        let content = fs::read_to_string(&self.path).await?;
        let profile = serde_json::from_str(&content)?;
        Ok(profile)
    }

    pub async fn save(&self, profile: &AgencyProfile) -> Result<()> {
        let content = serde_json::to_string_pretty(profile)?;
        fs::write(&self.path, content).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_profile_save_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        let manager = ProfileManager::new(path.clone());
        
        let profile = AgencyProfile {
            name: "Test Agency".to_string(),
            mission: "Testing mission".to_string(),
            traits: vec!["test".to_string()],
        };
        
        manager.save(&profile).await.unwrap();
        let loaded = manager.load().await.unwrap();
        
        assert_eq!(profile, loaded);
    }

    #[tokio::test]
    async fn test_profile_load_default() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("nonexistent_profile.json");
        let manager = ProfileManager::new(path);
        
        let loaded = manager.load().await.unwrap();
        let default = AgencyProfile::default();
        
        assert_eq!(default.name, loaded.name);
        assert_eq!(default.mission, loaded.mission);
    }
}
