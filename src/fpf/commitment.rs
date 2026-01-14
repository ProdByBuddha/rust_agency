/// A.2.8 U.Commitment: Deontic Commitment Object
/// 
/// "How to represent MUST/SHALL as a lintable object?"
/// BCP‑14 (RFC 2119/8174) alignment.

use serde::{Serialize, Deserialize};
use super::role::Window;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub id: String,
    pub modality: Modality,
    pub scope_id: String,              // USM scope
    pub validity_window: Window,
    pub description: String,
    pub evidence_refs: Vec<String>,    // IDs of Episteme (EvidenceRole)
    pub status: CommitmentStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Modality {
    Must,      // SHALL
    MustNot,   // SHALL NOT
    Should,
    ShouldNot,
    May,
    Optional,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CommitmentStatus {
    Open,
    AdjudicatedPass,
    AdjudicatedFail,
    Waivered,
    Expired,
}

/// A.2.9 U.SpeechAct: Communicative Work Object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechAct {
    pub id: String,
    pub kind: SpeechActKind,
    pub performer_assignment_id: String,
    pub utterance: String,
    pub institutes_commitment_id: Option<String>, // Linking act to commitment
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeechActKind {
    Approval,
    Authorization,
    Publication,
    Revocation,
    Declaration,
}
