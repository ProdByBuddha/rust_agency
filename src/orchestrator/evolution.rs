use serde::{Deserialize, Serialize};
use crate::orchestrator::{DesignRationaleRecord, MethodDescription};
use uuid::Uuid;

/// FPF-aligned Canonical Evolution Loop (CEL) (B.4)
/// 
/// Captures an 'Evolution Event' where a Method or Tool is refined 
/// based on historical evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEvent {
    pub id: String,
    pub rationale_id: String, // Link to the DRR that triggered this
    pub target_id: String,    // ID of the Method or Tool being evolved
    pub change_description: String,
    pub version_delta: String,
}

pub struct EvolutionEngine;

impl EvolutionEngine {
    /// Analyze a collection of failure DRRs to propose an evolution
    pub fn propose_refinement(
        failures: &[DesignRationaleRecord], 
        method: &MethodDescription
    ) -> Option<EvolutionEvent> {
        // FPF Standard: Evolution requires Evidence Cluster
        // If we have >= 2 failures for the same method, trigger evolution
        if failures.len() >= 2 {
            let rationale = failures.iter()
                .map(|f| f.rationale.clone())
                .collect::<Vec<_>>()
                .join(" | ");

            return Some(EvolutionEvent {
                id: Uuid::new_v4().to_string(),
                rationale_id: failures.last().unwrap().id.clone(),
                target_id: method.id.clone(),
                change_description: format!("Consolidated Refinement: {}", rationale),
                version_delta: "0.1.0".to_string(),
            });
        }
        None
    }
}
