/// C.19 - E/E-LOG — Explore–Exploit Governor
/// 
/// Defines exploration/exploitation policies and selection lenses.
/// Governs NQD-CAL generators and selectors.

use serde::{Serialize, Deserialize};

/// C.19:4 EmitterPolicy — Governs generator behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterPolicy {
    pub id: String,
    pub exploration_quota: f64, // Ratio [0, 1]
    pub risk_budget: f64,
    pub focus_lens_id: String,
}

/// C.19.1 Bitter-Lesson Preference (BLP)
/// Default policy that prefers general, scale-amenable methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitterLessonPreference {
    pub enabled: bool,
    pub scale_probe_required: bool,
    pub general_method_bonus: f64,
}

/// C.19.4 SelectionLens — How to pick from the Pareto front
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionLens {
    pub id: String,
    pub priority_sequence: Vec<LensPriority>, // e.g., Quality -> Novelty -> Diversity
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LensPriority {
    Quality,
    Novelty,
    Diversity,
    ConstraintFit,
    ScaleElasticity,
}

pub struct EELOG;

impl EELOG {
    pub fn apply_lens(indices: &[usize], _priorities: &[LensPriority]) -> usize {
        // Simple implementation: pick the first one if multiple are on the front.
        // In full implementation, this would sort indices based on priority sequence.
        indices.first().cloned().unwrap_or(0)
    }

    pub fn blp_check(candidate_is_general: bool, policy: &BitterLessonPreference) -> f64 {
        if policy.enabled && candidate_is_general {
            policy.general_method_bonus
        } else {
            0.0
        }
    }
}