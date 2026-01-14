/// A.21 GateProfilization: OperationalGate(profile)
/// 
/// Aggregates GateChecks (CV + GF) into a GateDecision.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalGate {
    pub id: String,
    pub profile_id: String,
    pub checks: Vec<GateCheckRef>,
    pub decision: GateDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheckRef {
    pub aspect: CheckAspect,
    pub kind: String,
    pub edition: String,
    pub scope: CheckScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckAspect {
    ConstraintValidity, // CV
    GateFit,            // GF
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckScope {
    Lane,
    Locus,
    Subflow,
    Profile,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub enum GateDecision {
    Abstain = 0,
    Pass = 1,
    Degrade = 2,
    Block = 3,
}

impl GateDecision {
    /// Join-semilattice aggregation (worst wins)
    pub fn join(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLog {
    pub gate_id: String,
    pub path_slice_id: String,
    pub entries: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub check_kind: String,
    pub outcome: GateDecision,
    pub rationale: String,
}
