/// C.2 - Epistemic holon composition (KD-CAL)
/// C.2.1 - U.Episteme — Epistemes and their slot graph
/// C.2.2 - Reliability R in the F–G–R triad

use serde::{Serialize, Deserialize};
use super::assurance::{Formality, ClaimScope, Reliability, CongruenceLevel, phi_penalty};

/// C.2.1 U.EpistemeSlotGraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemeSlotGraph {
    pub described_entity_id: String,
    pub grounding_holon_id: String,
    pub claim_graph_id: String,
    pub viewpoint_id: String,
    pub view_id: String,
    pub reference_scheme_id: String,
}

/// C.2.2 Epistemic Location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicLocation {
    pub formality: Formality,
    pub scope: ClaimScope,
    pub reliability: Reliability,
}

/// KD-CAL architheory
pub struct KDCAL;

impl KDCAL {
    /// C.2.2:4.3 Pathwise weakest-link propagation
    pub fn path_reliability(
        spine_reliabilities: &[Reliability],
        lowest_cl: CongruenceLevel,
    ) -> Reliability {
        let min_r = spine_reliabilities.iter()
            .map(|r| r.0)
            .fold(1.0, f64::min);
        
        let penalty = phi_penalty(lowest_cl);
        Reliability::new(min_r - penalty)
    }

    /// C.2.2:4.3 Parallel support (OR-style)
    pub fn parallel_reliability(path_reliabilities: &[Reliability]) -> Reliability {
        let max_r = path_reliabilities.iter()
            .map(|r| r.0)
            .fold(0.0, f64::max);
        Reliability::new(max_r)
    }
}
