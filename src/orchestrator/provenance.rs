use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// FPF-aligned Evidence Graph & Provenance Ledger (G.6)
/// 
/// Traces claims back to their physical Evidence Carriers (Tool Outputs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGraph {
    pub path_id: String,
    /// Mapping of Claim ID -> Evidence Carrier (Tool Summary or File Path)
    pub evidence_map: HashMap<String, String>,
}

impl EvidenceGraph {
    pub fn new() -> Self {
        Self {
            path_id: Uuid::new_v4().to_string(),
            evidence_map: HashMap::new(),
        }
    }

    pub fn record_evidence(&mut self, claim: impl Into<String>, carrier: impl Into<String>) {
        self.evidence_map.insert(claim.into(), carrier.into());
    }

    pub fn format_for_audit(&self) -> String {
        let mut output = format!("EVIDENCE GRAPH (Path: {})
", self.path_id);
        if self.evidence_map.is_empty() {
            output.push_str("  - No physical evidence carriers recorded.");
        } else {
            for (claim, carrier) in &self.evidence_map {
                output.push_str(&format!("  - Claim: '{}' -> Carrier: '{}'\n", claim, carrier));
            }
        }
        output
    }
}


