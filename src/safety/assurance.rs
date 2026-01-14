use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::tools::Tool;
use std::sync::Arc;

/// FPF-aligned Trust & Assurance Calculus (B.3)
/// 
/// R = F * G (Simplified for local agency)
/// Reliability = Formality * Scope Alignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssuranceScore {
    /// Formality (0.0 - 1.0): Structural integrity of the request
    pub f: f32,
    /// Scope (0.0 - 1.0): Alignment with U.WorkScope
    pub g: f32,
    /// Reliability (0.0 - 1.0): The final trust level
    pub r: f32,
}

impl AssuranceScore {
    pub fn calculate(tool: Arc<dyn Tool>, params: &Value) -> Self {
        // 1. Calculate Formality (F)
        // Check if parameters match the expected schema (simplified)
        let f = if params.is_object() && !params.as_object().unwrap().is_empty() {
            1.0
        } else {
            0.5
        };

        // 2. Calculate Scope Alignment (G)
        // Check params against the tool's WorkScope
        let scope = tool.work_scope();
        let g = if scope["status"] == "highly_constrained" || scope["status"] == "highly_agential" {
            // Risky tools require precise params to maintain high G
            if params.to_string().len() > 10 { 0.9 } else { 0.4 }
        } else {
            0.8 // Standard tools have higher base alignment
        };

        Self {
            f,
            g,
            r: f * g,
        }
    }

    pub fn is_trustworthy(&self) -> bool {
        self.r > 0.6
    }

    pub fn get_warning(&self) -> Option<String> {
        if self.r <= 0.4 {
            Some("CRITICAL: Low assurance. Plan is vague or tool is high-risk.".to_string())
        } else if self.r <= 0.6 {
            Some("WARNING: Medium assurance. Verification recommended.".to_string())
        } else {
            None
        }
    }
}
