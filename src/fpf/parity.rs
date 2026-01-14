/// G.9 - Parity / Benchmark Harness
/// 
/// Scaffolding for fair, apples-to-apples comparisons across families.

use serde::{Serialize, Deserialize};
use super::task_signature::{PortfolioMode, DominanceRegime};

/// G.9:4.1 ParityPlan@Context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityPlan {
    pub id: String,
    pub frame_id: String,
    pub baseline_set: Vec<String>, // MethodFamily IDs
    pub freshness_windows_ms: u64,
    pub comparator_set_id: String,
    pub budgeting: String, // Budgeting policy id
    pub epsilon: f64,
    pub edition_pins: EditionPins,
    pub portfolio_mode: PortfolioMode,
    pub dominance_regime: DominanceRegime,
}

/// G.9:4.1 EditionPins (when QD/OEE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditionPins {
    pub descriptor_map_edition: String,
    pub distance_def_edition: String,
    pub dhc_method_edition: String,
    pub emitter_policy_ref: String,
    pub insertion_policy_ref: String,
    pub transfer_rules_edition: Option<String>,
}

/// G.9:4.1 ParityReport@Context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityReport {
    pub plan_id: String,
    pub baseline_set: Vec<String>,
    pub portfolio_ids: Vec<String>, // IDs of selected portfolios
    pub path_ids: Vec<String>,      // justification paths
    pub path_slice_id: Option<String>,
    pub illumination_summary: Option<super::nqd_cal::IlluminationSummary>,
    pub rs_cr_refs: Vec<String>,
}

pub struct BenchmarkHarness;

impl ParityPlan {
    pub fn new(id: &str, frame_id: &str) -> Self {
        ParityPlan {
            id: id.to_string(),
            frame_id: frame_id.to_string(),
            baseline_set: vec![],
            freshness_windows_ms: 0,
            comparator_set_id: "".to_string(),
            budgeting: "".to_string(),
            epsilon: 0.0,
            edition_pins: EditionPins {
                descriptor_map_edition: "v1".to_string(),
                distance_def_edition: "v1".to_string(),
                dhc_method_edition: "v1".to_string(),
                emitter_policy_ref: "v1".to_string(),
                insertion_policy_ref: "v1".to_string(),
                transfer_rules_edition: None,
            },
            portfolio_mode: PortfolioMode::Archive,
            dominance_regime: DominanceRegime::ParetoOnly,
        }
    }
}