/// A.2.5 U.RoleStateGraph: The Named State Space of a Role
/// 
/// "The gate between assignment and action."
/// Governs role state transitions and enactment gating.

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleStateGraph {
    pub states: HashMap<String, RoleState>,
    pub transitions: HashSet<(String, String)>,
    pub initial_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleState {
    pub name: String,
    pub is_enactable: bool,
    pub checklist: Vec<String>,
}

/// A StateAssertion is the verdict of an evaluation against a state's checklist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateAssertion {
    pub role_assignment_id: String,
    pub state_name: String,
    pub verdict: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub rationale: String,
}

impl RoleStateGraph {
    pub fn new(initial_state: &str) -> Self {
        Self {
            states: HashMap::new(),
            transitions: HashSet::new(),
            initial_state: initial_state.to_string(),
        }
    }

    pub fn add_state(&mut self, state: RoleState) {
        self.states.insert(state.name.clone(), state);
    }

    pub fn add_transition(&mut self, from: &str, to: &str) {
        self.transitions.insert((from.to_string(), to.to_string()));
    }

    pub fn is_transition_valid(&self, from: &str, to: &str) -> bool {
        self.transitions.contains(&(from.to_string(), to.to_string()))
    }

    pub fn is_enactable(&self, state_name: &str) -> bool {
        self.states.get(state_name).map_or(false, |s| s.is_enactable)
    }
}
