/// C.3 - Kinds, Intent/Extent, and Typed Reasoning (Kind‑CAL)
/// C.3.1 - U.Kind & SubkindOf (Core)
/// C.3.3 - KindBridge & CL^k — Cross‑context Mapping of Kinds

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::assurance::CongruenceLevel;

/// C.3.1 U.Kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kind {
    pub id: String,
    pub intent: String, // Intentional definition (predicates)
    pub extent_ids: Vec<String>, // Extential definition (members)
    pub parent_kind_id: Option<String>,
}

/// C.3.2 KindSignature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindSignature {
    pub kind_id: String,
    pub formality_level: super::assurance::Formality,
    pub attributes: HashMap<String, String>,
}

/// C.3.3 KindBridge & CL^k
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindBridge {
    pub source_kind_id: String,
    pub target_kind_id: String,
    pub congruence_level: CongruenceLevel,
    pub mapping_logic: String,
}

pub struct KindCAL;

impl KindCAL {
    pub fn is_subkind(sub: &Kind, parent: &Kind) -> bool {
        sub.parent_kind_id.as_ref() == Some(&parent.id)
    }
}
