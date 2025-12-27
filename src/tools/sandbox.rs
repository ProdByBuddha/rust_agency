//! Sandbox Execution Tool
//! 
//! Provides a unified interface for executing code and managing files
//! across different backends (Local Docker, Daytona, E2B).

use anyhow::{Context, Result};
use async_trait::async_trait;
use bollard::container::LogOutput;
use bollard::models::ContainerCreateBody;
use bollard::container::Config;
use bollard::container::RemoveContainerOptions;
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::Docker;
use bollard::image::CreateImageOptions;
use bollard::container::CreateContainerOptions;
use bollard::container::StartContainerOptions;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn};

use super::{Tool, ToolOutput};

/// Backend providers for the sandbox
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SandboxProvider {
    Local,
    Daytona,
    E2B,
}

/// Unified Sandbox Tool
pub struct SandboxTool {
    provider: SandboxProvider,
}

impl SandboxTool {
    pub fn new(provider: SandboxProvider) -> Self {
        Self { provider }
    }

    async fn execute_local_docker(&self, code: &str, language: &str) -> Result<ToolOutput> {
        info!("Initializing local Docker sandbox for {}...", language);
        
        let docker = Docker::connect_with_local_defaults()
            .context("Failed to connect to Docker Desktop. Ensure it is running.")?;

        let image = match language {
            "python" => "python:3.11-slim",
            "rust" => "rust:1.75-slim",
            "javascript" => "node:20-slim",
            _ => "ubuntu:latest",
        };

        // 0. Ensure image exists
        let mut pull_stream = docker.create_image(
            Some(CreateImageOptions {
                from_image: image.to_string(),
                ..Default::default()
            }),
            None,
            None,
        );
        
        while let Some(pull_result) = pull_stream.next().await {
            if let Err(e) = pull_result {
                warn!("Image pull warning: {}", e);
            }
        }

        // 1. Create container
        let container_name = format!("agency-sandbox-{}", uuid::Uuid::new_v4());
        let config = ContainerCreateBody {
            image: Some(image.to_string()),
            tty: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        docker.create_container(
            Some(CreateContainerOptions { 
                name: container_name.clone(),
                ..Default::default()
            }),
            config
        ).await.context("Failed to create Docker container")?;

        // 2. Start container
        docker.start_container(&container_name, None::<StartContainerOptions<String>>)
            .await.context("Failed to start Docker container")?;

        // 3. Prepare execution - We use a file-based approach to avoid shell escaping issues
        let filename = match language {
            "python" => "script.py",
            "javascript" => "script.js",
            "rust" => "main.rs",
            _ => "script.sh",
        };

        // Escaping for the heredoc
        let escaped_code = code.replace("'", "'\'\''");
        let write_cmd = format!("cat << 'EOF' > {}
{}
EOF", filename, escaped_code);
        
        let exec_write = docker.create_exec(&container_name, CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            cmd: Some(vec!["sh", "-c", &write_cmd]),
            ..Default::default()
        }).await?.id;
        docker.start_exec(&exec_write, None).await?;

        // 4. Run the code
        let run_cmd = match language {
            "python" => vec!["python3", filename],
            "javascript" => vec!["node", filename],
            "rust" => vec!["sh", "-c", "rustc main.rs && ./main"],
            _ => vec!["sh", filename],
        };

        let exec_run = docker.create_exec(&container_name, CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            cmd: Some(run_cmd),
            ..Default::default()
        }).await?.id;

        let mut stdout = String::new();
        let mut stderr = String::new();

        if let StartExecResults::Attached { mut output, .. } = docker.start_exec(&exec_run, None).await? {
            while let Some(Ok(msg)) = output.next().await {
                match msg {
                    LogOutput::StdOut { message } => stdout.push_str(&String::from_utf8_lossy(&message)),
                    LogOutput::StdErr { message } => stderr.push_str(&String::from_utf8_lossy(&message)),
                    _ => {{}}
                }
            }
        }

        // 5. Cleanup
        let _ = docker.remove_container(&container_name, Some(RemoveContainerOptions { force: true, ..Default::default() })).await;

        if stderr.is_empty() || !stdout.is_empty() {
             Ok(ToolOutput::success(
                json!({ "stdout": stdout, "stderr": stderr }),
                format!("Execution Output:\n{}", stdout)
            ))
        } else {
            Ok(ToolOutput::failure(format!("Execution Error:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr)))
        }
    }

    async fn execute_daytona(&self, _code: &str, _language: &str) -> Result<ToolOutput> {
        Ok(ToolOutput::failure("Daytona provider is currently disabled. Using local Docker."))
    }
}

impl Default for SandboxTool {
    fn default() -> Self {
        Self::new(SandboxProvider::Local)
    }
}

#[async_trait]
impl Tool for SandboxTool {
    fn name(&self) -> &str {
        "sandbox"
    }

    fn description(&self) -> &str {
        "Advanced isolated execution environment. Supports 'run' for Python, Rust, JS, and Shell. \n        Code MUST print results to stdout. Scripts are executed as standalone files inside a clean container."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["run"],
                    "description": "Action to perform"
                },
                "code": {
                    "type": "string",
                    "description": "The code to execute"
                },
                "language": {
                    "type": "string",
                    "description": "Language: python, rust, javascript, shell",
                    "enum": ["python", "rust", "javascript", "shell"]
                }
            },
            "required": ["action", "code", "language"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let action = params["action"].as_str().unwrap_or("run");
        
        match action {
            "run" => {
                let code = params["code"].as_str().context("Missing code parameter")?;
                let lang = params["language"].as_str().unwrap_or("python");
                
                match self.provider {
                    SandboxProvider::Local => self.execute_local_docker(code, lang).await,
                    SandboxProvider::Daytona => self.execute_daytona(code, lang).await,
                    SandboxProvider::E2B => Ok(ToolOutput::failure("E2B provider not yet configured")),
                }
            },
            _ => Ok(ToolOutput::failure(format!("Action {} not supported by sandbox", action)))
        }
    }
}
