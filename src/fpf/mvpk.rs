/// E.17 - Multi-View Publication Kit (MVPK)
/// 
/// Disciplined, compositional way to publish morphisms across multiple didactic faces.

use serde::{Serialize, Deserialize};
use super::q_bundle::Scope;
use super::mm_chr::ScaleType;

/// E.17:5.0 PublicationScope (USM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationScope {
    pub id: String,
    pub scope: Scope,
}

/// E.17:5.2 Face Kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaceKind {
    PlainView,     // P: Explanatory prose
    TechCard,      // T: Typed catalog card
    InteropCard,   // I: Machine exchange
    AssuranceLane, // A: Evidence bindings
}

/// E.17:5.5 Publication Characteristic (PC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PubCharacteristic {
    /// PC.Number — numeric/comparable entries
    Number {
        value: f64,
        unit: String,
        scale: ScaleType,
        reference_plane: String,
        edition: String,
    },
    /// PC.EvidenceBinding — bindings to carriers and policies
    EvidenceBinding {
        path_slice_id: String,
        bridge_id: Option<String>,
        cl_notes: Option<String>,
    },
    /// PC.ComparatorSetRef — explicit comparator family
    ComparatorSetRef {
        id: String,
        edition: String,
    },
    /// PC.CharacteristicSpaceRef
    CharacteristicSpaceRef {
        id: String,
        edition: String,
    },
}

/// E.17:5.1 U.View (Episteme-level view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    pub id: String,
    pub morphism_id: String, // DescribedEntity
    pub kind: FaceKind,
    pub viewpoint_id: String, // PublicationVPId
    pub scope: PublicationScope,
    pub characteristics: Vec<PubCharacteristic>,
    pub underlying_episteme_ids: Vec<String>,
}

pub struct MVPK;

impl MVPK {
    pub fn emit_view(
        morphism_id: &str,
        kind: FaceKind,
        scope: PublicationScope,
        characteristics: Vec<PubCharacteristic>,
    ) -> View {
        View {
            id: format!("view_{}_{}", kind_to_prefix(kind), uuid::Uuid::new_v4()),
            morphism_id: morphism_id.to_string(),
            kind,
            viewpoint_id: format!("vp_{}", kind_to_prefix(kind)),
            scope,
            characteristics,
            underlying_episteme_ids: vec![],
        }
    }
}

fn kind_to_prefix(kind: FaceKind) -> &'static str {
    match kind {
        FaceKind::PlainView => "plain",
        FaceKind::TechCard => "tech",
        FaceKind::InteropCard => "interop",
        FaceKind::AssuranceLane => "assurance",
    }
}