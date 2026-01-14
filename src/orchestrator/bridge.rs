use serde::{Deserialize, Serialize};

/// FPF-aligned Alignment Bridge (F.9)
/// 
/// Maps concepts between two BoundedContexts and quantifies information loss.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bridge {
    pub source_context: String,
    pub target_context: String,
    /// Congruence Level (0.0 - 1.0): 1.0 = Bit-perfect mapping.
    pub congruence_level: f32,
    pub loss_notes: Vec<String>,
}

impl Bridge {
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source_context: source.into(),
            target_context: target.into(),
            congruence_level: 1.0,
            loss_notes: Vec::new(),
        }
    }

    pub fn with_loss(mut self, cl: f32, note: impl Into<String>) -> Self {
        self.congruence_level = cl.clamp(0.0, 1.0);
        self.loss_notes.push(note.into());
        self
    }

    pub fn format_for_audit(&self) -> String {
        format!(
            "BRIDGE: {} → {} (CL: {:.2})\nLOSS NOTES: {}",
            self.source_context,
            self.target_context,
            self.congruence_level,
            if self.loss_notes.is_empty() { "None".to_string() } else { self.loss_notes.join("; ") }
        )
    }
}


