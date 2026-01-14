use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// FPF-aligned Design-Rationale Record (DRR)
/// 
/// Captures the 'Why' behind a system or agentic decision to ensure 
/// auditability, evolvability, and long-term learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignRationaleRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    /// The BoundedContext where this decision was made
    pub context: String,
    /// The specific decision or action taken
    pub decision: String,
    /// The First Principles reasoning behind the decision
    pub rationale: String,
    /// What actually happened after the decision (feedback loop)
    pub consequences: Option<String>,
    /// Optional tags for categorization (e.g. "hardware", "performance", "tooling")
    pub tags: Vec<String>,
}

impl DesignRationaleRecord {
    pub fn new(context: impl Into<String>, decision: impl Into<String>, rationale: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            context: context.into(),
            decision: decision.into(),
            rationale: rationale.into(),
            consequences: None,
            tags: Vec::new(),
        }
    }

    pub fn with_consequences(mut self, consequences: impl Into<String>) -> Self {
        self.consequences = Some(consequences.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Format the DRR into a clear, semantic string for memory storage or prompting
    pub fn format_for_learning(&self) -> String {
        let mut output = format!(
            "--- DESIGN-RATIONALE RECORD (ID: {})
---\
             CONTEXT: {}
             DECISION: {}
             RATIONALE: {}
",
            self.id, self.context, self.decision, self.rationale
        );

        if let Some(ref cons) = self.consequences {
            output.push_str(&format!("CONSEQUENCES: {}
", cons));
        }

        if !self.tags.is_empty() {
            output.push_str(&format!("TAGS: [{}]
", self.tags.join(", ")));
        }

        output.push_str("------------------------------------------");
        output
    }
}
