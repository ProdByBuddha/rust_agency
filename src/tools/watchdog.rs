//! Watchdog Tool
//! 
//! Allows agents to set up proactive sensors (HTTP, RSS, File) to
//! monitor the world and trigger background tasks.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use crate::agent::{AgentResult, AgentError};
use crate::tools::{Tool, ToolOutput};
use crate::orchestrator::sensory::SensoryCortex;

pub struct WatchdogTool {
    sensory: Arc<SensoryCortex>,
}

impl WatchdogTool {
    pub fn new(sensory: Arc<SensoryCortex>) -> Self {
        Self { sensory }
    }
}

#[async_trait]
impl Tool for WatchdogTool {
    fn name(&self) -> String {
        "watchdog".to_string()
    }

        fn description(&self) -> String {

            "Set up a proactive sensor to monitor external resources. Supports 'http', 'rss', and 'file'. When a change is detected, a background task will be automatically enqueued.".to_string()

        }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "method": {
                    "type": "string",
                    "enum": ["http", "rss", "file"],
                    "description": "The sensing method to use."
                },
                "target": {
                    "type": "string",
                    "description": "The URL or file path to monitor."
                },
                "interval_seconds": {
                    "type": "integer",
                    "default": 3600,
                    "description": "How often to poll (for http/rss)."
                }
            },
            "required": ["method", "target"]
        })
    }

    async fn execute(&self, params: Value) -> AgentResult<ToolOutput> {
        let method = params["method"].as_str()
            .ok_or_else(|| AgentError::Execution("Missing 'method'".to_string()))?;
        let target = params["target"].as_str()
            .ok_or_else(|| AgentError::Execution("Missing 'target'".to_string()))?;
        let interval = Duration::from_secs(params["interval_seconds"].as_u64().unwrap_or(3600));

        match method {
            "http" => {
                self.sensory.watch_http(target.to_string(), interval).await
                    .map_err(|e| AgentError::Execution(e.to_string()))?;
                Ok(ToolOutput::success(json!({"status": "monitoring"}), format!("Now monitoring URL: {}", target)))
            },
            "rss" => {
                self.sensory.watch_rss(target.to_string(), interval).await
                    .map_err(|e| AgentError::Execution(e.to_string()))?;
                Ok(ToolOutput::success(json!({"status": "monitoring"}), format!("Now monitoring RSS feed: {}", target)))
            },
            "file" => {
                self.sensory.watch_file(target).await
                    .map_err(|e| AgentError::Execution(e.to_string()))?;
                Ok(ToolOutput::success(json!({"status": "monitoring"}), format!("Now watching file/directory: {}", target)))
            },
            _ => Err(AgentError::Execution(format!("Unsupported method: {}", method))),
        }
    }
}
