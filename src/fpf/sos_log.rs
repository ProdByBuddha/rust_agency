/// C.23 - MethodFamily Evidence & Maturity (Method‑SoS‑LOG)
/// 
/// Deductive shells for admissibility: Admit, Degrade, Abstain.

use serde::{Serialize, Deserialize};
use super::task_signature::TaskSignature;

/// C.23:4.1 MethodFamily
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodFamily {
    pub id: String,
    pub home_context_id: String,
    pub eligibility_predicates: Vec<String>,
}

/// C.23:4.3 MaturityCard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaturityCard {
    pub family_id: String,
    pub rung: MaturityRung,
    pub evidence_graph_path_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MaturityRung {
    L0Anecdotal,
    L1WorkedExamples,
    L2Replicated,
    L3BenchmarkSevere,
    L4QDHardened,
}

/// C.23:4.2 SoS-LOG Rule Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdmissibilityVerdict {
    Admit,
    Degrade { mode: DegradeMode, rationale: String },
    Abstain { rationale: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DegradeMode {
    ScopeNarrow,
    Sandbox,
    ProbeOnly,
}

/// C.23:4.5 Admissibility Ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissibilityLedgerEntry {
    pub family_id: String,
    pub rule_id: String,
    pub maturity_rung: MaturityRung,
    pub verdict: AdmissibilityVerdict,
    pub edition: String,
}

pub struct SoSLOG;

impl SoSLOG {
    pub fn deduce(
        _family: &MethodFamily,
        maturity: &MaturityCard,
        _signature: &TaskSignature,
    ) -> AdmissibilityVerdict {
        // R0: CG-Spec gate (Simplified placeholder)
        // R1: Admit logic
        if maturity.rung >= MaturityRung::L2Replicated {
            AdmissibilityVerdict::Admit
        } else if maturity.rung >= MaturityRung::L1WorkedExamples {
            AdmissibilityVerdict::Degrade { 
                mode: DegradeMode::Sandbox,
                rationale: "Method family only at L1 maturity".to_string()
            }
        } else {
            AdmissibilityVerdict::Abstain {
                rationale: "Method family maturity too low".to_string()
            }
        }
    }
}