/// C.18 - NQD-CAL — Open-Ended Search Calculus
/// 
/// Open-ended search based on Novelty, Quality, and Diversity (NQD).
/// Parameterized by E/E-LOG policies.

use serde::{Serialize, Deserialize};
use super::creativity_chr::NQDBundle;

/// C.18:4 U.DescriptorMap — Mapping to CharacteristicSpace for novelty/diversity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptorMap {
    pub id: String,
    pub edition: String,
    pub characteristic_ids: Vec<String>,
}

/// C.18:4 U.DistanceDef — Definition of "distance" in DescriptorSpace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceDef {
    pub id: String,
    pub edition: String,
    pub metric_type: String, // e.g., "Euclidean", "Cosine"
}

/// C.18:4 U.InsertionPolicy — How to add to the archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertionPolicy {
    pub id: String,
    pub k_capacity: usize,
    pub replacement_strategy: String, // e.g., "LeastNovel", "Oldest"
    pub dedup_threshold: f64,
}

/// C.18:4.2 ArchiveCell — A "niche" in the illumination map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveCell {
    pub id: String,
    pub coordinates: Vec<f64>, // Center of the cell in DescriptorSpace
    pub occupant_id: Option<String>, // Holon ID
    pub best_nqd: Option<NQDBundle>,
}

/// C.18:4.2 IlluminationSummary — Report-only telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IlluminationSummary {
    pub coverage_ratio: f64,
    pub total_cells: usize,
    pub occupied_cells: usize,
    pub avg_novelty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    pub topology: String, // e.g., "grid", "CVT"
    pub resolution: usize,
    pub k_capacity: usize,
    pub insertion_policy_ref: String,
    pub distance_def_edition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterCall {
    pub generator_id: String,
    pub policy_id: String, // Reference to E/E-LOG policy
    pub budget: f64,
    pub seed_ids: Vec<String>,
}

/// C.18.1:4.2 S — Scale Variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaleVariableKind {
    Compute,
    Data,
    Capacity,
    FreedomOfAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleVariable {
    pub kind: ScaleVariableKind,
    pub units: String,
    pub phase: String, // e.g., "TRAIN", "INFER"
}

/// C.18.1:4.2 ElasticityClass χ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElasticityClass {
    Rising,
    Knee,
    Flat,
    Declining,
}

/// C.18.1:4.2 ScaleProbe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleProbe {
    pub points: Vec<f64>,
    pub design: String, // e.g., "factorial", "LHD"
    pub confidence_interval: f64,
}

/// C.18.1:9 SLLProfile (Card@Context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLLProfile {
    pub variables: Vec<ScaleVariable>,
    pub window: (f64, f64), // (Low, High)
    pub probe: ScaleProbe,
    pub elasticity: ElasticityClass,
    pub iso_scale_parity: bool,
}

pub struct NQDCAL;

impl NQDCAL {
    pub fn compute_dominance(candidates: &[NQDBundle]) -> Vec<usize> {
        // Returns indices of non-dominated candidates (Pareto front)
        let mut pareto_indices = Vec::new();
        for (i, c1) in candidates.iter().enumerate() {
            let mut is_dominated = false;
            for (j, c2) in candidates.iter().enumerate() {
                if i == j { continue; }
                if Self::is_dominated_by(c1, c2) {
                    is_dominated = true;
                    break;
                }
            }
            if !is_dominated {
                pareto_indices.push(i);
            }
        }
        pareto_indices
    }

    fn is_dominated_by(c1: &NQDBundle, c2: &NQDBundle) -> bool {
        // C.18:4.1 - Q components dominance (Simplified)
        // c1 is dominated by c2 if c2 is better or equal in all, and better in at least one.
        
        let q1 = match c1.quality.coordinate {
            super::mm_chr::CoordinateValue::Scalar(v) => v,
            _ => 0.0,
        };
        let q2 = match c2.quality.coordinate {
            super::mm_chr::CoordinateValue::Scalar(v) => v,
            _ => 0.0,
        };

        let n1 = match c1.novelty.coordinate {
            super::mm_chr::CoordinateValue::Scalar(v) => v,
            _ => 0.0,
        };
        let n2 = match c2.novelty.coordinate {
            super::mm_chr::CoordinateValue::Scalar(v) => v,
            _ => 0.0,
        };

        // Simplified: only Quality and Novelty for this check
        (q2 >= q1 && n2 >= n1) && (q2 > q1 || n2 > n1)
    }
}