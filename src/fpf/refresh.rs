/// G.11 - Telemetry‑Driven Refresh & Decay Orchestrator
/// 
/// Turning telemetry and decay into concrete refresh actions.

use serde::{Serialize, Deserialize};

/// G.11:4.4 RefreshPlan@Context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshPlan {
    pub id: String,
    pub path_slice_ids: Vec<String>,
    pub triggers: Vec<RefreshTrigger>,
    pub actions: Vec<RefreshAction>,
    pub rs_cr_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefreshTrigger {
    T0PolicyChange,
    T1IlluminationIncrease,
    T2EditionBumpQD,
    T3EditionBumpOEE,
    T4BridgeChange,
    T5FreshnessExpiry,
    T6MaturityChange,
    T7DominancePolicyChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefreshAction {
    RecomputeSelection,
    UpdateArchive,
    RebindBridge,
    RepublishBundle,
    RebuildPortfolioSurface,
}

/// G.11:4.4 RefreshReport@Context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshReport {
    pub plan_id: String,
    pub path_ids: Vec<String>,
    pub scr_deltas: Vec<String>,
    pub edition_bump_log: Vec<String>,
}

pub struct RefreshOrchestrator;

impl RefreshPlan {
    pub fn new(id: &str) -> Self {
        RefreshPlan {
            id: id.to_string(),
            path_slice_ids: vec![],
            triggers: vec![],
            actions: vec![],
            rs_cr_refs: vec![],
        }
    }
}