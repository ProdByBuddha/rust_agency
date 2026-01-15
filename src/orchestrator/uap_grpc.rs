//! UAP gRPC Server Wrapper
//! 
//! Bridges the native `SovereignAgent` trait with the external UAP gRPC protocol.
//! "Striking down the middle": Thin translation layer, zero-cost internal logic.

use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::agent::SovereignAgent; 
use pai_core::sap::AuditStatus;

// Import the generated gRPC code
pub mod proto {
    tonic::include_proto!("uap.v1");
}

use proto::agent_service_server::AgentService;
pub use proto::agent_service_server::AgentServiceServer;

pub struct UapGrpcWrapper {
    /// The underlying native agent (e.g., ReActAgent)
    agent: Arc<dyn SovereignAgent>,
}

impl UapGrpcWrapper {
    pub fn new(agent: Arc<dyn SovereignAgent>) -> Self {
        Self { agent }
    }
}

#[tonic::async_trait]
impl AgentService for UapGrpcWrapper {
    async fn create_task(
        &self,
        request: Request<proto::CreateTaskRequest>,
    ) -> Result<Response<proto::Task>, Status> {
        let req = request.into_inner();
        
        let task = self.agent.create_task(&req.input, None)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(proto::Task {
            task_id: task.task_id,
            input: task.input,
            artifacts: vec![],
            created_at: None,
            updated_at: None,
            status: 1, // Created
            commitment: None,
            audit: None, // Will be filled in future refinement
        }))
    }

    async fn audit_alignment(
        &self,
        request: Request<proto::AuditAlignmentRequest>,
    ) -> Result<Response<proto::AlignmentAudit>, Status> {
        let req = request.into_inner();
        let audit = self.agent.audit_alignment(&req.input, req.override_lever)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(proto::AlignmentAudit {
            audit_id: audit.audit_id,
            timestamp: None,
            target_id: audit.target_id,
            score: audit.score,
            violations: audit.violations,
            status: match audit.status {
                AuditStatus::Aligned => proto::AuditStatus::Aligned as i32,
                AuditStatus::Flagged => proto::AuditStatus::Flagged as i32,
                AuditStatus::Blocked => proto::AuditStatus::Blocked as i32,
                AuditStatus::Overridden => proto::AuditStatus::Overridden as i32,
            },
        }))
    }

    async fn propose_commitment(
        &self,
        request: Request<proto::ProposeCommitmentRequest>,
    ) -> Result<Response<proto::ValueCommitment>, Status> {
        let req = request.into_inner();
        let commitment = self.agent.propose_commitment(&req.input)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(proto::ValueCommitment {
            commitment_id: commitment.commitment_id,
            task_id: commitment.task_id,
            issuer_id: commitment.issuer_id,
            created_at: None,
            status: proto::CommitmentStatus::Proposed as i32,
            modality: proto::CommitmentModality::Aspirational as i32,
            impact: Some(proto::EconomicImpact {
                labor_decoupling_score: commitment.impact.labor_decoupling_score,
                capital_generation_score: commitment.impact.capital_generation_score,
                utility_value: commitment.impact.utility_value,
                dsgm_metrics: commitment.impact.dsgm_metrics,
            }),
            anchor_hash: commitment.anchor_hash,
        }))
    }

    async fn execute_step(
        &self,
        request: Request<proto::ExecuteStepRequest>,
    ) -> Result<Response<proto::Step>, Status> {
        let req = request.into_inner();
        
        let step = self.agent.execute_step(&req.task_id, None)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(proto::Step {
            step_id: step.step_id,
            task_id: step.task_id,
            name: step.name,
            input: step.input,
            output: step.output,
            status: proto::StepStatus::Completed as i32,
            artifacts: vec![],
            is_last: step.is_last,
            phase_metadata: step.phase_metadata,
        }))
    }

    async fn list_tasks(
        &self,
        _request: Request<proto::ListTasksRequest>,
    ) -> Result<Response<proto::ListTasksResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }

    async fn get_task(
        &self,
        request: Request<proto::GetTaskRequest>,
    ) -> Result<Response<proto::Task>, Status> {
        let req = request.into_inner();
        let task = self.agent.get_task(&req.task_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(proto::Task {
            task_id: task.task_id,
            input: task.input,
            artifacts: vec![],
            created_at: None,
            updated_at: None,
            status: 1, // Created
            commitment: None,
            audit: None,
        }))
    }

    async fn list_steps(
        &self,
        request: Request<proto::ListStepsRequest>,
    ) -> Result<Response<proto::ListStepsResponse>, Status> {
        let req = request.into_inner();
        let _steps = self.agent.list_steps(&req.task_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // SOTA: Map native steps to proto steps
        Ok(Response::new(proto::ListStepsResponse {
            steps: vec![], 
            next_page_token: "".to_string(),
        }))
    }

    async fn list_artifacts(
        &self,
        _request: Request<proto::ListArtifactsRequest>,
    ) -> Result<Response<proto::ListArtifactsResponse>, Status> {
        Ok(Response::new(proto::ListArtifactsResponse {
            artifacts: vec![],
            next_page_token: "".to_string(),
        }))
    }
}
