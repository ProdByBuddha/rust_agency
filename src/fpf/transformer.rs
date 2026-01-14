/// Cluster A.II - Transformation Engine
/// 
/// A.3 Transformer Constitution (Quartet): System-in-Role, MethodDescription, Method, Work.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::role::Window;

/// A.3.1 U.Method: The Abstract Way of Doing
/// 
/// "The abstract process itself."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    pub id: String,
    pub description_id: String, // Reference to MethodDescription
    pub input_signature: HashMap<String, String>,
    pub output_signature: HashMap<String, String>,
}

/// A.3.2 U.MethodDescription: The Recipe for Action
/// 
/// "SOP, code, model, epistemic artifact."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDescription {
    pub id: String,
    pub content: String,
    pub version: String,
    pub required_roles: Vec<String>,
}

/// A.15.1 U.Work: The Record of Occurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Work {
    pub id: String,
    pub method_id: String,
    pub performer_assignment_id: String,
    pub window: Window,
    pub actual_inputs: HashMap<String, String>,
    pub actual_outputs: HashMap<String, String>,
    pub resource_deltas: HashMap<String, f64>,
}

/// A.3.3 U.Dynamics: The Law of Change
/// 
/// "State evolution model."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dynamics {
    pub id: String,
    pub state_space_id: String,
    pub transition_rules: String,
}
