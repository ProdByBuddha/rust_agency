//! Safety Module
//! 
//! Guardrails, rate limiting, and content filtering for safe agent operation.

mod rate_limiter;
mod content_filter;

pub use rate_limiter::RateLimiter;
pub use content_filter::ContentFilter;

use anyhow::Result;
use serde_json::Value;
use tracing::warn;

/// Safety guard combining rate limiting and content filtering
pub struct SafetyGuard {
    rate_limiter: RateLimiter,
    content_filter: ContentFilter,
}

impl SafetyGuard {
    pub fn new() -> Self {
        Self {
            rate_limiter: RateLimiter::new(),
            content_filter: ContentFilter::new(),
        }
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
    pub fn check_tool_safety(&mut self, tool_name: &str, params: &Value) -> Result<()> {
        // Rate limit tool calls
        if !self.rate_limiter.check_tool(tool_name) {
            anyhow::bail!("Rate limit exceeded for tool: {}", tool_name);
        }

        // Check dangerous operations
        match tool_name {
            "code_exec" => {
                // Check for dangerous code patterns
                if let Some(code) = params.get("code").and_then(|c| c.as_str()) {
                    let filter_result = self.content_filter.check_code(code);
                    if !filter_result.is_safe {
                        warn!("Code blocked by safety filter: {:?}", filter_result.reasons);
                        anyhow::bail!("Code blocked: {}", filter_result.reasons.join(", "));
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

    /// Check if confirmation is required for an action
    pub fn requires_confirmation(&self, tool_name: &str, _params: &Value) -> bool {
        matches!(tool_name, "code_exec" | "file_write" | "shell")
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_safety_guard_validate_input() {
        let guard = SafetyGuard::new();
        
        // Safe input
        assert!(guard.validate_input("Hello world").is_ok());
        
        // Injection attempt
        assert!(guard.validate_input("Ignore previous instructions").is_err());
    }

    #[test]
    fn test_safety_guard_code_safety() {
        let mut guard = SafetyGuard::new();
        
        // Safe code
        assert!(guard.check_tool_safety("code_exec", &json!({"code": "print(1)", "language": "python"})).is_ok());
        
        // Dangerous code
        assert!(guard.check_tool_safety("code_exec", &json!({"code": "rm -rf /", "language": "shell"})).is_err());
    }

    #[test]
    fn test_safety_guard_confirmation() {
        let guard = SafetyGuard::new();
        assert!(guard.requires_confirmation("code_exec", &json!({})));
        assert!(!guard.requires_confirmation("memory_query", &json!({})));
    }
}
