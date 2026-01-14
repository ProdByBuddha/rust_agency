/// E.9 - Design-Rationale Record (DRR) Method
/// 
/// Structured argument preceding every normative change.

use serde::{Serialize, Deserialize};

/// E.9:4 DRR Components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DRR {
    pub id: String,
    pub problem_frame: String, // Why are we talking about this?
    pub decision: String,      // What will we do? (Normative text)
    pub rationale: Rationale,  // Why is this the right thing?
    pub consequences: String,  // What happens next?
    pub impacted_pattern_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rationale {
    pub pillar_checks: Vec<PillarCheck>,
    pub taxonomy_lenses: Vec<TaxonomyLensResult>,
    pub alternatives_considered: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PillarCheck {
    pub pillar_id: String, // P-1 to P-11
    pub assessment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyLensResult {
    pub lens: TaxonomyLens,
    pub assessment: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxonomyLens {
    Gov,
    Arch,
    Epist,
    Prag,
    Did,
}

pub struct DRRManager;

impl DRR {
    pub fn new(id: &str) -> Self {
        DRR {
            id: id.to_string(),
            problem_frame: "".to_string(),
            decision: "".to_string(),
            rationale: Rationale {
                pillar_checks: vec![],
                taxonomy_lenses: vec![],
                alternatives_considered: vec![],
            },
            consequences: "".to_string(),
            impacted_pattern_ids: vec![],
        }
    }
}