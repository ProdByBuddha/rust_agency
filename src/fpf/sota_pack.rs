/// G.10 - SoTA Pack Shipping (Core Publication Surface)
/// 
/// Release-quality, selector-ready, edition-aware portfolio.

use serde::{Serialize, Deserialize};
use super::task_signature::{PortfolioMode, DominanceRegime};
use super::parity::EditionPins;

/// G.10:3.1 SoTA-Pack(Core)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoTAPack {
    pub id: String,
    pub edition: String,
    pub context_id: String,
    pub frame_ref: String,
    pub comparator_set_ref: String,
    pub parity_pins: ParityPins,
    pub family_ids: Vec<String>,
    pub generator_family_ids: Vec<String>,
    pub sos_log_bundle_ref: Option<String>,
    pub portfolio_mode: PortfolioMode,
    pub dominance_regime: DominanceRegime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityPins {
    pub edition_pins: EditionPins,
    pub phi_policy_ids: Vec<String>,
}

pub struct PackShipping;

impl SoTAPack {
    pub fn new(id: &str, edition: &str) -> Self {
        SoTAPack {
            id: id.to_string(),
            edition: edition.to_string(),
            context_id: "".to_string(),
            frame_ref: "".to_string(),
            comparator_set_ref: "".to_string(),
            parity_pins: ParityPins {
                edition_pins: EditionPins {
                    descriptor_map_edition: "v1".to_string(),
                    distance_def_edition: "v1".to_string(),
                    dhc_method_edition: "v1".to_string(),
                    emitter_policy_ref: "v1".to_string(),
                    insertion_policy_ref: "v1".to_string(),
                    transfer_rules_edition: None,
                },
                phi_policy_ids: vec![],
            },
            family_ids: vec![],
            generator_family_ids: vec![],
            sos_log_bundle_ref: None,
            portfolio_mode: PortfolioMode::Archive,
            dominance_regime: DominanceRegime::ParetoOnly,
        }
    }
}