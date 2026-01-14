/// G.8 - SoS-LOG Bundles & Maturity Ladders
/// 
/// Bundling rules, maturity, and telemetry into a selector-facing package.

use serde::{Serialize, Deserialize};
use super::sos_log::{MaturityCard, MaturityRung, AdmissibilityVerdict};
use super::task_signature::PortfolioMode;

/// G.8:4.2 SoS-LOGBundle@Context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoSLOGBundle {
    pub id: String,
    pub method_family_id: String,
    pub home_context_id: String,
    pub rule_ids: Vec<String>,
    pub maturity_card: MaturityCard,
    pub portfolio_mode: PortfolioMode,
    pub bridge_ids: Vec<String>,
    pub phi_policy_ids: Vec<String>,
}

/// G.8:4.3 Admissibility Ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissibilityLedger {
    pub id: String,
    pub entries: Vec<AdmissibilityLedgerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissibilityLedgerEntry {
    pub family_id: String,
    pub rule_id: String,
    pub maturity_rung: MaturityRung,
    pub verdict: AdmissibilityVerdict,
    pub edition: String,
}

pub struct LOGBundling;

impl SoSLOGBundle {
    pub fn new(id: &str, family_id: &str) -> Self {
        SoSLOGBundle {
            id: id.to_string(),
            method_family_id: family_id.to_string(),
            home_context_id: "".to_string(),
            rule_ids: vec![],
            maturity_card: MaturityCard {
                family_id: family_id.to_string(),
                rung: MaturityRung::L0Anecdotal,
                evidence_graph_path_ids: vec![],
            },
            portfolio_mode: PortfolioMode::Archive,
            bridge_ids: vec![],
            phi_policy_ids: vec![],
        }
    }
}