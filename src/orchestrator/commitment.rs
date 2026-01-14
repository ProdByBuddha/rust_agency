use serde::{Deserialize, Serialize};

/// FPF-aligned Deontic Commitment (A.2.8)
/// 
/// Normalizes MUST/SHALL/SHOULD/MAY as lintable objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub description: String,
    pub modality: Modality,
    pub source_id: String, // Reference to the Requirement/Standard
    pub status: CommitmentStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Modality {
    /// MUST / SHALL: Non-negotiable requirement.
    Must,
    /// SHOULD: Strong recommendation.
    Should,
    /// MAY: Explicit permission.
    May,
    /// FORBIDDEN: Strict prohibition.
    Forbidden,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CommitmentStatus {
    Pending,
    Satisfied,
    Violated,
    Waivered,
}

impl Commitment {
    pub fn new(desc: impl Into<String>, modality: Modality, source: impl Into<String>) -> Self {
        Self {
            description: desc.into(),
            modality,
            source_id: source.into(),
            status: CommitmentStatus::Pending,
        }
    }

    pub fn format_for_audit(&self) -> String {
        format!(
            "COMMITMENT: [{:?}] {} (Source: {}) -> STATUS: {:?}",
            self.modality, self.description, self.source_id, self.status
        )
    }
}

