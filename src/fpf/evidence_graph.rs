/// G.6 - Evidence Graph & Provenance Ledger
/// 
/// Typed DAG for claim justifications: Anchors -> Paths -> PathIds.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::cg_spec::ReferencePlane;
use super::assurance::{Reliability, CongruenceLevel};

/// G.6:4.1 EvidenceGraph Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceNode {
    EvidenceRole { id: String, holder_id: String },
    SymbolCarrier { id: String, uri: String },
    TransformerRole { id: String, system_id: String },
    MethodDescription { id: String, edition: String },
    Observation { id: String, timestamp: chrono::DateTime<chrono::Utc> },
    
    // QD / Illumination attributes
    QDAttributes {
        descriptor_map_ref: String,
        edition: String,
        distance_def_id: String,
    },
}

/// G.6:4.1 EvidenceGraph Edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceEdge {
    VerifiedBy,   // Formal line
    ValidatedBy,  // Empirical line
    FromWorkSet,  // Run-time provenance
    HappenedBefore,
    DerivedFrom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeBinding {
    pub source_id: String,
    pub target_id: String,
    pub relation: EvidenceEdge,
    pub assurance_use: AssuranceLane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssuranceLane {
    TA, // Tool/Technique Assurance
    VA, // Verification Assurance
    LA, // Validation Assurance (empirical)
}

/// G.6:4.2 PathId — Stable identifier for a justification path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathId {
    pub id: String,
    pub claim_id: String,
    pub target_slice_id: String,
    pub reference_plane: ReferencePlane,
    pub lane_split: Vec<AssuranceLane>,
    pub lowest_cl: CongruenceLevel,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
}

/// G.6:4.3 PathSliceId — PathId × Time Window × Plane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSliceId {
    pub path_id: String,
    pub time_window: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
    pub plane: ReferencePlane,
    pub descriptor_map_edition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGraph {
    pub nodes: HashMap<String, EvidenceNode>,
    pub edges: Vec<EdgeBinding>,
}

pub struct ProvenanceLedger;

impl EvidenceGraph {
    pub fn compute_r_eff(&self, _path: &PathId, phi_cl: f64) -> Reliability {
        // G.6:4.4 - R_eff := max(0, min(R_i) - Phi(CL_min))
        // Placeholder: Assuming min(R_i) = 1.0 for simplicity
        let r_raw = 1.0;
        Reliability::new(r_raw - phi_cl)
    }
}