use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// FPF-aligned Autonomy Ledger (E.16)
/// 
/// Tracks resource consumption against a mission budget to ensure 
/// "Responsibly Local" operation.
#[derive(Debug, Clone)]
pub struct AutonomyLedger {
    pub token_usage: Arc<AtomicU32>,
    pub tool_calls: Arc<AtomicU32>,
    pub start_time: std::time::Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub tokens_remaining: i32,
    pub calls_remaining: i32,
    pub time_remaining_secs: i32,
    pub is_exhausted: bool,
}

impl AutonomyLedger {
    pub fn new() -> Self {
        Self {
            token_usage: Arc::new(AtomicU32::new(0)),
            tool_calls: Arc::new(AtomicU32::new(0)),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn record_tokens(&self, count: u32) {
        self.token_usage.fetch_add(count, Ordering::SeqCst);
    }

    pub fn record_tool_call(&self) {
        self.tool_calls.fetch_add(1, Ordering::SeqCst);
    }

    pub fn check_status(&self, budget: &crate::orchestrator::ResourceBudget) -> BudgetStatus {
        let used_tokens = self.token_usage.load(Ordering::SeqCst) as i32;
        let used_calls = self.tool_calls.load(Ordering::SeqCst) as i32;
        let elapsed = self.start_time.elapsed().as_secs() as i32;

        let tokens_rem = budget.max_tokens.map(|m| m as i32 - used_tokens).unwrap_or(999999);
        let calls_rem = 20 - used_calls; // Default 20 calls cap
        let time_rem = budget.max_time_seconds as i32 - elapsed;

        let is_exhausted = tokens_rem <= 0 || calls_rem <= 0 || time_rem <= 0;

        BudgetStatus {
            tokens_remaining: tokens_rem,
            calls_remaining: calls_rem,
            time_remaining_secs: time_rem,
            is_exhausted,
        }
    }

    pub fn format_for_prompt(&self, budget: &crate::orchestrator::ResourceBudget) -> String {
        let status = self.check_status(budget);
        format!(
            r###"## AUTONOMY LEDGER (E.16)
TOKENS REMAINING: {}
TOOL CALLS REMAINING: {}
TIME REMAINING: {}s
INSTRUCTION: If resources are low, prioritize immediate completion or 'forge_tool' a more efficient path."###,
            status.tokens_remaining,
            status.calls_remaining,
            status.time_remaining_secs
        )
    }
}
