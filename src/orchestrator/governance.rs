use serde::{Deserialize, Serialize};

use crate::orchestrator::Commitment;

/// FPF-aligned Boundary Norm Square (A.6.B)
/// 
/// Segregates claims into four quadrants to ensure principled governance.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NormSquare {
    /// L: Laws - Hard system invariants (Non-negotiable logic)
    pub laws: Vec<String>,
    /// A: Admissibility - Technical "Green Gates" (Resource/Context checks)
    pub admissibility: Vec<AdmissibilityGate>,
    /// D: Deontics - Obligations and Prohibitions (Ethics/Safety)
    pub deontics: Vec<Commitment>,
    /// E: Work-Effects - Physical traces and Evidence
    pub effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissibilityGate {
    pub name: String,
    pub status: GateStatus,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeonticRule {
    pub description: String,
    pub modality: DeonticModality,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GateStatus {
    Green,
    Yellow,
    Red,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdjudicationVerdict {
    Pass,
    Fail,
    Abstain,
}

/// FPF-aligned Success Adjudication (F.12)
/// 
/// Captures the verdict of a Reviewer agent checking a Performer's work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjudicationResult {
    pub verdict: AdjudicationVerdict,
    pub rationale: String,
    pub reviewer_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeonticModality {
    Must,
    Shall,
    May,
    Forbidden,
}

impl NormSquare {
    pub fn new() -> Self {
        let mut square = Self::default();
        
        // Default System Laws (L)
        square.laws.push("Hardware Lock (fs2) is primary.".to_string());
        square.laws.push("Memory Isolation (A.1.1) must be maintained.".to_string());
        
        square
    }

    pub fn add_gate(&mut self, name: impl Into<String>, status: GateStatus, rationale: impl Into<String>) {
        self.admissibility.push(AdmissibilityGate {
            name: name.into(),
            status,
            rationale: rationale.into(),
        });
    }

    pub fn add_rule(&mut self, commitment: Commitment) {
        self.deontics.push(commitment);
    }

    pub fn is_lawful(&self) -> bool {
        // Red gates block the entire 'will'
        !self.admissibility.iter().any(|g| g.status == GateStatus::Red)
    }

    pub fn format_for_audit(&self) -> String {
        let mut output = String::from("--- FPF BOUNDARY NORM SQUARE (A.6.B) ---\
");
        
        output.push_str("L (LAWS): \n");
        for l in &self.laws { output.push_str(&format!("  - {}\n", l)); }
        
        output.push_str("A (ADMISSIBILITY): \n");
        for g in &self.admissibility { 
            output.push_str(&format!("  - [{:?}] {}: {}\n", g.status, g.name, g.rationale)); 
        } 
        
        output.push_str("D (DEONTICS): \n");
        for d in &self.deontics { 
            output.push_str(&format!("  - {}\n", d.format_for_audit())); 
        } 
        
        output.push_str("E (EFFECTS): \n");
        for e in &self.effects { output.push_str(&format!("  - {}\n", e)); } 
        
        output.push_str("----------------------------------------");
        output
    }
}
