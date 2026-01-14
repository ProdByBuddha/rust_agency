use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

/// FPF-aligned Heuristic Debt (BLP-4)
/// 
/// Captures a hand-tuned rule that was admitted for pragmatic reasons 
/// but is slated for replacement by a general scalable method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicDebt {
    pub id: String,
    pub scope: String,
    pub description: String,
    pub admission_date: DateTime<Utc>,
    pub expiry_date: DateTime<Utc>,
    pub replacement_target: String, // e.g. "LLM-based Complexity Estimator"
    pub de_hardening_plan: String,
}

impl HeuristicDebt {
    pub fn new(
        scope: impl Into<String>,
        desc: impl Into<String>,
        target: impl Into<String>,
        days_valid: i64
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            scope: scope.into(),
            description: desc.into(),
            admission_date: now,
            expiry_date: now + Duration::days(days_valid),
            replacement_target: target.into(),
            de_hardening_plan: "Transition to scale-leveraged search/inference.".to_string(),
        }
    }
}

/// Central Ledger for tracking constitutional debt.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DebtRegistry {
    pub entries: HashMap<String, HeuristicDebt>,
}

impl DebtRegistry {
    pub fn register(&mut self, debt: HeuristicDebt) {
        self.entries.insert(debt.id.clone(), debt);
    }

    pub fn format_for_audit(&self) -> String {
        if self.entries.is_empty() {
            return "HEURISTIC DEBT REGISTER: [CLEAR]".to_string();
        }

        let mut output = "--- FPF HEURISTIC DEBT REGISTER (BLP-4) ---\
".to_string();
        for debt in self.entries.values() {
            let status = if debt.expiry_date < Utc::now() { "[EXPIRED]" } else { "[ACTIVE]" };
            output.push_str(&format!(
                "{} ID: {} | Scope: {}\n  - Desc: {}\n  - Target: {}\n",
                status, debt.id, debt.scope, debt.description, debt.replacement_target
            ));
        }
        output.push_str("-------------------------------------------");
        output
    }
}
