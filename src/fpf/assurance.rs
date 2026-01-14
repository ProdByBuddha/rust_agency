/// B.3 - Trust & Assurance Calculus (F–G–R with Congruence)
/// 
/// F: Formality (Ordinal)
/// G: ClaimScope (SpanUnion)
/// R: Reliability (Ratio)
/// CL: Congruence Level (Ordinal)

use serde::{Serialize, Deserialize};
use std::collections::HashSet;

/// B.3:4.1.1 Formality (F) - Ordinal scale
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum Formality {
    F0Informal,
    F1Structured,
    F2Formalizable,
    F3MachineCheckable,
    F4VerifiedSpecification,
    F5RigorousProof,
    F6FullyFormal,
    F7MechanicalVerification,
    F8VerifiedCompiler,
    F9Foundational,
}

/// B.3:4.1.2 ClaimScope (G) - SpanUnion/Coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimScope {
    pub span: HashSet<String>, // Domain identifiers or coordinates
    pub description: String,
}

/// B.3:4.1.3 Reliability (R) - Ratio scale [0, 1]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Reliability(pub f64);

impl Reliability {
    pub fn new(val: f64) -> Self {
        Reliability(val.clamp(0.0, 1.0))
    }
}

/// B.3:4.1.4 Congruence Level (CL) - Ordinal scale
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum CongruenceLevel {
    CL0WeakGuess,
    CL1Plausible,
    CL2Validated,
    CL3Verified,
}

/// B.3:4.2 Assurance Tuple
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssuranceTuple {
    pub formality: Formality,
    pub scope: ClaimScope,
    pub reliability: Reliability,
    pub notes: String,
}

/// B.3:4.4.3 Penalty Function Φ(CL)
pub fn phi_penalty(cl: CongruenceLevel) -> f64 {
    match cl {
        CongruenceLevel::CL0WeakGuess => 0.5,
        CongruenceLevel::CL1Plausible => 0.2,
        CongruenceLevel::CL2Validated => 0.05,
        CongruenceLevel::CL3Verified => 0.0,
    }
}

impl AssuranceTuple {
    /// Skeleton for aggregating two assurance tuples
    pub fn aggregate(a: &Self, b: &Self, cl_ab: CongruenceLevel) -> Self {
        // F_eff = min(F_i)
        let formality = std::cmp::min(a.formality, b.formality);
        
        // G_eff = SpanUnion({G_i})
        let mut span = a.scope.span.clone();
        span.extend(b.scope.span.clone());
        let scope = ClaimScope {
            span,
            description: format!("Union of {} and {}", a.scope.description, b.scope.description),
        };
        
        // R_raw = min(R_i)
        let r_raw = a.reliability.0.min(b.reliability.0);
        // R_eff = max(0, R_raw - Phi(CL_min))
        let r_eff = (r_raw - phi_penalty(cl_ab)).max(0.0);
        
        AssuranceTuple {
            formality,
            scope,
            reliability: Reliability::new(r_eff),
            notes: "Aggregated tuple".to_string(),
        }
    }
}
