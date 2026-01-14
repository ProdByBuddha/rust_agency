//! System Monitor Tool
//! 
//! Provides real-time information about hardware resources, processes, and peripherals.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::info;
use sysinfo::System;

use super::{Tool, ToolOutput};
use crate::memory::MemoryManager;

/// Tool for monitoring system resources and awareness
pub struct SystemTool {
    manager: Arc<MemoryManager>,
}

impl SystemTool {
    pub fn new(manager: Arc<MemoryManager>) -> Self {
        Self { manager }
    }

    fn get_peripherals(&self) -> Value {
        let mut peripherals = json!({
            "usb": [],
            "serial": []
        });

        // List USB devices using nusb
        if let Ok(devices) = nusb::list_devices() {
            for device in devices {
                peripherals["usb"].as_array_mut().unwrap().push(json!({
                    "vendor_id": format!("0x{:04x}", device.vendor_id()),
                    "product_id": format!("0x{:04x}", device.product_id()),
                    "manufacturer": device.manufacturer_string(),
                    "product": device.product_string(),
                }));
            }
        }

        // List Serial ports using serialport
        if let Ok(ports) = serialport::available_ports() {
            for port in ports {
                peripherals["serial"].as_array_mut().unwrap().push(json!({
                    "port_name": port.port_name,
                    "type": format!("{:?}", port.port_type),
                }));
            }
        }

        peripherals
    }

    fn get_processes(&self) -> Value {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let mut processes: Vec<_> = sys.processes().values().collect();
        // Sort by CPU usage descending
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
        
        let top_processes: Vec<_> = processes.iter().take(10).map(|p| {
            json!({
                "pid": p.pid().to_string(),
                "name": p.name(),
                "cpu_usage": format!("{:.1}%", p.cpu_usage()),
                "memory": format!("{} MB", p.memory() / 1024 / 1024),
            })
        }).collect();

        json!(top_processes)
    }
}

#[async_trait]
impl Tool for SystemTool {
    fn name(&self) -> String {
        "system_monitor".to_string()
    }

    fn description(&self) -> String {
        "Monitor local system resources, active processes, and connected peripheral devices (USB/Serial). \n        Use this for system awareness and determining hardware availability.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["status", "processes", "peripherals", "self_awareness"],
                    "description": "The information to retrieve"
                }
            },
            "required": ["action"]
        })
    }

    fn work_scope(&self) -> Value {
        json!({
            "status": "constrained",
            "environment": "local host",
            "access": "telemetry & peripheral listing",
            "side_effects": "none"
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let action = params["action"].as_str().unwrap_or("status");
        info!("SystemTool: Action = {}", action);

        match action {
            "status" => {
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
            },
            "processes" => {
                let proc_data = self.get_processes();
                let summary = "Top 10 CPU Consuming Processes retrieved.";
                Ok(ToolOutput::success(proc_data, summary))
            },
            "peripherals" => {
                let devices = self.get_peripherals();
                let usb_count = devices["usb"].as_array().unwrap().len();
                let serial_count = devices["serial"].as_array().unwrap().len();
                let summary = format!("Detected {} USB devices and {} Serial ports.", usb_count, serial_count);
                Ok(ToolOutput::success(devices, summary))
            },
            "self_awareness" => {
                let mut sys = System::new_all();
                sys.refresh_all();
                let pid = sysinfo::get_current_pid().map_err(|e| anyhow::anyhow!(e))?;
                let process = sys.process(pid).context("Failed to find self process")?;
                
                let data = json!({
                    "pid": pid.to_string(),
                    "memory_usage": format!("{} MB", process.memory() / 1024 / 1024),
                    "cpu_usage": format!("{:.1}%", process.cpu_usage()),
                    "runtime_env": "Nexus SOTA Server",
                });
                
                let summary = format!("Agency Self-Awareness: Running as PID {} with {} MB RAM usage.", pid, process.memory() / 1024 / 1024);
                Ok(ToolOutput::success(data, summary))
            },
            _ => Ok(ToolOutput::failure("Unknown system action"))
        }
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
