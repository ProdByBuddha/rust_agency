use anyhow::Result;
use tracing::{info, error, warn, debug};
use reqwest::Client;
use serde_json::json;
use std::env;

pub struct Speaker {
    client: Client,
    server_url: String,
    enabled: bool,
}

impl Speaker {
    pub fn new() -> Result<Self> {
        let enabled = env::var("AGENCY_ENABLE_MOUTH").unwrap_or_else(|_| "1".to_string()) == "1";
        
        if enabled {
            info!("Speaker: Initializing client for remote Speaker Server...");
        } else {
            debug!("Speaker: Mouth is disabled.");
        }
        
        let host = env::var("AGENCY_SPEAKER_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("AGENCY_SPEAKER_PORT").unwrap_or_else(|_| "3000".to_string());
        let server_url = format!("http://{}:{}", host, port);
        
        Ok(Self {
            client: Client::new(),
            server_url,
            enabled,
        })
    }

    pub async fn init_default_voice(&mut self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Health check
        match self.client.get(format!("{}/health", self.server_url)).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("Speaker: Connected to server at {}", self.server_url);
                } else {
                    warn!("Speaker: Server responded with status {}", resp.status());
                }
            }
            Err(e) => {
                warn!("Speaker: Could not connect to server at {}: {}. Ensure bin/speaker_server is running.", self.server_url, e);
            }
        }
        Ok(())
    }

    pub async fn say(&mut self, text: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let url = format!("{}/say", self.server_url);
        let payload = json!({ "text": text });

        info!("Speaker: Sending text to server...");
        let resp = self.client.post(&url)
            .json(&payload)
            .send()
            .await;

        match resp {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Speaker: Synthesis requested successfully.");
                } else {
                    error!("Speaker: Server error: {:?}", response.text().await);
                }
            }
            Err(e) => {
                error!("Speaker: Failed to send request: {}. Is speaker_server running?", e);
            }
        }
        Ok(())
    }
}

impl Default for Speaker {
    fn default() -> Self {
        Self::new().expect("Failed to create speaker client")
    }
}