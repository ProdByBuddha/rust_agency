/// A.2.2 U.Capability: System Ability
/// 
/// "Can do (within its WorkScope and measures)"
/// Dispositional property of a U.System.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::role::Window;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub holder_id: String,
    pub task_family: String,           // Reference to MethodDescription or family
    pub work_scope: WorkScope,         // Conditions/Assumptions
    pub work_measures: WorkMeasures,   // Performance targets
    pub qualification_window: Window,  // Time policy
}

/// A.2.6 Unified Scope Mechanism (USM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkScope {
    pub context_slices: Vec<String>,   // Set of conditions under which capability works
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkMeasures {
    pub characteristics: HashMap<String, MeasureValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureValue {
    pub value: f64,
    pub unit: String,
    pub scale_kind: String, // Ordinal, Interval, Ratio
}
