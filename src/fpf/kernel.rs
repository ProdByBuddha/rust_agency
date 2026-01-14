/// Cluster A.IV - Kernel Modularity
/// 
/// A.5 Open-Ended Kernel: micro-kernel architecture for thought.
/// A.6 Signature Stack: Universal, law‑governed declaration.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// A.6.0 U.Signature: Universal, law‑governed declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub id: String,
    pub vocabulary: Vec<String>,
    pub laws: Vec<String>,
    pub context_id: String,
}

/// A.6.1 U.Mechanism: Law-governed application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mechanism {
    pub id: String,
    pub signature: Signature,
    pub admissibility_conditions: Vec<String>,
    pub transport_bridge_id: Option<String>,
    pub gamma_time_policy: String,
}

/// A.6.5 U.RelationSlotDiscipline: SlotSpec triple
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotSpec {
    pub slot_kind: String, // Tech name with *Slot suffix
    pub value_kind: String, // U.Type or U.Kind
    pub ref_mode: RefMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefMode {
    ByValue,
    RefKind(RefKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefKind {
    EntityRef,
    HolonRef,
    MethodRef,
    EpistemeRef,
    ViewpointRef,
}

/// A.6.6 U.ScopedWitnessedBaseDeclaration (SWBD)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SWBD {
    pub dependent_id: String,
    pub base_id: String,
    pub base_relation: String, // Vocabulary token
    pub scope_id: String,      // U.Scope ID
    pub gamma_time: Option<String>,
    pub witness_ids: Vec<String>,
}

/// A.6.7 MechSuiteDescription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechSuiteDescription {
    pub id: String,
    pub mechanism_ids: Vec<String>,
    pub suite_obligations: Vec<String>,
    pub suite_contract_pins: Vec<String>,
}

/// A.6.P Relational Precision Restoration (RPR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationKind {
    pub id: String,
    pub polarity: String,
    pub slot_specs: Vec<SlotSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualifiedRelationRecord {
    pub kind_id: String, // Reference to RelationKind
    pub participant_ids: Vec<String>,
    pub scope_id: Option<String>,
    pub gamma_time: Option<String>,
    pub witness_ids: Vec<String>,
}

/// A.6.A Architheory Signature & Realization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Architheory {
    pub id: String,
    pub signature: Signature,
    pub invariants: Vec<String>,
}

/// A.6.B Boundary Norm Square
/// Laws / Admissibility / Deontics / Work-Effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormSquare {
    pub laws: Vec<String>,          // Physical/Logical invariants
    pub admissibility: Vec<String>, // Gates/Conditions
    pub deontics: Vec<String>,      // Commitments/Obligations
    pub work_effects: Vec<String>,  // Traceable outcomes
}

/// A.5 Kernel Registry for Architheories
pub struct Kernel {
    pub architheories: HashMap<String, Architheory>,
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            architheories: HashMap::new(),
        }
    }

    pub fn register_architheory(&mut self, theory: Architheory) {
        self.architheories.insert(theory.id.clone(), theory);
    }
}
