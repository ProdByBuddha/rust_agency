/// G.0 - CG-Spec - Frame Standard & Comparability Governance
/// 
/// Design-time rules for safe, auditable comparison.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::mm_chr::{ScaleType, Polarity};
use super::assurance::CongruenceLevel;

/// G.0:5.1 CG-Spec Data Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CGSpec {
    pub uts_id: String,
    pub edition: String,
    pub context_id: String,
    pub purpose: String,
    pub scope: CGScope,
    pub described_entity: DescribedEntity,
    pub comparator_set: Vec<ComparatorSpec>,
    pub characteristics: Vec<String>, // CHR Characteristic IDs
    pub scp: HashMap<String, ScaleComplianceProfile>,
    pub minimal_evidence: HashMap<String, EvidenceGate>,
    pub gamma_fold: GammaFold,
    pub cl_routing: HashMap<CongruenceLevel, f64>, // Penalty on R_eff
    pub illumination: Option<IlluminationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CGScope {
    pub slice_id: String, // USM ContextSlice ID
    pub task_kinds: Vec<String>,
    pub object_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribedEntity {
    pub grounding_holon_id: String,
    pub reference_plane: ReferencePlane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferencePlane {
    World,
    Concept,
    Episteme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparatorSpec {
    pub id: String,
    pub kind: ComparatorKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparatorKind {
    ParetoDominance,
    Lexicographic,
    Medoid,
    Median,
    WeightedSum, // Only for interval/ratio
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleComplianceProfile {
    pub scale_types: Vec<ScaleType>,
    pub polarity: Polarity,
    pub unit_alignment_rules: Vec<String>,
    pub guard_macros: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGate {
    pub lanes: Vec<String>, // TA, LA, VA
    pub carriers: Vec<String>,
    pub freshness_window_ms: u64,
    pub failure_behavior: FailureBehavior,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureBehavior {
    Abstain,
    DegradeOrder,
    Sandbox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GammaFold {
    WeakestLink,
    Override { proof_refs: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IlluminationConfig {
    pub q_refs: Vec<String>,
    pub d_refs: Vec<String>,
    pub archive_ref: String,
    pub insertion_policy: String,
    pub dominance_default: bool,
}

pub struct ComparabilityGovernance;

impl CGSpec {
    pub fn verify_legality(&self, characteristic_id: &str, op: &str) -> bool {
        // G.0:7 - No illegal ops (e.g. mean on ordinal)
        if let Some(profile) = self.scp.get(characteristic_id) {
            if profile.scale_types.contains(&ScaleType::Ordinal) && op == "mean" {
                return false;
            }
        }
        true
    }
}