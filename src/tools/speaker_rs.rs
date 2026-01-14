use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent::Speaker;
use crate::tools::{Tool, ToolOutput};

/// Tool for generating speech using the native Rust Speaker
pub struct SpeakerRsTool {
    speaker: Arc<Mutex<Speaker>>,
}

impl SpeakerRsTool {
    pub fn new(speaker: Arc<Mutex<Speaker>>) -> Self {
        Self { speaker }
    }
}

#[async_trait]
impl Tool for SpeakerRsTool {
    fn name(&self) -> String {
        "speaker_rust".to_string()
    }

    fn description(&self) -> String {
        "Generates speech from text using a high-performance native Rust engine. \
         Supports paralinguistic tags for realism, such as [laugh], [chuckle], [cough], [sigh], [um], and [uh]. \
         Returns a status indicator representing completion.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "The text to convert to speech"
                }
            },
            "required": ["text"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let text = params["text"].as_str().ok_or_else(|| anyhow::anyhow!("Missing text parameter"))?;
        
        let mut speaker = self.speaker.lock().await;
        // High-level say() handles internal streaming and async pipeline
        speaker.say(text).await?;

        Ok(ToolOutput::success(
            json!({ "status": "completed" }),
            format!("Successfully synthesized and played: '{}'", text)
        ))
    }
}