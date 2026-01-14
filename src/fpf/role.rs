/// A.2 Role Taxonomy & A.2.1 U.RoleAssignment
/// 
/// "A holon’s essence tells us what it is; its roles tell us what it is being, here and now."

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// U.Window: A time interval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

impl Window {
    pub fn now_open() -> Self {
        Self {
            start: Utc::now(),
            end: None,
        }
    }

    pub fn contains(&self, time: DateTime<Utc>) -> bool {
        time >= self.start && self.end.map_or(true, |e| time <= e)
    }
}

/// U.Role: A context-bound capability/obligation schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub family: RoleFamily,
    pub description: String,
    // RCS and RSG are recorded in RoleDescription (Episteme), 
    // but here we can have their identifiers or simplified versions.
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoleFamily {
    Agential,      // Agent, Decision-Maker
    Transformer,   // Welder, ETL-Runner
    Observer,      // Monitor, Sensor
    Speech,        // Authorizer, Notifier
    ServiceGovernance,
    EpistemicStatus,
    NormativeStatus,
}

/// U.RoleAssignment: Contextual Role Assignment
/// Holder#Role:Context@Window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    pub holder_id: String,
    pub role_id: String,
    pub context_id: String,
    pub window: Window,
    pub justification: Option<String>, // ID of Episteme
    pub provenance: Option<String>,    // ID of Method/Work
}

/// U.RoleEnactment: Run-time fact that Work was performed under an assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleEnactment {
    pub work_id: String,
    pub assignment_id: String, // Or the RoleAssignment itself
}

/// A.2.5 U.RoleStateGraph (RSG)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleStateGraph {
    pub states: Vec<RoleState>,
    pub transitions: Vec<(String, String)>, // (From, To)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleState {
    pub name: String,
    pub is_enactable: bool,
    pub checklist: Vec<String>,
}

/// A.2.1:4.3 Role Characterisation Space (RCS)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleCharacterisationSpace {
    pub characteristics: HashMap<String, String>,
}
