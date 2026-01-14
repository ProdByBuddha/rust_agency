/// C.13 — Constructional Mereology (Compose‑CAL)
/// B.1.1:4.5 — Γ_m (Compose‑CAL) structural aggregators

use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComposeOp {
    Sum,
    Set,
    Slice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeTrace {
    pub op: ComposeOp,
    pub input_ids: Vec<String>,
    pub output_id: String,
    pub notes: String,
}

pub struct ComposeCAL;

impl ComposeCAL {
    /// Γ_m.sum(parts : Set[U.Entity]) -> W : U.Holon
    pub fn sum(&self, parts: HashSet<String>, notes: &str) -> ComposeTrace {
        let output_id = format!("sum_{}", uuid::Uuid::new_v4());
        ComposeTrace {
            op: ComposeOp::Sum,
            input_ids: parts.into_iter().collect(),
            output_id,
            notes: notes.to_string(),
        }
    }

    /// Γ_m.set(elems : Multiset[U.Entity]) -> C : U.Holon
    pub fn set(&self, elems: Vec<String>, notes: &str) -> ComposeTrace {
        let output_id = format!("set_{}", uuid::Uuid::new_v4());
        ComposeTrace {
            op: ComposeOp::Set,
            input_ids: elems,
            output_id,
            notes: notes.to_string(),
        }
    }

    /// Γ_m.slice(ent : U.Entity, facet : U.Facet) -> S : U.Holon
    pub fn slice(&self, entity_id: String, facet: &str, notes: &str) -> ComposeTrace {
        let output_id = format!("slice_{}_{}", entity_id, facet);
        ComposeTrace {
            op: ComposeOp::Slice,
            input_ids: vec![entity_id],
            output_id,
            notes: notes.to_string(),
        }
    }
}
