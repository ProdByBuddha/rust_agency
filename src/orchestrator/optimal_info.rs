//! Optimal Information Selector
//! 
//! Inspired by "What Data Enables Optimal Decisions?" (Bennouna et al., 2025).
//! This module implements a heuristic to select the "minimal sufficient dataset" 
//! (or set of queries) required to make an optimal decision (plan).
//!
//! It maps the paper's concepts to LLM-based planning:
//! - **Decision Task**: The Plan we want to execute.
//! - **Uncertainty Set**: The set of possible states of the environment (codebase, external world).
//! - **Relevant Directions**: The specific pieces of information that would cause us to change our plan.
//!
//! Algorithm:
//! 1. Identify critical assumptions in the current plan.
//! 2. Generate potential queries to verify these assumptions.
//! 3. Select the minimal subset of queries that spans the "relevant directions" of uncertainty.

use anyhow::Result;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::agent::LLMProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQuery {
    pub description: String,
    pub tool_call: String,
    pub cost_estimate: u32, // Abstract cost (e.g., token count, latency)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationNeed {
    pub assumption: String,
    pub relevance_score: f32, // 0.0 to 1.0, how much it impacts the plan
    pub queries: Vec<DataQuery>,
}

pub struct OptimalInfoSelector {
    provider: Arc<dyn LLMProvider>,
    model: String,
}

impl OptimalInfoSelector {
    pub fn new(provider: Arc<dyn LLMProvider>, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
        }
    }

    /// Selects the minimal sufficient set of queries to verify a plan.
    /// 
    /// Corresponds to finding a basis for `dir(X*(C))` in the paper.
    pub async fn select_minimal_queries(&self, goal: &str, plan_summary: &str) -> Result<Vec<DataQuery>> {
        info!("Selecting optimal information for goal: {}", goal);

        // Step 1: Identify Relevant Directions (Assumptions that matter)
        // This mirrors finding directions where the optimal solution changes.
        let prompt = format!(
            r#"Goal: {}
Plan: {}

Task: Identify critical assumptions in this plan.
An assumption is CRITICAL if its falsehood would require changing the plan (Decision Sensitivity).

For each assumption, suggest 1-2 specific tool queries (e.g., `read_file`, `grep`, `web_search`) to verify it.
Keep queries MINIMAL (e.g., check specific lines rather than reading whole files).

Output JSON format:
[
  {{
    "assumption": "The API endpoint /v1/chat exists",
    "relevance_score": 0.9,
    "queries": [
      {{ "description": "Check routes file", "tool_call": "grep '/v1/chat' src/routes.rs", "cost_estimate": 1 }}
    ]
  }}
]
"#,
            goal, plan_summary
        );

        let system = Some("You are an expert in Optimal Experiment Design and Decision Theory.".to_string());
        let response = self.provider.generate(&self.model, prompt, system).await?;

        // Parse relevant needs
        let needs: Vec<InformationNeed> = self.parse_needs(&response);

        // Step 2: Construct Minimal Sufficient Dataset
        // We select the cheapest query for each high-relevance assumption.
        // This is a greedy approximation of the paper's basis construction.
        let mut selected_queries = Vec::new();
        
        for need in needs {
            if need.relevance_score > 0.5 {
                // Find the lowest cost query that satisfies this need
                if let Some(best_query) = need.queries.iter().min_by_key(|q| q.cost_estimate) {
                    selected_queries.push(best_query.clone());
                }
            }
        }

        Ok(selected_queries)
    }

    fn parse_needs(&self, response: &str) -> Vec<InformationNeed> {
        // Attempt to parse JSON from the response
        if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                let json_str = &response[start..=end];
                if let Ok(needs) = serde_json::from_str::<Vec<InformationNeed>>(json_str) {
                    return needs;
                }
            }
        }
        
        // Fallback or empty if parsing fails (in a real implementation, we'd have robust parsing)
        Vec::new()
    }
}
