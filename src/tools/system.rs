//! System Monitor Tool
//! 
//! Provides real-time information about hardware resources (RAM, CPU, Swap).

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::info;

use super::{Tool, ToolOutput};
use crate::memory::MemoryManager;

/// Tool for monitoring system resources
pub struct SystemTool {
    manager: Arc<MemoryManager>,
}

impl SystemTool {
    pub fn new(manager: Arc<MemoryManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl Tool for SystemTool {
    fn name(&self) -> &str {
        "system_monitor"
    }

    fn description(&self) -> &str {
        "Get current hardware resource usage (RAM, CPU, Swap). Use this to assess performance or check if enough memory is available for tasks."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "verbose": {
                    "type": "boolean",
                    "description": "Whether to include detailed breakdown"
                }
            }
        })
    }

    async fn execute(&self, _params: Value) -> Result<ToolOutput> {
        info!("SystemTool: Fetching resource status...");
        let status = self.manager.get_status().await;
        
        let summary = format!(
            "Hardware Status:\n- OS: {}\n- RAM Usage: {:.1}%\n- Used: {} MB / {} MB\n- Swap Usage: {:.1}%",
            status.os_type,
            status.ram_usage_percent,
            status.used_memory_mb,
            status.total_memory_mb,
            status.swap_usage_percent
        );

        Ok(ToolOutput::success(json!(status), summary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::VectorMemory;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_system_tool_execute() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("test_memory.json");
        let memory = Arc::new(VectorMemory::new(path).unwrap());
        let manager = Arc::new(MemoryManager::new(memory));
        let tool = SystemTool::new(manager);
        
        let res = tool.execute(json!({})).await.unwrap();
        assert!(res.success);
        assert!(res.summary.contains("Hardware Status"));
        assert!(res.data.is_object());
    }
}
