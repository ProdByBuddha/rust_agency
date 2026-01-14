use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use anyhow::Result;
use rust_agency::memory::{Memory, MemoryEntry, vector::LocalVectorMemory};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, debug};
use tracing_subscriber::FmtSubscriber;
use std::env;
use axum::http::StatusCode;

struct AppState {
    memory: LocalVectorMemory,
}

#[derive(Deserialize)]
struct StoreRequest {
    entry: MemoryEntry,
}

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
    top_k: usize,
    context: Option<String>,
    kind: Option<rust_agency::orchestrator::Kind>,
}

#[derive(Serialize)]
struct StoreResponse {
    id: String,
}

#[derive(Serialize)]
struct SearchResponse {
    entries: Vec<MemoryEntry>,
}

// SOTA: Custom error type for better Axum integration
struct ServerError(anyhow::Error);

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, message) = (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal Server Error: {}", self.0),
        );
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    let subscriber = FmtSubscriber::builder()
        .with_env_filter("memory_server=debug,rust_agency=info")
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    info!("🧠 Memory Server (SOTA) Starting...");

    let memory_path = env::var("AGENCY_MEMORY_PATH").unwrap_or_else(|_| "memory.json".to_string());
    
    // Always use LOCAL memory in the server to avoid infinite recursion
    let memory = LocalVectorMemory::new(memory_path.into())?;
    
    // Ensure it's waked up
    memory.wake().await?;

    let state = Arc::new(AppState { memory });

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/store", post(store_handler))
        .route("/search", post(search_handler))
        .route("/persist", post(persist_handler))
        .route("/hibernate", post(hibernate_handler))
        .route("/wake", post(wake_handler))
        .route("/count", get(count_handler))
        .with_state(state);

    let port = env::var("AGENCY_MEMORY_PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{}", port);
    info!("🚀 Memory Server ready at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn store_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<StoreRequest>,
) -> Result<Json<StoreResponse>, ServerError> {
    debug!("Store request received");
    let id = state.memory.store(payload.entry).await?;
    Ok(Json(StoreResponse { id }))
}

async fn search_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, ServerError> {
    debug!("Search request: '{}'", payload.query);
    
    let entries = state.memory.search(
        &payload.query, 
        payload.top_k, 
        payload.context.as_deref(), 
        payload.kind
    ).await?;
    
    Ok(Json(SearchResponse { entries }))
}

async fn persist_handler(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, ServerError> {
    info!("Persist requested");
    state.memory.persist().await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn hibernate_handler(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, ServerError> {
    info!("Hibernate requested");
    state.memory.hibernate().await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn wake_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    info!("Wake requested");
    match state.memory.wake().await {
        Ok(_) => Json(serde_json::json!({ "status": "ok" })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() }))
    }
}

async fn count_handler(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, ServerError> {
    let count = state.memory.count().await?;
    Ok(Json(serde_json::json!({ "count": count })))
}
