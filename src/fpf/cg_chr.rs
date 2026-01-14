/// G.3 - CHR Authoring: Characteristics - Scales - Levels - Coordinates
/// 
/// Notation-independent authoring discipline for typed characterisation.

use serde::{Serialize, Deserialize};
use super::mm_chr::{ScaleType, Polarity};

/// G.3:5 S3 - Characteristic Card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacteristicCard {
    pub id: String,
    pub context_id: String,
    pub reference_plane: String,
    pub object_kind: String,
    pub intent: String,
    pub scale_type: ScaleType,
    pub polarity: Polarity,
    pub unit_set: Vec<String>,
    pub freshness_half_life_ms: u64,
    pub missingness_semantics: String, // MCAR, MAR, MNAR
    pub qd_role: QDRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QDRole {
    Q,       // Quality
    D,       // Diversity
    QDScore, // Combined
    None,
}

/// G.3:5 S4 - Scale and Level Cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleCard {
    pub scale_type: ScaleType,
    pub admissible_transforms: Vec<String>,
    pub resolution: f64,
    pub bounds: (Option<f64>, Option<f64>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelCard {
    pub enumeration: Vec<String>,
    pub is_total_order: bool,
}

/// G.3:5 S6 - Legality Matrix & Guard Macros
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalityMatrix {
    pub scale_type: ScaleType,
    pub allowed_ops: Vec<String>, // e.g., "mean", "median", "sum"
}

pub struct CHRPackAuthoring;

impl CharacteristicCard {
    pub fn verify_legality(&self, op: &str) -> bool {
        // G.3:9 CC-G.3-8 - No illegal ops (e.g. mean on ordinal)
        if self.scale_type == ScaleType::Ordinal && op == "mean" {
            return false;
        }
        true
    }
}