/// C.17 - Creativity-CHR
/// 
/// Captures creativity qualities: Novelty, Quality, Diversity (NQD).
/// Aligns with B.5.2.1 NQD and C.16 measurement substrate.

use serde::{Serialize, Deserialize};
use super::mm_chr::Measure;

/// C.17.1 NQD Qualities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NQDBundle {
    pub novelty: Measure,   // Measured on a defined Scale (e.g., 0-1)
    pub quality: Measure,   // Semantic fidelity/Utility
    pub diversity: Measure, // Variance across a set
}

/// C.17.2 Creativity Context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativityProfile {
    pub id: String,
    pub primary_nqd: NQDBundle,
    pub secondary_metrics: Vec<Measure>,
}

pub struct CreativityCHR;

impl CreativityCHR {
    pub fn is_pareto_dominant(a: &NQDBundle, b: &NQDBundle) -> bool {
        // Simple Pareto dominance check for NQD (C.18/C.19 logic)
        // Assumes higher coordinates are better for all three in this simplified model.
        // In full implementation, would use Polarity from DHCMethod.
        
        let a_vals = vec![a.novelty.coordinate.clone(), a.quality.coordinate.clone(), a.diversity.coordinate.clone()];
        let b_vals = vec![b.novelty.coordinate.clone(), b.quality.coordinate.clone(), b.diversity.coordinate.clone()];
        
        let mut better_in_one = false;
        for i in 0..3 {
            match (&a_vals[i], &b_vals[i]) {
                (super::mm_chr::CoordinateValue::Scalar(av), super::mm_chr::CoordinateValue::Scalar(bv)) => {
                    if av < bv { return false; }
                    if av > bv { better_in_one = true; }
                }
                _ => {} // Handle non-scalar categories if needed
            }
        }
        better_in_one
    }
}