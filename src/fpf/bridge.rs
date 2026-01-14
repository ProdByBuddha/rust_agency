/// Part F — The Unification Suite (U‑Suite)
/// 
/// F.9 Alignment & Bridge Across Contexts
/// B.3 Trust & Assurance (Congruence Level)

use serde::{Serialize, Deserialize};
use super::assurance::CongruenceLevel;

/// F.9:4 Alignment Bridge — Mapping between SenseCells with fit/loss
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentBridge {
    pub id: String,
    pub left_cell: String,  // Reference to SenseCell (Context:Label)
    pub right_cell: String, // Reference to SenseCell (Context:Label)
    pub relation: BridgeRelation,
    pub cl: CongruenceLevel,
    pub loss_notes: String,
    pub fit_notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeRelation {
    EquivalentUnderAssumptions,
    NearEquivalent,
    Overlaps,
    BroaderThan,
    NarrowerThan,
    DesignSpecOf,
    RunTraceOf,
    RepresentationOf,
    MemberOfSetIn,
    ProvidesValueFor,
}

/// F.9:10.4 Bridge Composition (Attenuating)
pub struct BridgeCAL;

impl BridgeCAL {
    pub fn compose(a: &AlignmentBridge, b: &AlignmentBridge) -> Option<AlignmentBridge> {
        if a.right_cell != b.left_cell {
            return None;
        }

        // cl* := min(cl1, cl2)
        let cl_star = std::cmp::min(a.cl, b.cl);
        
        // rel* := weaken(rel1, rel2)
        let rel_star = Self::weaken(a.relation, b.relation);

        Some(AlignmentBridge {
            id: format!("composed_{}_{}", a.id, b.id),
            left_cell: a.left_cell.clone(),
            right_cell: b.right_cell.clone(),
            relation: rel_star,
            cl: cl_star,
            loss_notes: format!("Composed loss: {}; {}", a.loss_notes, b.loss_notes),
            fit_notes: format!("Composed fit: {}; {}", a.fit_notes, b.fit_notes),
        })
    }

    fn weaken(r1: BridgeRelation, r2: BridgeRelation) -> BridgeRelation {
        use BridgeRelation::*;
        match (r1, r2) {
            (NearEquivalent, NearEquivalent) => NearEquivalent,
            (NearEquivalent, Overlaps) | (Overlaps, NearEquivalent) => Overlaps,
            _ => Overlaps, // Default weakening
        }
    }
}