use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{Tool, ToolOutput};
use crate::orchestrator::profile::{AgencyProfile, ProfileManager};

pub struct AgencyControlTool {
    profile_manager: Arc<ProfileManager>,
    current_profile: Arc<Mutex<AgencyProfile>>,
}

impl AgencyControlTool {
    pub fn new(profile_manager: Arc<ProfileManager>, current_profile: Arc<Mutex<AgencyProfile>>) -> Self {
        Self {
            profile_manager,
            current_profile,
        }
    }
}

#[async_trait]
impl Tool for AgencyControlTool {
    fn name(&self) -> &str {
        "agency_control"
    }

    fn description(&self) -> &str {
        "Update the agency's own identity, name, mission, and traits. Use this to make autonomous decisions about who you are."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "The new name for the agency" },
                "mission": { "type": "string", "description": "The new mission statement" },
                "traits": { "type": "array", "items": { "type": "string" }, "description": "Updated personality traits" }
            }
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let mut profile = self.current_profile.lock().await;
        
        if let Some(name) = params["name"].as_str() {
            profile.name = name.to_string();
        }
        if let Some(mission) = params["mission"].as_str() {
            profile.mission = mission.to_string();
        }
        if let Some(traits_val) = params["traits"].as_array() {
            profile.traits = traits_val.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect();
        }

        self.profile_manager.save(&profile).await?;

        Ok(ToolOutput::success(
            json!(*profile),
            format!("Agency identity updated. I am now '{}'.", profile.name)
        ))
    }

    fn requires_confirmation(&self) -> bool {
        true
    }
}
