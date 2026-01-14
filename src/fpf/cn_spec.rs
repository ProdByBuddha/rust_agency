/// A.19.D1 CN‑frame (comparability & normalization)
/// 
/// The governance card for comparability and normalization.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CNSpec {
    pub name: String,
    pub context_id: String,
    pub cs_basis: Vec<SlotDefinition>,
    pub chart: Chart,
    pub normalization: NormalizationGovernance,
    pub comparability: ComparabilityMode,
    pub indicator_policy: Option<IndicatorPolicy>,
    pub acceptance: AcceptanceGovernance,
    pub aggregation: AggregationGovernance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotDefinition {
    pub slot_id: String,
    pub characteristic_id: String,
    pub scale_type: ScaleType,
    pub unit: Option<String>,
    pub polarity: Polarity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaleType {
    Nominal,
    Ordinal,
    Interval,
    Ratio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Polarity {
    Up,
    Down,
    TargetRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub reference_state: String,
    pub coordinate_patch: String,
    pub protocol_ref: String, // MethodDescription ID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationGovernance {
    pub unm_id: Option<String>,
    pub method_ids: Vec<String>,
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparabilityMode {
    Coordinatewise,
    NormalizationBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorPolicy {
    pub policy_ref: String,
    pub edition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceGovernance {
    pub checklist: Vec<String>,
    pub window_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationGovernance {
    pub gamma_fold: String,
    pub wlnk: bool,
    pub comm: bool,
    pub loc: bool,
    pub mono: bool,
}
