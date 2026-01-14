//! Safety Module
//! 
//! Guardrails, rate limiting, and content filtering for safe agent operation.

mod rate_limiter;
mod content_filter;
pub mod assurance;
mod command;
pub mod hardening;

pub use rate_limiter::RateLimiter;
pub use content_filter::ContentFilter;
pub use assurance::AssuranceScore;
pub use command::is_dangerous_command;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{warn, info};
use crate::tools::ToolRegistry;
use std::sync::Arc;
use std::collections::HashSet;
use sha2::{Sha256, Digest};

/// Represents a request for human intervention (HITL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub tool_name: String,
    pub parameters: Value,
    pub assurance: AssuranceScore,
    pub rationale: String,
}

/// Safety guard combining rate limiting and content filtering
pub struct SafetyGuard {
    rate_limiter: RateLimiter,
    content_filter: ContentFilter,
    approved_hashes: HashSet<String>,
}

impl SafetyGuard {
    pub fn new() -> Self {
        Self {
            rate_limiter: RateLimiter::new(),
            content_filter: ContentFilter::new(),
            approved_hashes: HashSet::new(),
        }
    }

    /// Calculate a deterministic hash for a tool call to track approvals
    pub fn hash_tool_call(&self, tool_name: &str, params: &Value) -> String {
        let mut hasher = Sha256::new();
        hasher.update(tool_name.as_bytes());
        // Use a stable string representation of JSON for hashing
        let params_str = serde_json::to_string(params).unwrap_or_default();
        hasher.update(params_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Mark a specific tool call as approved
    pub fn approve_call(&mut self, tool_name: &str, params: &Value) {
        let hash = self.hash_tool_call(tool_name, params);
        info!("Registering human approval for tool call hash: {}", hash);
        self.approved_hashes.insert(hash);
    }

    /// Check if a specific tool call was already approved
    pub fn is_approved(&self, tool_name: &str, params: &Value) -> bool {
        let hash = self.hash_tool_call(tool_name, params);
        self.approved_hashes.contains(&hash)
    }

    /// Validate user input before processing
    pub fn validate_input(&self, input: &str) -> Result<()> {
        // Check for prompt injection attempts
        let filter_result = self.content_filter.check_input(input);
        if !filter_result.is_safe {
            warn!("Input blocked by content filter: {:?}", filter_result.reasons);
            anyhow::bail!("Input blocked by safety filter: {}", filter_result.reasons.join(", "));
        }

        // Check input length
        if input.len() > 50000 {
            anyhow::bail!("Input too long (max 50000 characters)");
        }

        Ok(())
    }

    /// Check if a tool call is safe to execute
    pub async fn check_tool_safety(&mut self, tool_name: &str, params: &Value, registry: Arc<ToolRegistry>) -> Result<()> {
        // BYPASS: If human already approved this exact call, we skip further safety hurdles
        if self.is_approved(tool_name, params) {
            info!("Bypassing safety checks for human-approved tool call: {}", tool_name);
            return Ok(());
        }

        // Rate limit tool calls
        if !self.rate_limiter.check_tool(tool_name) {
            anyhow::bail!("Rate limit exceeded for tool: {}", tool_name);
        }

        // FPF Integration: Trust & Assurance (B.3)
        if let Some(tool) = registry.get_tool(tool_name).await {
            let score = AssuranceScore::calculate(tool, params);
            if score.r < 0.3 {
                let msg = score.get_warning().unwrap_or_else(|| "Low trust score.".to_string());
                warn!("Blocked low-assurance tool call: {} (R={:.2}). Rationale: {}", tool_name, score.r, msg);
                anyhow::bail!("FPF ASSURANCE BLOCKED: {} (Score R={:.2} is too low. Refine your plan.)", msg, score.r);
            }
            info!("Tool call evaluated with assurance R={:.2}", score.r);
        }

        // Check dangerous operations
        match tool_name {
            "code_exec" | "sandbox" => {
                // Check for dangerous code patterns
                if let Some(code) = params.get("code").and_then(|c| c.as_str()) {
                    let filter_result = self.content_filter.check_code(code);
                    if !filter_result.is_safe {
                        warn!("Code blocked by safety filter: {:?}", filter_result.reasons);
                        anyhow::bail!("Code blocked: {}", filter_result.reasons.join(", "));
                    }
                    
                    // Also check for shell commands in sandbox
                    if let Some(lang) = params.get("language").and_then(|l| l.as_str()) {
                        if lang == "shell" {
                            let cmd_parts: Vec<String> = code.split_whitespace().map(|s| s.to_string()).collect();
                            if is_dangerous_command(&cmd_parts) {
                                warn!("Dangerous shell command detected in {}: {}", tool_name, code);
                                anyhow::bail!("Dangerous shell command blocked: {}", code);
                            }
                        }
                    }
                }
            }
            "web_search" => {
                // Rate limit web searches more aggressively
                if !self.rate_limiter.check_web_search() {
                    anyhow::bail!("Web search rate limit exceeded");
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if human-in-the-loop approval is needed for a tool call
    pub async fn needs_human_approval(&self, tool_name: &str, params: &Value, registry: Arc<ToolRegistry>) -> Option<ApprovalRequest> {
        // If already approved, definitely don't ask again
        if self.is_approved(tool_name, params) {
            return None;
        }

        if let Some(tool) = registry.get_tool(tool_name).await {
            let score = AssuranceScore::calculate(tool, params);
            
            let is_risky_tool = matches!(tool_name, "code_exec" | "sandbox" | "system_monitor");
            let is_caution_zone = score.r < 0.6 && score.r >= 0.3;
            
            let mut dangerous_cmd = false;
            if tool_name == "sandbox" {
                if let Some(code) = params.get("code").and_then(|c| c.as_str()) {
                    let cmd_parts: Vec<String> = code.split_whitespace().map(|s| s.to_string()).collect();
                    dangerous_cmd = is_dangerous_command(&cmd_parts);
                }
            }

            if is_risky_tool || is_caution_zone || dangerous_cmd {
                return Some(ApprovalRequest {
                    id: uuid::Uuid::new_v4().to_string(),
                    tool_name: tool_name.to_string(),
                    parameters: params.clone(),
                    assurance: score,
                    rationale: if dangerous_cmd { 
                        "Dangerous shell command detected.".to_string() 
                    } else if is_caution_zone {
                        "Assurance score is below trust threshold.".to_string()
                    } else {
                        "High-risk tool call.".to_string()
                    },
                });
            }
        }
        None
    }

    /// Check if confirmation is required for an action (Legacy method, kept for compatibility)
    pub fn requires_confirmation(&self, tool_name: &str, params: &Value) -> bool {
        if matches!(tool_name, "code_exec" | "file_write" | "shell" | "sandbox") {
            // Check if the command is dangerous
            if tool_name == "sandbox" {
                if let Some(code) = params.get("code").and_then(|c| c.as_str()) {
                    let cmd_parts: Vec<String> = code.split_whitespace().map(|s| s.to_string()).collect();
                    if is_dangerous_command(&cmd_parts) {
                        return true;
                    }
                }
            }
            return true;
        }
        false
    }

    /// Reset rate limiters (e.g., at start of new session)
    pub fn reset(&mut self) {
        self.rate_limiter.reset();
    }
}

impl Default for SafetyGuard {
    fn default() -> Self {
        Self::new()
    }
}