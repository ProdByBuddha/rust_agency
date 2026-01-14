/// E.18 - Transduction Graph Architecture (E.TGA)
/// 
/// The "operating system" for morphisms.
/// Nodes = Morphisms, Edges = U.Transfer.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// E.18:5.1 CtxState — Projection of E.17 Publication Scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtxState {
    pub locus: String,            // L: ContextSlice identifier
    pub reference_plane: String,   // P: Reference plane identifier
    pub editions: HashMap<String, String>, // E⃗: Edition vector (key -> EditionId)
    pub tag: DesignRunTag,         // D: Design or Run
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesignRunTag {
    Design,
    Run,
}

/// E.18:5.1 Nodes (Vertices)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    Signature,                // A.6.0
    Mechanism,                // A.6.1
    Work,                     // A.15 U.WorkEnactment
    Check,                    // OperationalGate
    StructuralReinterpretation, // A.6.4 U.EpistemicRetargeting
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub species_id: String, // Domain-specific specialization
    pub ctx_state: CtxState,
}

/// E.18:5.1 Edge (U.Transfer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub assurance_ops: Vec<AssuranceOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssuranceOp {
    ConstrainTo(String),
    CalibrateTo(String),
    CiteEvidence(String),
    AttributeTo(String),
}

/// E.18:5.1 OperationalGate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalGate {
    pub id: String,
    pub profile: GateProfile,
    pub decision_log: Vec<DecisionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateProfile {
    pub id: String,
    pub required_checks: Vec<String>,
    pub fold_policy: ErrorFoldPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorFoldPolicy {
    Lean,
    Core,
    SafetyCritical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub decision: GateDecision,
    pub rationale: String,
    pub equivalence_witness: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GateDecision {
    Abstain, // Neutral
    Pass,
    Degrade,
    Block,   // Absorbing
}

/// E.18:5.1 CrossingSurface — Auditable publication of GateCrossing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossingSurface {
    pub gate_id: String,
    pub bridge_id: String,
    pub from_state: CtxState,
    pub to_state: CtxState,
    pub cl_penalty: Option<f64>,
    pub path_slice_id: String,
}

/// E.18:5.1 TransductionGraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransductionGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Transfer>,
    pub gates: HashMap<String, OperationalGate>,
}

pub struct TGA;

impl TGA {
    pub fn join_decisions(a: GateDecision, b: GateDecision) -> GateDecision {
        // E.18:7 CC-TGA-21a - Join-semilattice logic
        // abstain < pass < degrade < block
        std::cmp::max(a, b)
    }

    pub fn verify_transfer_preservation(_transfer: &Transfer, source: &Node, target: &Node) -> bool {
        // E.18:5.2 S2 - Raw transfer preserves CtxState
        // This is a simplified check. In a real system, nodes would be resolved from the graph.
        
        // If they have different CtxState, it MUST be a GateCrossing (which is a Node, not a Transfer)
        // Transfers are only between nodes of the same CtxState.
        // Wait, the spec says "any write/update ... occurs at exactly one OperationalGate".
        // So a Transfer edge itself should ideally not bridge different states.
        
        source.ctx_state.locus == target.ctx_state.locus &&
        source.ctx_state.reference_plane == target.ctx_state.reference_plane &&
        source.ctx_state.tag == target.ctx_state.tag &&
        source.ctx_state.editions == target.ctx_state.editions
    }
}