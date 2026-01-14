/// Cluster A.III - Time & Evolution
/// 
/// A.4 Temporal Duality: design-time vs run-time.
/// P-10 Open-Ended Evolution: systems are perpetually incomplete.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionPrinciple {
    pub id: String,
    pub description: String,
}

/// Design-Time Stance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignState {
    pub holon_id: String,
    pub spec_id: String,
    pub timestamp: DateTime<Utc>,
}

/// Run-Time Stance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub holon_id: String,
    pub actual_work_id: String,
    pub timestamp: DateTime<Utc>,
}

/// B.4 Canonical Evolution Loop
/// Run -> Observe -> Refine -> Deploy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionLoop {
    pub id: String,
    pub current_phase: EvolutionPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionPhase {
    Run,
    Observe,
    Refine,
    Deploy,
}
