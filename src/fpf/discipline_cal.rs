/// C.20 - Composition of U.Discipline (Discipline-CAL)
/// 
/// Fold canons, standards, and org-carriers into a reusable holon of talk.

use serde::{Serialize, Deserialize};
use super::holon::{Boundary, BoundaryKind};

/// C.20:4.1 U.Discipline — The discipline holon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discipline {
    pub id: String,
    pub tech_name: String,
    pub plain_name: String,
    pub canon_id: String,      // Episteme ID
    pub standards_id: String,  // Episteme ID
    pub carriers_ids: Vec<String>, // System IDs
    pub boundary: Boundary,
}

/// C.20:4.1 Tradition / Lineage — Auxiliary holons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tradition {
    pub id: String,
    pub discipline_id: String,
    pub operator_set: Vec<String>,
    pub method_family_ids: Vec<String>,
}

pub struct DisciplineCAL;

impl DisciplineCAL {
    /// C.20:4.3 Γ_disc — Discipline constructor
    pub fn compose(
        id: &str,
        tech_name: &str,
        plain_name: &str,
        canon_id: &str,
        standards_id: &str,
        carriers_ids: Vec<String>,
    ) -> Discipline {
        Discipline {
            id: id.to_string(),
            tech_name: tech_name.to_string(),
            plain_name: plain_name.to_string(),
            canon_id: canon_id.to_string(),
            standards_id: standards_id.to_string(),
            carriers_ids,
            boundary: Boundary {
                kind: BoundaryKind::Permeable,
                description: format!("Discipline boundary for {}", tech_name),
            },
        }
    }
}