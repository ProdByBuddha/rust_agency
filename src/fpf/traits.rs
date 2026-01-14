/// A.7 Strict Distinction (Clarity Lattice) & A.9 Cross-Scale Consistency
/// 
/// Orthogonal characteristics and aggregation invariants.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SenseFamily {
    Role,
    Status,
    Measurement,
    TypeStructure,
    Method,
    Execution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferencePlane {
    World,      // External/Physical
    Conceptual, // Definition
    Epistemic,  // About a claim
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IDSLayer {
    Intension,
    Description,
    Specification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DesignRunTag {
    Design,
    Run,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PublicationSurface {
    PublicationSurface,
    InteropSurface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Face {
    PlainView,
    TechCard,
    NormsCard,
    AssuranceLane,
}

/// A.9 Invariant Quintet (S-O-L-I-D)
/// 
/// Aggregation operators must preserve these.
pub trait InvariantQuintet {
    /// Idempotence: Folding a singleton changes nothing.
    fn idempotence(&self);
    /// Local Commutativity: Order of independent folds is irrelevant.
    fn commutativity(&self);
    /// Locality: Worker or partition choice cannot affect result.
    fn locality(&self);
    /// Weakest-Link Bound: Whole never outperforms its frailest part.
    fn weakest_link(&self);
    /// Monotonicity: Improving a part cannot worsen the whole.
    fn monotonicity(&self);
}
