//! Code Execution Tool
//! 
//! Safely executes code snippets in a sandboxed environment.

use anyhow::Context;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{debug, warn};

use crate::agent::{AgentResult, AgentError};
use super::{Tool, ToolOutput};

/// Sandboxed code execution tool
pub struct CodeExecTool {
    /// Maximum execution time in seconds
    timeout_secs: u64,
    /// Maximum output length
    max_output_len: usize,
}

impl CodeExecTool {
    pub fn new() -> Self {
        Self {
            timeout_secs: 30,
            max_output_len: 10000,
        }
    }

    #[allow(dead_code)]
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    async fn execute_python(&self, code: &str) -> anyhow::Result<(String, String, i32)> {
        self.run_command("python3", &["-c", code]).await
    }

    async fn execute_rust(&self, code: &str) -> anyhow::Result<(String, String, i32)> {
        // For Rust, we need to create a temp file and compile
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("agent_code.rs");
        let binary_path = temp_dir.join("agent_code");

        let file_path_str = file_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp file path"))?;
        let binary_path_str = binary_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid binary path"))?;

        tokio::fs::write(&file_path, code).await
            .context("Failed to write Rust code to temp file")?;

        // Compile
        let (stdout, stderr, code_result) = self
            .run_command("rustc", &[
                file_path_str,
                "-o",
                binary_path_str,
            ])
            .await?;

        if code_result != 0 {
            let _ = tokio::fs::remove_file(&file_path).await;
            return Ok((stdout, format!("Compilation failed:\n{}", stderr), code_result));
        }

        // Run the compiled binary
        let result = self.run_command(binary_path_str, &[]).await;

        // Clean up
        let _ = tokio::fs::remove_file(&file_path).await;
        let _ = tokio::fs::remove_file(&binary_path).await;

        result
    }

    async fn execute_javascript(&self, code: &str) -> anyhow::Result<(String, String, i32)> {
        self.run_command("node", &["-e", code]).await
    }

    async fn execute_shell(&self, code: &str) -> anyhow::Result<(String, String, i32)> {
        self.run_command("sh", &["-c", code]).await
    }

    async fn run_command(&self, program: &str, args: &[&str]) -> anyhow::Result<(String, String, i32)> {
        debug!("Running command: {} {:?}", program, args);

        let result = timeout(
            Duration::from_secs(self.timeout_secs),
            Command::new(program)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::null())
                .output()
        ).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let code = output.status.code().unwrap_or(-1);

                // Truncate output if too long
                let stdout = if stdout.len() > self.max_output_len {
                    format!("{}...[truncated]", &stdout[..self.max_output_len])
                } else {
                    stdout.to_string()
                };

                let stderr = if stderr.len() > self.max_output_len {
                    format!("{}...[truncated]", &stderr[..self.max_output_len])
                } else {
                    stderr.to_string()
                };

                Ok((stdout, stderr, code))
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Failed to execute command: {}", e)),
            Err(_) => Err(anyhow::anyhow!("Execution timed out after {} seconds", self.timeout_secs)),
        }
    }
}

impl Default for CodeExecTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CodeExecTool {
    fn name(&self) -> String {
        "code_exec".to_string()
    }

    fn description(&self) -> String {
        "Execute code in a sandboxed environment. Supports Python, JavaScript, Rust, and shell commands.\n 
         Use this to run calculations, test code snippets, or perform automated tasks.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "The code to execute"
                },
                "language": {
                    "type": "string",
                    "description": "Programming language",
                    "enum": ["python", "javascript", "rust", "shell"]
                }
            },
            "required": ["code", "language"]
        })
    }

    fn work_scope(&self) -> Value {
        json!({
            "status": "constrained",
            "environment": "local process (isolated but sharing host resources)",
            "safety": "high (requires manual confirmation)",
            "resource_limits": {
                "timeout": format!("{}s", self.timeout_secs),
                "max_output": format!("{} bytes", self.max_output_len)
            },
            "requirements": ["manual confirmation"]
        })
    }

    fn requires_confirmation(&self) -> bool {
        true // Always require confirmation for code execution
    }

    async fn security_oracle(&self, params: &Value) -> AgentResult<bool> {
        let code = params["code"].as_str().unwrap_or("");
        let language = params["language"].as_str().unwrap_or("");

        if language == "shell" {
            // Check for shell operators using PAI Oracle standard
            if let Ok(true) = pai_core::oracle::VerificationOracle::verify(
                pai_core::oracle::OracleType::GrepMatch,
                &format!("[;&|`$]|{}", code)
            ) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn execute(&self, params: Value) -> AgentResult<ToolOutput> {
        let code = params["code"]
            .as_str()
            .ok_or_else(|| AgentError::Validation("Missing required parameter: code".to_string()))?;
        
        let language = params["language"]
            .as_str()
            .ok_or_else(|| AgentError::Validation("Missing required parameter: language".to_string()))?;

        debug!("Executing {} code ({} chars)", language, code.len());

        let result = match language {
            "python" => self.execute_python(code).await,
            "javascript" => self.execute_javascript(code).await,
            "rust" => self.execute_rust(code).await,
            "shell" => self.execute_shell(code).await,
            _ => return Ok(ToolOutput::failure(format!("Unsupported language: {}", language))),
        };

        match result {
            Ok((stdout, stderr, exit_code)) => {
                let success = exit_code == 0;
                let mut output_parts = Vec::new();
                
                if !stdout.is_empty() {
                    output_parts.push(format!("stdout:\n{}", stdout));
                }
                if !stderr.is_empty() {
                    output_parts.push(format!("stderr:\n{}", stderr));
                }
                
                let summary = if success {
                    if stdout.is_empty() && stderr.is_empty() {
                        "Code executed successfully (no output)".to_string()
                    } else {
                        output_parts.join("\n\n")
                    }
                } else {
                    format!("Execution failed (exit code: {})\n{}", exit_code, output_parts.join("\n\n"))
                };

                if success {
                    Ok(ToolOutput::success(
                        json!({
                            "language": language,
                            "stdout": stdout,
                            "stderr": stderr,
                            "exit_code": exit_code
                        }),
                        summary
                    ))
                } else {
                    Ok(ToolOutput {
                        success: false,
                        data: json!({
                            "language": language,
                            "stdout": stdout,
                            "stderr": stderr,
                            "exit_code": exit_code
                        }),
                        summary,
                        error: Some(format!("Exit code: {}", exit_code)),
                    })
                }
            }
            Err(e) => {
                warn!("Code execution failed: {}", e);
                Ok(ToolOutput::failure(format!("Execution failed: {}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_code_exec_shell() {
        let tool = CodeExecTool::new();
        let res = tool.execute(json!({
            "language": "shell",
            "code": "echo 'hello'"
        })).await.expect("Execution failed");
        
        assert!(res.success);
        assert!(res.summary.contains("hello"));
    }

    #[tokio::test]
    async fn test_code_exec_unsupported() {
        let tool = CodeExecTool::new();
        let res = tool.execute(json!({
            "language": "cobol",
            "code": "DISPLAY 'HELLO'"
        })).await.expect("Execution failed");
        
        assert!(!res.success);
        assert!(res.summary.contains("Unsupported language"));
    }
}