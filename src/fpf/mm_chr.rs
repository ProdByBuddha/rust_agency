/// C.16 - Measurement & Metrics Characterization (MM-CHR)
/// 
/// Exports the measurement substrate: U.DHCMethod, U.Measure, U.Unit, U.EvidenceStub.
/// Disciplined by CSLC (Characteristic, Scale, Level, Coordinate).

use serde::{Serialize, Deserialize};
use super::assurance::Reliability;

/// C.16:5.3.1 U.DHCMethod — The metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DHCMethod {
    pub id: String,
    pub characteristic_id: String,
    pub scale_type: ScaleType,
    pub unit: Option<Unit>,
    pub polarity: Polarity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleType {
    Nominal,
    Ordinal,
    Interval,
    Ratio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Polarity {
    Positive, // Higher is better
    Negative, // Lower is better
    Neutral,
}

/// C.16:5.3.3 U.Unit — Semantics of quantities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Unit {
    pub name: String,
    pub symbol: String,
    pub dimension_id: String, // Linking to physical/logical dimensions
}

/// C.16:5.3.2 U.Measure — The recorded reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measure {
    pub method_id: String,
    pub coordinate: CoordinateValue,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub reliability: Reliability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinateValue {
    Scalar(f64),
    Category(String),
    Level(usize),
}

/// C.16:5.3.4 U.EvidenceStub — Pointer to grounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStub {
    pub source_id: String,
    pub uri: String,
    pub confidence_score: f64,
}

pub struct MMCHR;

impl MMCHR {
    pub fn verify_comparability(a: &DHCMethod, b: &DHCMethod) -> bool {
        // C.16:5.2 - Comparability stance
        a.characteristic_id == b.characteristic_id && a.scale_type == b.scale_type && a.unit == b.unit
    }
}