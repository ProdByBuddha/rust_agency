/// A.15.2 U.WorkPlan: The Schedule of Intent
/// 
/// "When, by whom in intent, under which constraints."

use serde::{Serialize, Deserialize};
use super::role::Window;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPlan {
    pub id: String,
    pub context_id: String,
    pub items: Vec<PlanItem>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub id: String,
    pub method_id: String,
    pub planned_window: Window,
    pub required_roles: Vec<String>,
    pub proposed_performer_id: Option<String>,
    pub budget_reservations: Vec<ResourceReservation>,
    pub dependencies: Vec<String>, // IDs of other PlanItems
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReservation {
    pub resource_kind: String,
    pub amount: f64,
    pub unit: String,
}
