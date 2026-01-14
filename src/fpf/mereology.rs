/// A.14 Advanced Mereology: Components, Portions, Aspects & Phases
/// 
/// Metrical parthood (PortionOf) and temporal parthood (PhaseOf).

use serde::{Serialize, Deserialize};
use chrono::Utc;

/// B.1.1 Normative edge vocabulary V_rel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MereologicalRelation {
    /// Physical/machined part in an assembly (Γ_sys)
    ComponentOf { whole_id: String },
    /// Logical/content part in a conceptual whole (Γ_epist)
    ConstituentOf { whole_id: String },
    /// Quantitative fraction of a homogeneous stock or carrier (Γ_sys / Γ_work)
    PortionOf(Portion),
    /// Temporal phase/slice of the same carrier (Γ_time / Γ_work)
    PhaseOf(Phase),
}

/// PortionOf: metrical part of a measurable whole
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portion {
    pub whole_id: String,
    pub measure_kind: String, // mass, volume, token_count
    pub quantity: f64,
    pub unit: String,
}

/// PhaseOf: temporal part of the same carrier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    pub carrier_id: String,
    pub start_time: chrono::DateTime<Utc>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub identity_criterion: String,
}

/// Non-mereological membership (outside V_rel)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberOf {
    pub member_id: String,
    pub collection_id: String,
}