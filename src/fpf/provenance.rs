/// A.10 Evidence Graph Referring & B.1.1 Dependency Graph
/// 
/// "A claim without a chain is only an opinion."

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::traits::DesignRunTag;
use super::mereology::MereologicalRelation;

/// EPV-DAG: Evidence-Provenance DAG
/// Node types: SymbolCarrier, Transformer, MethodDescription, Observation, Episteme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPVDAG {
    pub nodes: Vec<ProvenanceNode>,
    pub edges: Vec<ProvenanceEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvenanceNode {
    SymbolCarrier(SymbolCarrier),
    Transformer(String), // Reference to System in TransformerRole
    MethodDescription(String),
    Observation(String),
    Episteme(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvenanceEdge {
    /// verifiedBy / validatedBy (A.10.2)
    Evidences { from: String, to: String },
    DerivedFrom { from: String, to: String },
    MeasuredBy { from: String, to: String },
    InterpretedBy { from: String, to: String },
    UsedCarrier { from: String, to: String },
    HappenedBefore { from: String, to: String },
}

/// B.1.1 Dependency Graph (D)
/// Encodes part-whole structure only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<String>, // Holon IDs
    pub edges: Vec<MereologicalRelation>,
    pub scope: DesignRunTag,
    pub notes: String,
}

/// SCR: Symbol Carrier Register (A.10.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolCarrier {
    pub id: String,
    pub kind: String, // file, volume, dataset_item
    pub version: String,
    pub checksum: String,
    pub source: String,
}

/// RSCR: Release Symbol Carrier Register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSCR {
    pub context_id: String,
    pub carriers: HashMap<String, SymbolCarrier>,
}

impl EPVDAG {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}