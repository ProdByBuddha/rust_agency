use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::tools::ToolCall;

/// FPF-aligned Novelty-Quality-Diversity (NQD) Scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NQDScores {
    /// Novelty (0.0 - 1.0): How unique this action is compared to history
    pub novelty: f32,
    /// Quality (0.0 - 1.0): Progress toward the U.Objective
    pub quality: f32,
    /// Diversity (0.0 - 1.0): Contribution to the exploration portfolio
    pub diversity: f32,
}

/// A 'Niche' in the exploration space (e.g. "Codebase", "Memory", "Web")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NQDPartition {
    Codebase,
    Memory,
    Execution,
    Search,
    Metadata,
    General,
}

impl NQDPartition {
    pub fn from_tool_name(name: &str) -> Self {
        match name {
            "codebase_explorer" | "artifact_manager" => Self::Codebase,
            "memory_query" | "knowledge_graph_viewer" => Self::Memory,
            "code_exec" | "sandbox" => Self::Execution,
            "web_search" => Self::Search,
            "agency_control" | "forge_tool" => Self::Metadata,
            _ => Self::General,
        }
    }
}

/// Portfolio of attempted paths to ensure diversity (C.18)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NQDPortfolio {
    /// Count of attempts per niche
    pub niches: HashMap<NQDPartition, usize>,
    /// History of unique action fingerprints
    pub action_fingerprints: Vec<String>,
}

impl NQDPortfolio {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an action and return its NQD scores
    pub fn evaluate_action(&mut self, actions: &[ToolCall], quality_estimate: f32) -> NQDScores {
        if actions.is_empty() {
            return NQDScores { novelty: 0.0, quality: quality_estimate, diversity: 0.0 };
        }

        let mut total_novelty = 0.0;
        let mut total_diversity = 0.0;

        for action in actions {
            // 1. Calculate Novelty (Is the specific parameter set new?)
            let fingerprint = format!("{}:{}", action.name, serde_json::to_string(&action.parameters).unwrap_or_default());
            let novelty = if self.action_fingerprints.contains(&fingerprint) {
                0.1 // Repetitive
            } else {
                self.action_fingerprints.push(fingerprint);
                0.9 // New
            };
            total_novelty += novelty;

            // 2. Calculate Diversity (Are we stuck in one tool category?)
            let niche = NQDPartition::from_tool_name(&action.name);
            let count = self.niches.entry(niche).or_insert(0);
            *count += 1;
            
            let diversity = if *count > 2 { 0.3 } else { 0.8 };
            total_diversity += diversity;
        }

        NQDScores {
            novelty: total_novelty / actions.len() as f32,
            quality: quality_estimate,
            diversity: total_diversity / actions.len() as f32,
        }
    }

    pub fn format_for_prompt(&self) -> String {
        let mut output = String::from("## NQD EXPLORATION PORTFOLIO\n");
        if self.niches.is_empty() {
            output.push_str("Status: Initial exploration. All niches available.\n");
        } else {
            output.push_str("Occupied Niches:\n");
            for (niche, count) in &self.niches {
                output.push_str(&format!("  - {:?}: {} attempts\n", niche, count));
            }
            output.push_str("INSTRUCTION: If a niche has > 2 attempts, prioritize OTHER categories to ensure Diversity (D).\n");
        }
        output
    }
}
