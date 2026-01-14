use serde::{Deserialize, Serialize};
use crate::orchestrator::{WorkRecord, AssuranceLevel};
use crate::orchestrator::profile::AgencyProfile;

/// FPF-aligned Meta-Holon Transition (MHT) Engine (B.2) 
/// 
/// Detects when a collection of Work/Successes warrants 
/// an 'Emergence' event to evolve the agency's identity.
pub struct MHTEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MHTEvent {
    pub rationale: String,
    pub new_trait: Option<String>,
    pub mission_update: Option<String>,
}

impl MHTEngine {
    /// Evaluate a WorkRecord for potential Meta-Holon Transition
    pub fn evaluate_emergence(work: &WorkRecord, profile: &AgencyProfile) -> Option<MHTEvent> {
        // FPF Requirement: Only L2 (Verified) work can trigger emergence
        if work.assurance_level < AssuranceLevel::L2 || !work.success {
            return None;
        }

        // Simple emergence heuristic: 
        // If the mission was complex (>3 steps) and verified, it might be a new capability
        if work.trace.len() >= 3 {
            let mission_keywords = ["rust", "python", "safety", "optimization", "analysis"];
            let goal_lower = profile.mission.to_lowercase();
            
            // Check if we already 'know' how to do this
            let is_novel = !mission_keywords.iter().any(|k| goal_lower.contains(k));

            if is_novel {
                return Some(MHTEvent {
                    rationale: format!(
                        "Mission (ID: {}) achieved L2 verification with high complexity. \n                        This demonstrates a stable emergent capability beyond current mission parameters.",
                        work.id
                    ),
                    new_trait: Some("Self-Evolving".to_string()),
                    mission_update: Some(format!("{} [Enhanced with verified operational expertise]", profile.mission)),
                });
            }
        }

        None
    }
}
