/// G.12 - DHC Dashboards - Discipline‑Health Time‑Series
/// 
/// Lawful, reproducible, refresh‑aware dashboard series.

use serde::{Serialize, Deserialize};
use super::mm_chr::ScaleType;

/// G.12:4.1 DHCSeries@Context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DHCSeries {
    pub id: String,
    pub context_id: String,
    pub discipline_id: String,
    pub slots: Vec<DHCSlot>,
    pub edition_pins: DashboardEditionPins,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DHCSlot {
    pub id: String,
    pub scale_type: ScaleType,
    pub reference_plane: String,
    pub gamma_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardEditionPins {
    pub method_ref_edition: String,
    pub spec_ref_edition: String,
    pub distance_def_edition: String,
}

/// G.12:4.2 Dashboard Slice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSlice {
    pub series_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub row_values: std::collections::HashMap<String, f64>,
    pub path_ids: Vec<String>,
}

pub struct DHCDashboard;

impl DHCSeries {
    pub fn new(id: &str, discipline_id: &str) -> Self {
        DHCSeries {
            id: id.to_string(),
            context_id: "".to_string(),
            discipline_id: discipline_id.to_string(),
            slots: vec![],
            edition_pins: DashboardEditionPins {
                method_ref_edition: "v1".to_string(),
                spec_ref_edition: "v1".to_string(),
                distance_def_edition: "v1".to_string(),
            },
        }
    }
}