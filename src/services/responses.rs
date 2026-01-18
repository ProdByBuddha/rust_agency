//! Open Responses API Implementation
//! 
//! Implements the standard "Responses" API endpoint (/v1/responses).
//! This endpoint delegates to the Supervisor (Agency), enabling full
//! autonomous capabilities (Tools, Memory, Planning) behind a standard interface.

use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use crate::server::{AppState, ServerError};

#[derive(Deserialize)]
pub struct CreateResponseRequest {
    pub model: Option<String>,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub tools: Option<Vec<serde_json::Value>>, // Pass-through for now, or ignored if Agency handles tools
    pub stream: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ResponseObject {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ResponseChoice>,
    pub usage: Usage,
}

#[derive(Serialize)]
pub struct ResponseChoice {
    pub index: usize,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub async fn responses_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateResponseRequest>,
) -> Result<impl IntoResponse, ServerError> {
    // 1. Extract the latest user query from messages
    // The Agency Supervisor manages its own history, so we treat this as a "turn".
    // If the client sends a full history, we only really act on the last message.
    // (Future improvement: Sync client history with Agency memory if needed)
    let query = req.messages.last()
        .map(|m| m.content.clone())
        .ok_or_else(|| anyhow::anyhow!("No messages provided"))?;

    // 2. Lock Supervisor and Execute
    let mut supervisor = state.supervisor.lock().await;
    
    // Notify dashboard via WebSocket
    let _ = state.tx.send(format!("🚀 Request (API): {}", query));

    // Execute Agentic Loop
    let result = supervisor.handle(&query).await
        .map_err(|e| anyhow::anyhow!("Agency Execution Failed: {}", e))?;

    // 3. Map Result to Response Object
    let response = ResponseObject {
        id: format!("resp_{}", uuid::Uuid::new_v4()),
        object: "response".to_string(),
        created: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        model: "rust_agency_sovereign".to_string(),
        choices: vec![
            ResponseChoice {
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content: result.answer,
                },
                finish_reason: "stop".to_string(),
            }
        ],
        usage: Usage {
            prompt_tokens: 0, // TODO: Track actual tokens
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    Ok(Json(response))
}
