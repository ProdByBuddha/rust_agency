/// C.22 - Problem Typing & TaskSignature Assignment (Problem-CHR)
/// 
/// Minimal typed record sufficient for Eligibility -> Acceptance -> Lawful Selection.

use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use super::mm_chr::{Polarity, ScaleType};
use super::nqd_cal::ArchiveConfig;

/// C.22:5.1 Problem-CHR Fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSignature {
    pub id: String,
    pub context_id: String,
    pub task_kind: String,
    pub kind_set: Vec<String>, // U.Kind[]
    pub data_shape: DataShape,
    pub noise_model: NoiseModel,
    pub objective_profile: ObjectiveProfile,
    pub constraints: Vec<Constraint>,
    pub scope_slice_id: String, // USM ContextSlice ID
    pub evidence_graph_ref: String,
    pub size_scale: SizeScale,
    pub freshness_window: String,
    pub missingness: Missingness,
    pub shift_class: Option<ShiftClass>,
    
    // QD / Illumination Extensions
    pub behavior_space_ref: Option<String>,
    pub archive_config: Option<ArchiveConfig>,
    pub emitter_policy_ref: Option<String>,
    pub dominance_regime_qd: DominanceRegime,
    pub portfolio_mode: PortfolioMode,
    pub budgeting: Budgeting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataShape {
    Tabular,
    Sequence,
    Graph,
    Image,
    Text,
    ODE,
    MIP,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoiseModel {
    IIDGaussian,
    HeavyTailed,
    Adversarial(f64),
    Deterministic,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveProfile {
    pub heads: Vec<ObjectiveHead>,
    pub dominance_regime: DominanceRegime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveHead {
    pub id: String,
    pub scale_type: ScaleType,
    pub polarity: Polarity,
    pub reference_plane: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    Hard(String),
    Soft(String),
    ResourceEnvelope(String),
    RiskEnvelope(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeScale {
    pub n: usize,
    pub m: Option<usize>,
    pub complexity_proxy: f64,
    pub units: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Missingness {
    MCAR, // Missing Completely At Random
    MAR,  // Missing At Random
    MNAR, // Missing Not At Random
    None,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShiftClass {
    CovariateShift,
    ConceptDrift,
    Adversarial,
    Stationary,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DominanceRegime {
    ParetoOnly,
    ParetoPlusIllumination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortfolioMode {
    Pareto,
    Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budgeting {
    pub time_limit_ms: u64,
    pub compute_budget: f64,
    pub cost_ceiling: f64,
    pub units: String,
}

pub struct ProblemCHR;

impl ProblemCHR {

    pub fn is_eligible(&self, _signature: &TaskSignature, _requirements: &HashSet<String>) -> bool {

        // Placeholder for eligibility gating

        true

    }

}
