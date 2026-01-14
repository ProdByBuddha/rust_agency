use tracing::{error, info};
use std::io::{Write, BufReader, BufRead};
use std::process::{Command, Child, Stdio, ChildStdin};
use std::sync::Arc;
use tokio::sync::Mutex;
use tempfile::NamedTempFile;
use base64::{engine::general_purpose, Engine as _};
use std::time::Duration;
use std::path::Path;

/// A seamless Rust-managed speaker that bridges to the high-performance 
/// PerthNet/Chatterbox model via a persistent process pipe.
/// This completes the "Full Port" by automating the lifecycle of the model backend.
pub struct Speaker {
    inner: Arc<Mutex<Option<SpeakerInner>>>,
}

struct SpeakerInner {
    child: Child,
    stdin: ChildStdin,
}

impl Speaker {
    /// Creates a new Speaker instance. Initialization is lazy to ensure 
    /// fast application startup.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Ensures the backend process is running and ready.
    async fn ensure_initialized(&self) -> Result<(), String> {
        let mut lock = self.inner.lock().await;
        if lock.is_none() {
            info!("Speaker: Initializing managed backend (MPS/GPU enabled)...");
            
            let python_path = "./speak_venv/bin/python";
            let script_path = "speaker_pipe.py";

            if !Path::new(python_path).exists() {
                return Err(format!("Missing virtualenv at {}", python_path));
            }

            let mut child = Command::new(python_path)
                .arg(script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to spawn speaker process: {}", e))?;

            let stdin = child.stdin.take().ok_or("Failed to open stdin")?;
            let stdout = child.stdout.take().ok_or("Failed to open stdout")?;

            *lock = Some(SpeakerInner { child, stdin });

            // Spawn background monitoring and audio processing thread
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                
                while let Ok(n) = reader.read_line(&mut line) {
                    if n == 0 { break; }
                    let trimmed = line.trim();
                    
                    if trimmed == "READY" {
                        info!("Speaker: Managed port is READY.");
                    } else if trimmed.starts_with("AUDIO:") {
                        let b64 = &trimmed[6..];
                        if let Ok(bytes) = general_purpose::STANDARD.decode(b64) {
                            Self::play_bytes(bytes);
                        }
                    } else if trimmed.starts_with("ERROR:") {
                        error!("Speaker: Backend error: {}", &trimmed[6..]);
                    }
                    line.clear();
                }
                error!("Speaker: Backend process pipe closed.");
            });
            
            // Give it a moment to initialize the GPU
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Ok(())
    }

    /// Synthesizes and plays the given text using the managed port.
    pub async fn say(&self, text: &str) {
        if text.trim().is_empty() { return; } // Avoid empty strings
        
        if let Err(e) = self.ensure_initialized().await {
            error!("Speaker: Initialization failed: {}", e);
            return;
        }

        let mut lock = self.inner.lock().await;
        if let Some(ref mut inner) = *lock {
            // Sanitize text for pipe
            let clean_text = text.replace('\n', " ").replace('\r', "");
            if let Err(e) = writeln!(inner.stdin, "{}", clean_text) {
                error!("Speaker: Pipe write failed: {}", e);
            }
            let _ = inner.stdin.flush();
        }
    }

    fn play_bytes(bytes: Vec<u8>) {
        let mut file = match NamedTempFile::new() {
            Ok(f) => f,
            Err(_) => return,
        };

        if file.write_all(&bytes).is_ok() {
            let path = file.path().to_string_lossy().to_string();
            // High-fidelity, zero-latency playback via native afplay
            let _ = Command::new("afplay")
                .arg(path)
                .status();
        }
    }
}

impl Default for Speaker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SpeakerInner {
    fn drop(&mut self) {
        // Graceful shutdown
        let _ = writeln!(self.stdin, "EXIT");
        let _ = self.child.kill();
    }
}