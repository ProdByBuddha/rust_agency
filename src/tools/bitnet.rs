use anyhow::{Context, Result};
use async_trait::async_trait;
use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use serde_json::{json, Value};
use tracing::info;

use super::{Tool, ToolOutput};

/// Tool for low-resource inference using BitNet models (via Ollama or custom runner)
pub struct BitNetInferenceTool {
    ollama: Ollama,
    model: String,
}

impl BitNetInferenceTool {
    pub fn new(ollama: Ollama, model: impl Into<String>) -> Self {
        Self {
            ollama,
            model: model.into(),
        }
    }
}

impl Default for BitNetInferenceTool {
    fn default() -> Self {
        Self::new(Ollama::default(), "llama3.2:1b")
    }
}

#[async_trait]
impl Tool for BitNetInferenceTool {
    fn name(&self) -> &str {
        "bitnet_inference"
    }

    fn description(&self) -> &str {
        "Perform rapid, low-resource inference for simple logic, classification, or extraction tasks using 1-bit quantized models."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "The task or prompt for the BitNet model"
                },
                "task_type": {
                    "type": "string",
                    "enum": ["logic", "extraction", "summary", "classification"],
                    "description": "The type of task to optimize for"
                }
            },
            "required": ["prompt"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let prompt = params["prompt"].as_str().context("Missing prompt")?;
        let task_type = params["task_type"].as_str().unwrap_or("logic");

        info!("BitNet Inference: Executing {} task...", task_type);

        // For now, we route this to Ollama using the specialized BitNet model
        // In a production environment, this would call a native 1-bit kernel runner
        let response = self.ollama
            .send_chat_messages(ChatMessageRequest::new(
                self.model.clone(),
                vec![ChatMessage::user(prompt.to_string())],
            ))
            .await
            .context("BitNet inference failed. Ensure the model is available in Ollama.")?;

        let answer = response.message.content;

        Ok(ToolOutput::success(
            json!({ "model": self.model, "response": answer, "task": task_type }),
            format!("BitNet ({}) Result:\n{}", self.model, answer)
        ))
    }
}
