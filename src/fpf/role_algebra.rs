/// A.2.7 U.RoleAlgebra: In-Context Role Relations
/// 
/// Relations: Specialization (≤), Incompatibility (⊥), Bundles (⊗)

use serde::{Serialize, Deserialize};
use std::collections::{HashSet, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoleAlgebra {
    pub context_id: String,
    pub specializations: HashSet<(String, String)>, // (Child, Parent)
    pub incompatibilities: HashSet<(String, String)>, // (A, B)
    pub bundles: HashMap<String, HashSet<String>>, // BundleName -> Set of Roles
}

impl RoleAlgebra {
    pub fn new(context_id: &str) -> Self {
        Self {
            context_id: context_id.to_string(),
            specializations: HashSet::new(),
            incompatibilities: HashSet::new(),
            bundles: HashMap::new(),
        }
    }

    /// RoleS ≤ RoleG
    pub fn add_specialization(&mut self, child: &str, parent: &str) {
        self.specializations.insert((child.to_string(), parent.to_string()));
    }

    /// RoleA ⊥ RoleB
    pub fn add_incompatibility(&mut self, a: &str, b: &str) {
        self.incompatibilities.insert((a.to_string(), b.to_string()));
        self.incompatibilities.insert((b.to_string(), a.to_string()));
    }

    /// BundleName = RoleA ⊗ RoleB ⊗ ...
    pub fn add_bundle(&mut self, name: &str, roles: HashSet<String>) {
        self.bundles.insert(name.to_string(), roles);
    }

    pub fn satisfies(&self, role_s: &str, role_g: &str) -> bool {
        if role_s == role_g { return true; }
        // Simple one-level check, could be recursive
        self.specializations.contains(&(role_s.to_string(), role_g.to_string()))
    }

    pub fn is_incompatible(&self, a: &str, b: &str) -> bool {
        self.incompatibilities.contains(&(a.to_string(), b.to_string()))
    }
}
