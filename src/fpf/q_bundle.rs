/// C.25 - Q-Bundle: Authoring “-ilities” as Structured Quality Bundles
/// 
/// Definitional pattern reusing A.2.6 USM, A.6.1 Mechanism, and C.16 MM-CHR.

use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, HashSet};
use super::mm_chr::Measure;

/// A.2.6:6.1 U.ContextSlice — where scope is evaluated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ContextSlice {
    pub context_id: String,
    pub standard_versions: BTreeMap<String, String>,
    pub environment_selectors: BTreeMap<String, String>,
    pub gamma_time: String, // Mandatory time selector/policy
}

/// A.2.6:6.2 U.Scope — set-valued scope object over U.ContextSlice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub slices: HashSet<ContextSlice>,
}

impl Scope {
    pub fn covers(&self, slice: &ContextSlice) -> bool {
        // Primitive membership check
        self.slices.contains(slice)
    }

    pub fn covers_set(&self, target: &HashSet<ContextSlice>) -> bool {
        // Subset relation
        target.is_subset(&self.slices)
    }
}

/// C.25:4 Q-Bundle Normal Form
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QBundle {
    pub name: String,
    pub carrier_id: String, // Holon ID
    pub claim_scope: Option<Scope>,
    pub work_scope: Option<Scope>,
    pub measures: Vec<Measure>,
    pub qualification_window: Option<String>, // Time policy id
    pub mechanisms: Vec<String>, // Mechanism IDs
    pub status: String,
    pub evidence: Vec<String>, // EvidenceStub IDs
}

pub struct QBundleCAL;

impl QBundleCAL {
    pub fn verify_admissibility(bundle: &QBundle, target_slice: &ContextSlice) -> bool {
        // C.25:4 - Admissibility check
        // Scope covers TargetSlice AND (Measures satisfied - placeholder)
        
        let scope_ok = if let Some(ref scope) = bundle.work_scope {
            scope.covers(target_slice)
        } else if let Some(ref scope) = bundle.claim_scope {
            scope.covers(target_slice)
        } else {
            true // No scope declared
        };

        // In a full implementation, we would check if measures satisfy thresholds in target_slice
        scope_ok
    }
}