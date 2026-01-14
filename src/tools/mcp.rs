//! Model Context Protocol (MCP) Tool Integration
//! 
//! Allows rust_agency to act as an MCP client, connecting to external
//! MCP servers over stdio and dynamically registering their tools.

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use std::sync::Arc;
use tracing::{info, debug};

use crate::agent::{AgentResult, AgentError};
use super::{Tool, ToolOutput};

/// JSON-RPC 2.0 Request
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

/// MCP Tool definition from server
#[derive(Debug, Clone, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP Server Manager
pub struct McpServer {
    name: String,
    child: Mutex<Child>,
    request_counter: Mutex<u64>,
}

impl McpServer {
    pub async fn spawn(name: &str, command: &str, args: &[String]) -> anyhow::Result<Arc<Self>> {
        info!("Spawning MCP server '{}' via {} {:?}...", name, command, args);
        
        let child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Forward stderr to main logs
            .spawn()
            .context("Failed to spawn MCP server process")?;

        let server = Arc::new(Self {
            name: name.to_string(),
            child: Mutex::new(child),
            request_counter: Mutex::new(0),
        });

        // Initialize MCP
        server.initialize().await?;
        
        Ok(server)
    }

    async fn call(&self, method: &str, params: Option<Value>) -> anyhow::Result<Value> {
        let mut id_lock = self.request_counter.lock().await;
        *id_lock += 1;
        let id = *id_lock;
        drop(id_lock);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: json!(id),
        };

        let request_str = serde_json::to_string(&request)? + "\n";
        debug!("MCP Request to {}: {}", self.name, request_str.trim());

        let mut child = self.child.lock().await;
        let stdin = child.stdin.as_mut().context("Failed to open stdin")?;
        stdin.write_all(request_str.as_bytes()).await?;
        stdin.flush().await?;

        let stdout = child.stdout.as_mut().context("Failed to open stdout")?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        
        debug!("MCP Response from {}: {}", self.name, line.trim());

        let response: JsonRpcResponse = serde_json::from_str(&line)?;
        
        if let Some(error) = response.error {
            return Err(anyhow!("MCP Error: {} (code {})", error.message, error.code));
        }

        response.result.context("MCP response missing result and error")
    }

    async fn initialize(&self) -> anyhow::Result<()> {
        let params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "rust_agency",
                "version": "0.1.0"
            }
        });

        self.call("initialize", Some(params)).await?;
        
        // Send initialized notification
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let mut child = self.child.lock().await;
        let stdin = child.stdin.as_mut().context("Failed to open stdin")?;
        stdin.write_all((serde_json::to_string(&notification)? + "\n").as_bytes()).await?;
        stdin.flush().await?;

        Ok(())
    }

    pub async fn list_tools(&self) -> anyhow::Result<Vec<McpToolDefinition>> {
        let result = self.call("tools/list", None).await?;
        let tools: Vec<McpToolDefinition> = serde_json::from_value(result["tools"].clone())?;
        Ok(tools)
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> anyhow::Result<Value> {
        let params = json!({
            "name": name,
            "arguments": arguments
        });
        self.call("tools/call", Some(params)).await
    }
}

/// A Tool implementation that proxies to an MCP server
pub struct McpProxyTool {
    server: Arc<McpServer>,
    definition: McpToolDefinition,
}

impl McpProxyTool {
    pub fn new(server: Arc<McpServer>, definition: McpToolDefinition) -> Self {
        Self { server, definition }
    }
}

#[async_trait]
impl Tool for McpProxyTool {
    fn name(&self) -> String {
        // Prefix with server name to avoid collisions
        format!("{}__{}", self.server.name, self.definition.name)
    }

    fn description(&self) -> String {
        self.definition.description.clone().unwrap_or_else(|| format!("MCP tool from {}", self.server.name))
    }

    fn parameters(&self) -> Value {
        self.definition.input_schema.clone()
    }

    async fn execute(&self, params: Value) -> AgentResult<ToolOutput> {
        info!("Executing MCP tool {}...", self.name());
        let result = self.server.call_tool(&self.definition.name, params).await
            .map_err(|e| AgentError::Tool(format!("MCP call failed: {}", e)))?;
        
        // MCP tools/call result has a 'content' field which is an array of blocks
        let content = result["content"].as_array().ok_or_else(|| AgentError::Tool("Invalid MCP response: missing content array".to_string()))?;
        
        let mut summary = String::new();
        for block in content {
            if let Some(text) = block["text"].as_str() {
                summary.push_str(text);
            }
        }

        let is_error = result["isError"].as_bool().unwrap_or(false);

        if is_error {
            Ok(ToolOutput::failure(summary))
        } else {
            Ok(ToolOutput::success(result, summary))
        }
    }
}
