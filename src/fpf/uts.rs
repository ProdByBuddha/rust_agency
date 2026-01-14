/// Part F — The Unification Suite (U‑Suite): Concept‑Sets, SenseCells & Contextual Role Assignment
/// 
/// F.0.1 Contextual Lexicon Principles
/// F.17 Unified Term Survey (UTS)

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// F.0.1:3.1 Context Card — Terse descriptor for a BoundedContext
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCard {
    pub id: String,
    pub title: String,
    pub edition: String,
    pub family: String,
    pub scope_gist: String,
    pub time_stance: Option<TimeStance>,
    pub trip_wires: Vec<String>,
    pub d_sig: [String; 5], // [Sector, Function, Archetype, Regime, MetricFamily]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeStance {
    Design,
    Run,
}

/// F.0.1:3.2 SenseCell — Unit of local meaning inside one context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenseCell {
    pub context_id: String,
    pub tech_label: String,
    pub plain_label: String,
    pub gloss: String,
    pub sense_family: SenseFamily,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SenseFamily {
    Role,
    Status,
    Measurement,
    TypeStructure,
    Method,
    Execution,
}

/// F.7 Concept-Set — Collection of bridged SenseCells that are "the same thing"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptSet {
    pub id: String,
    pub u_type: String, // FPF U.Type
    pub tech_name: String,
    pub plain_name: String,
    pub description: String,
    pub cells: Vec<SenseCell>, // Cells from different contexts
    pub rationale: String,
    
    // F.17:6.1 NQD Fields (optional)
    pub nqd: Option<UtsNqd>,
    
    // F.17:6.2 Autonomy Fields (optional)
    pub autonomy: Option<UtsAutonomy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtsNqd {
    pub novelty: String,
    pub use_value: String,
    pub constraint_fit: String,
    pub portfolio_diversity: String,
    pub policy_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtsAutonomy {
    pub budget_decl_ref: String,
    pub guard_policy_id: String,
    pub override_protocol_ref: String,
    pub scope_g: String,
    pub gamma_time: String,
}

/// F.17 Unified Term Sheet (UTS)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTS {
    pub id: String,
    pub context_cards: HashMap<String, ContextCard>,
    pub concept_sets: Vec<ConceptSet>,
    pub block_plan: Vec<String>, // Sequence of block names
}

pub struct UTSManager;

impl UTS {
    pub fn find_cell(&self, context_id: &str, tech_label: &str) -> Option<&SenseCell> {
        for cs in &self.concept_sets {
            for cell in &cs.cells {
                if cell.context_id == context_id && cell.tech_label == tech_label {
                    return Some(cell);
                }
            }
        }
        None
    }
}