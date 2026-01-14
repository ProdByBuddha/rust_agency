use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FPF-aligned Unified Normalization Mechanism (UNM) (A.19)
/// 
/// Provides a single-writer frame for normalizing and comparing 
/// heterogeneous metrics (F, G, R, N, Q, D, CL).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CNFrame {
    pub metrics: HashMap<String, f32>,
    /// Γ-fold policy: "WeakestLink" or "Additive"
    pub fold_policy: String,
}

impl CNFrame {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            fold_policy: "WeakestLink".to_string(),
        }
    }

    pub fn record(&mut self, key: impl Into<String>, value: f32) {
        self.metrics.insert(key.into(), value.clamp(0.0, 1.0));
    }

    /// Calculate the Coherence Score (χ) of the current state
    pub fn calculate_coherence(&self) -> f32 {
        if self.metrics.is_empty() { return 1.0; } 
        
        // FPF Standard: Global Coherence measures INTEGRITY, not SCALE.
        // We only roll up metrics that represent protocol adherence or trust.
        let mut min_integrity = 1.0;
        let mut found_integrity = false;

        for (k, v) in &self.metrics {
            if k.contains("Coherence") || k.contains("Congruence") || k.contains("Assurance") {
                if *v < min_integrity { min_integrity = *v; }
                found_integrity = true;
            }
        }

        if found_integrity { min_integrity } else { 1.0 }
    }

    pub fn format_for_audit(&self) -> String {
        let mut output = format!("CN-FRAME (Coherence χ: {:.2})\n", self.calculate_coherence());
        for (k, v) in &self.metrics {
            output.push_str(&format!("  - {}: {:.2}\n", k, v));
        }
        output
    }
}
