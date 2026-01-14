use serde::{Deserialize, Serialize};


/// FPF-aligned U.Kind (C.3.1)
/// 
/// Provides a formal type system for reasoning. 
/// Kinds define the 'Nature' of an entity or claim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Technical artifacts (Code, Build Logs, Specs)
    Technical,
    /// Physical evidence (File existence, Web search results)
    Evidence,
    /// Strategic intents (Objectives, Missions)
    Strategic,
    /// Operational traces (WorkRecords, Telemetry)
    Operational,
    /// Theoretical claims (LLM guesses, Unsubstantiated thoughts)
    Theoretical,
    /// Governance artifacts (Bridges, NormSquares, Adjudications)
    Governance,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Technical => "Technical",
            Self::Evidence => "Evidence",
            Self::Strategic => "Strategic",
            Self::Operational => "Operational",
            Self::Theoretical => "Theoretical",
            Self::Governance => "Governance",
        }
    }

    /// FPF Standard: Detect kind based on content cues
    pub fn detect(content: &str) -> Self {
        let lower = content.to_lowercase();
        if lower.contains("fn ") || lower.contains("pub struct") || lower.contains("import ") {
            Self::Technical
        } else if lower.contains("verified") || lower.contains("carrier") || lower.contains("observation") {
            Self::Evidence
        } else if lower.contains("goal") || lower.contains("mission") || lower.contains("objective") {
            Self::Strategic
        } else if lower.contains("norm") || lower.contains("bridge") || lower.contains("adjudication") {
            Self::Governance
        } else {
            Self::Theoretical
        }
    }
}

/// FPF-aligned Kind-Hierarchy (C.3.1 SubkindOf)
pub struct KindAlgebra;

impl KindAlgebra {
    /// Check if child is a SubkindOf parent (≤)
    pub fn satisfies(child: &Kind, parent: &Kind) -> bool {
        if child == parent { return true; }
        
        // FPF Standard Hierarchy
        match (child, parent) {
            (Kind::Evidence, Kind::Technical) => true, // Evidence can be code
            (Kind::Governance, Kind::Operational) => true, 
            _ => false,
        }
    }
}
