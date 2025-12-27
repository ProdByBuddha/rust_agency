use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;
use reqwest::Client;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate(&self, model: &str, prompt: String, system: Option<String>) -> Result<String>;
}

pub struct OllamaProvider {
    client: ollama_rs::Ollama,
}

impl OllamaProvider {
    pub fn new(client: ollama_rs::Ollama) -> Self {
        Self { client }
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn generate(&self, model: &str, prompt: String, system: Option<String>) -> Result<String> {
        use ollama_rs::generation::chat::{request::ChatMessageRequest, ChatMessage};
        
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(ChatMessage::system(sys));
        }
        messages.push(ChatMessage::user(prompt));

        let res = self.client.send_chat_messages(ChatMessageRequest::new(
            model.to_string(),
            messages,
        )).await?;

        Ok(res.message.content)
    }
}

pub struct OpenAICompatibleProvider {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl OpenAICompatibleProvider {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAICompatibleProvider {
    async fn generate(&self, model: &str, prompt: String, system: Option<String>) -> Result<String> {
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(json!({ "role": "system", "content": sys }));
        }
        messages.push(json!({ "role": "user", "content": prompt }));

        let body = json!({
            "model": model,
            "messages": messages,
            "temperature": 0.7,
        });

        let mut request = self.client.post(format!("{}/chat/completions", self.base_url.trim_end_matches('/')))
            .json(&body);

        if let Some(ref key) = self.api_key {
            request = request.bearer_auth(key);
        }

        let res = request.send().await?.error_for_status()?;
        let json: serde_json::Value = res.json().await?;
        
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .context("Failed to parse content from OpenAI response")?;

        Ok(content.to_string())
    }
}
