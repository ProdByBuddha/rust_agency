/// A.1.1 U.BoundedContext: The Semantic Frame
/// 
/// Localizes meaning, roles, and invariants.
/// "In which semantic frame does this term, rule, or role-claim hold?"

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use super::holon::{Holon, Entity, Boundary};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundedContext {
    pub id: String,
    pub boundary: Boundary,
    pub glossary: HashMap<String, String>,      // Local Lexicon
    pub invariants: HashSet<String>,            // Local Rules/Constraints
    pub roles: HashSet<String>,                 // Local Role Taxonomy
    pub bridges: Vec<Bridge>,                   // Cross-context alignments
}

impl Entity for BoundedContext {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Holon for BoundedContext {
    fn boundary(&self) -> &Boundary {
        &self.boundary
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bridge {
    pub target_context_id: String,
    pub congruence_level: f32, // CL: 0.0 to 1.0
    pub loss_notes: String,
}

impl BoundedContext {
    pub fn new(id: &str, boundary: Boundary) -> Self {
        Self {
            id: id.to_string(),
            boundary,
            glossary: HashMap::new(),
            invariants: HashSet::new(),
            roles: HashSet::new(),
            bridges: Vec::new(),
        }
    }

    pub fn with_role(mut self, role: &str) -> Self {
        self.roles.insert(role.to_string());
        self
    }

    pub fn with_invariant(mut self, invariant: &str) -> Self {
        self.invariants.insert(invariant.to_string());
        self
    }
}
