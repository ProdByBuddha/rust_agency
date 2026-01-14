/// Part D – Multi-scale Ethics & Conflict-Optimisation
/// 
/// D.1 Axiological Neutrality Principle
/// D.2 Multi-Scale Ethics Framework
/// D.3 Holonic Conflict Topology
/// D.5 Bias-Audit Cycle

use serde::{Serialize, Deserialize};

/// D.1 Axiological Neutrality - Preferences as lattices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceLattice {
    pub id: String,
    pub nodes: Vec<String>, // Value identifiers
    pub edges: Vec<(String, String)>, // (greater, lesser)
}

/// D.2 Multi-Scale Ethics Framework
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EthicalScale {
    L0Self,
    L1Team,
    L2Ecosystem,
    L3Planet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalDuty {
    pub id: String,
    pub scale: EthicalScale,
    pub description: String,
    pub priority: usize,
}

/// D.3 Holonic Conflict Topology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    Resource,
    Goal,
    Epistemic,
    Temporal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub id: String,
    pub conflict_type: ConflictType,
    pub participants: Vec<String>, // Holon IDs
    pub status: ConflictStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStatus {
    Detected,
    Negotiating,
    Mediated,
    Resolved,
    Appealed,
}

/// D.5 Bias-Audit Cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasRegister {
    pub id: String,
    pub entries: Vec<BiasEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasEntry {
    pub code: BiasCategory,
    pub description: String,
    pub flagged_by: String, // Role/ID
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiasCategory {
    REP, // Representation
    ALG, // Algorithmic
    VIS, // Visual Framing
    MET, // Metric Proxy
    LNG, // Lexical
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasAuditReport {
    pub id: String,
    pub version: String,
    pub findings: Vec<BiasFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasFinding {
    pub bias_code: BiasCategory,
    pub severity: Severity,
    pub description: String,
    pub mitigation: String, // Proposed Method or Constraint
    pub status: FindingStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingStatus {
    Blocking,
    Resolved,
    RiskAccepted,
}

pub struct EthicsCAL;

impl EthicsCAL {
    pub fn detect_conflict(a: &EthicalDuty, b: &EthicalDuty) -> Option<ConflictRecord> {
        if a.scale == b.scale && a.priority == b.priority {
            Some(ConflictRecord {
                id: format!("conflict_{}", uuid::Uuid::new_v4()),
                conflict_type: ConflictType::Goal,
                participants: vec![a.id.clone(), b.id.clone()],
                status: ConflictStatus::Detected,
            })
        } else {
            None
        }
    }
}