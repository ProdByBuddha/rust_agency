/// G.5 - Multi-Method Dispatcher & MethodFamily Registry
/// 
/// Registry and selector for families of methods (LOG bundles).

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::task_signature::{TaskSignature, PortfolioMode};
use super::assurance::CongruenceLevel;
use super::creativity_chr::NQDBundle;
use super::nqd_cal::IlluminationSummary;

/// G.5:5 S1 - MethodFamily Registry Row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodFamily {
    pub id: String,
    pub context_id: String,
    pub tradition: String,
    pub version: String,
    pub eligibility_standard: EligibilityStandard,
    pub assurance_profile: AssuranceProfile,
    pub cost_model: String,
    pub method_description_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityStandard {
    pub required_data_shapes: Vec<String>,
    pub noise_tolerances: Vec<String>,
    pub resource_envelope: String,
    pub scope_prerequisites: Vec<String>, // USM claims
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssuranceProfile {
    pub formality_level: String, // F0-F9
    pub expected_lanes: Vec<String>, // TA, LA, VA
    pub cl_allowances: HashMap<CongruenceLevel, f64>,
}

/// G.5:5 S1' - GeneratorFamily Registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorFamily {
    pub id: String,
    pub signature: String,
    pub environment_validity_region: String,
    pub transfer_rules_edition: String,
    pub co_evo_couplers: Vec<String>, // MethodFamily IDs
}

/// G.5:5 S3 - Selection Kernel Output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    pub candidates: Vec<String>, // MethodFamily IDs
    pub chosen_family: Option<String>,
    pub portfolio: Option<PortfolioPack>,
    pub drr_id: String,
    pub scr_id: String,
    pub action_hint: Option<String>, // e.g., "strategize"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPack {
    pub mode: PortfolioMode,
    pub variants: Vec<String>, // Candidate IDs
    pub tie_break_notes: String,
}

/// G.5:5 Telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchTelemetry {
    pub path_slice_id: String,
    pub policy_id: String,
    pub qd_metrics: Option<NQDBundle>,
    pub illumination_summary: Option<IlluminationSummary>,
}

pub struct Dispatcher;

impl Dispatcher {
    pub fn select(
        registry: &HashMap<String, MethodFamily>,
        signature: &TaskSignature,
        _policy_id: &str,
    ) -> SelectionResult {
        // G.5:5 S3 - Selection Kernel
        // 1. Eligibility filter
        let mut eligible_ids = Vec::new();
        for (id, family) in registry {
            if Self::is_eligible(family, signature) {
                eligible_ids.push(id.clone());
            }
        }

        // 2. Partial order handling (simplified)
        SelectionResult {
            candidates: eligible_ids.clone(),
            chosen_family: eligible_ids.first().cloned(),
            portfolio: Some(PortfolioPack {
                mode: signature.portfolio_mode,
                variants: eligible_ids,
                tie_break_notes: "Initial selection".to_string(),
            }),
            drr_id: format!("drr_{}", uuid::Uuid::new_v4()),
            scr_id: format!("scr_{}", uuid::Uuid::new_v4()),
            action_hint: None,
        }
    }

    fn is_eligible(_family: &MethodFamily, _signature: &TaskSignature) -> bool {
        // Simplified eligibility check
        true
    }
}