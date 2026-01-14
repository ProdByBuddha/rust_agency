/// B.5 - Canonical Reasoning Cycle
/// 
/// Abductive -> Deductive -> Inductive Loop

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningPhase {
    /// B.5:4.1 Abduction (Hypothesis Generation): "Propose"
    Abduction,
    /// B.5:4.2 Deduction (Consequence Derivation): "Analyze"
    Deduction,
    /// B.5:4.3 Induction (Empirical Evaluation): "Test"
    Induction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub id: String,
    pub phase: ReasoningPhase,
    pub transformer_role: String,
    pub hypothesis_id: Option<String>,
    pub predictions: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub outcome: String,
}

pub struct CanonicalReasoningCycle;

impl CanonicalReasoningCycle {
    pub fn next_phase(current: ReasoningPhase) -> ReasoningPhase {
        match current {
            ReasoningPhase::Abduction => ReasoningPhase::Deduction,
            ReasoningPhase::Deduction => ReasoningPhase::Induction,
            ReasoningPhase::Induction => ReasoningPhase::Abduction,
        }
    }
}
