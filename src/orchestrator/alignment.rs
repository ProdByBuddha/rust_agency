use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::agent::ReActStep;

/// FPF-aligned Assurance Levels (B.3.3)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub enum AssuranceLevel {
    /// L0: Unsubstantiated - Purely theoretical or chat-based response.
    L0,
    /// L1: Evidenced - Reasoning trace and tool outputs exist.
    L1,
    /// L2: Verified - Physical side-effects (files, states) confirmed by cross-check.
    L2,
}

impl std::fmt::Display for AssuranceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::L0 => write!(f, "L0 (Unsubstantiated)"),
            Self::L1 => write!(f, "L1 (Evidenced)"),
            Self::L2 => write!(f, "L2 (Verified)"),
        }
    }
}

/// FPF-aligned U.MethodDescription (A.3.2)
/// 
/// The abstract 'Recipe' or 'SOP' for achieving an objective.
/// This persists as a reusable Episteme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDescription {
    pub id: String,
    pub name: String,
    pub goal: String,
    /// The sequence of abstract steps
    pub steps: Vec<MethodStep>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodStep {
    pub description: String,
    /// The roles required to enact this step (FPF A.2.1)
    pub required_roles: Vec<String>,
}

/// FPF-aligned U.Work (A.15.1)
/// 
/// The 'Record of Occurrence'. Captures what actually happened during a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkRecord {
    pub id: String,
    /// Reference to the MethodDescription this work is executing
    pub method_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    /// The actual execution trace (thoughts, actions, observations)
    pub trace: Vec<ReActStep>,
    pub success: bool,
    pub performer_role: String,
    /// The FPF Assurance Level of this work (B.3.3)
    pub assurance_level: AssuranceLevel,
    /// The formal adjudication result from an independent reviewer (F.12)
    pub adjudication: Option<crate::orchestrator::AdjudicationResult>,
    /// FPF Integration: Evidence Graph & Provenance Ledger (G.6)
    pub evidence_graph: crate::orchestrator::EvidenceGraph,
}

impl MethodDescription {
    pub fn new(name: impl Into<String>, goal: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            goal: goal.into(),
            steps: Vec::new(),
            version: "1.0.0".to_string(),
        }
    }

    pub fn with_step(mut self, description: impl Into<String>, roles: Vec<String>) -> Self {
        self.steps.push(MethodStep {
            description: description.into(),
            required_roles: roles,
        });
        self
    }
}

impl WorkRecord {
    pub fn new(method_id: String, role: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            method_id,
            start_time: Utc::now(),
            end_time: None,
            trace: Vec::new(),
            success: false,
            performer_role: role,
            assurance_level: AssuranceLevel::L0,
            adjudication: None,
            evidence_graph: crate::orchestrator::EvidenceGraph::new(),
        }
    }

    pub fn complete(&mut self, success: bool, level: AssuranceLevel) {
        self.success = success;
        self.assurance_level = level;
        self.end_time = Some(Utc::now());
    }

    pub fn with_adjudication(mut self, adjudication: crate::orchestrator::AdjudicationResult) -> Self {
        self.adjudication = Some(adjudication);
        self
    }

    pub fn format_for_audit(&self) -> String {
        format!(
            "--- U.WORK RECORD (ID: {}) ---\n\
             METHOD: {}\n\
             PERFORMED BY: {}\n\
             STATUS: {}\n\
             ASSURANCE: {}\n\
             DURATION: {}s\n\
             TRACE STEPS: {}\n\
             --------------------------------",
            self.id,
            self.method_id,
            self.performer_role,
            if self.success { "SUCCESS" } else { "FAILURE" },
            self.assurance_level,
            self.end_time.map(|e| (e - self.start_time).num_seconds()).unwrap_or(0),
            self.trace.len()
        )
    }
}
