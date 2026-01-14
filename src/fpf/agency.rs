/// A.13 The Agential Role & Agency Spectrum
/// 
/// Graded agency via Agency-CHR.

use serde::{Serialize, Deserialize};

/// Agency Grade: Didactic Layer (0-4)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum AgencyGrade {
    NonAgential = 0,
    Reactive = 1,
    Predictive = 2,
    Adaptive = 3,
    ReflectiveStrategic = 4,
}

/// Agency-CHR: Graded Agency Characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgencyCHR {
    /// Boundary Maintenance Capacity (BMC)
    pub bmc: f64,
    /// Predictive Horizon (PH)
    pub ph: f64,
    /// Model Plasticity (MP)
    pub mp: f64,
    /// Policy Enactment Reliability (PER)
    pub per: f64,
    /// Objective Complexity (OC)
    pub oc: f64,
}

impl AgencyGrade {
    pub fn from_chr(chr: &AgencyCHR) -> Self {
        // Simplified mapping logic
        if chr.mp > 0.5 && chr.ph > 0.5 && chr.oc > 0.5 {
            AgencyGrade::ReflectiveStrategic
        } else if chr.mp > 0.0 && chr.ph > 0.0 {
            AgencyGrade::Adaptive
        } else if chr.ph > 0.0 {
            AgencyGrade::Predictive
        } else if chr.bmc > 0.0 {
            AgencyGrade::Reactive
        } else {
            AgencyGrade::NonAgential
        }
    }
}
