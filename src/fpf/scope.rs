/// A.2.6 Unified Scope Mechanism (USM): Context Slices & Scopes
/// 
/// "How to define the scope of a claim or capability?"
/// What is G in F-G-R?

use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USM {
    pub id: String,
    pub slices: HashSet<ContextSlice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ContextSlice {
    pub dimension: String,
    pub value: String,
}

/// G in F-G-R
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimScope {
    pub target_holon_id: String,
    pub attribute_set: HashSet<String>,
    pub context_slices: USM,
}

/// WorkScope for Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkScope {
    pub method_id: String,
    pub allowable_slices: USM,
}

impl USM {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            slices: HashSet::new(),
        }
    }

    pub fn with_slice(mut self, dimension: &str, value: &str) -> Self {
        self.slices.insert(ContextSlice {
            dimension: dimension.to_string(),
            value: value.to_string(),
        });
        self
    }

    pub fn covers(&self, other: &USM) -> bool {
        // Simple subset check for now
        other.slices.is_subset(&self.slices)
    }
}
