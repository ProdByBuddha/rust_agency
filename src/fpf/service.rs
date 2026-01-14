/// A.2.3 U.ServiceClause: The Service Promise Clause
/// 
/// "Promise content — a consumer‑facing promise statement."
/// Distinguished from U.System, U.RoleAssignment, U.MethodDescription, and U.Work.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceClause {
    pub id: String,
    pub provider_role_id: String,
    pub consumer_role_id: Option<String>,
    pub promise_content: String,
    pub access_spec: AccessSpec,
    pub acceptance_spec: AcceptanceSpec,
    pub slo: Option<SLO>,
    pub sla: Option<SLA>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessSpec {
    pub endpoint: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceSpec {
    pub criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLO {
    pub metric: String,
    pub target: f64,
    pub window_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLA {
    pub slo_ref: String,
    pub penalty: String,
}

/// A.6.8 Service Situation Bundle (RPR-SERV)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSituation {
    pub id: String,
    pub clause_id: String,
    pub provider_principal_id: String,
    pub access_point_id: String,
    pub delivery_system_id: String,
    pub access_spec_id: String,
    pub commitment_id: Option<String>,
    pub promise_act_id: Option<String>,
    pub work_id: Option<String>,
}

/// A.6.C Contract Bundle Unpacking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractBundle {
    pub id: String,
    pub clause_ids: Vec<String>,
    pub commitment_ids: Vec<String>,
    pub utterance_ids: Vec<String>, // Speech acts/publications
    pub work_evidence_ids: Vec<String>,
}
