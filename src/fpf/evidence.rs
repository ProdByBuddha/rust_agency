/// A.2.4 U.EvidenceRole: The Evidential Stance
/// 
/// "How an episteme serves as evidence for a claim."
/// Non-behavioural role enacted via U.RoleAssignment.

use serde::{Serialize, Deserialize};
use super::role::Window;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRole {
    pub id: String,
    pub target_claim_id: String,
    pub claim_scope: String,           // G in F-G-R
    pub relevance_window: Window,
}

/// A.10 Evidence Graph Referring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceAnchor {
    pub episteme_id: String,           // The carrier
    pub role_assignment_id: String,    // Linking to EvidenceRole
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
