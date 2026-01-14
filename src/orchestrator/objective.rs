use serde::{Deserialize, Serialize};
use crate::orchestrator::ServiceClause;

/// FPF-aligned U.Objective
/// 
/// Formalizes the "Will" of the agency by binding a goal to specific 
/// success criteria and resource constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    /// The high-level intent (The "What")
    pub goal: String,
    /// FPF Integration: Service Clause (The formal Promise)
    pub service_clause: ServiceClause,
    /// Hard limits on the execution (The "Bounds")
    pub resource_budget: ResourceBudget,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub max_cycles: usize,
    pub max_tokens: Option<u32>,
    pub max_time_seconds: u64,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self {
            max_cycles: 5,
            max_tokens: None,
            max_time_seconds: 300,
        }
    }
}

impl Objective {
    pub fn new(goal: impl Into<String>) -> Self {
        let g = goal.into();
        Self {
            goal: g.clone(),
            service_clause: ServiceClause::new(&g, "Performer", "User"),
            resource_budget: ResourceBudget::default(),
        }
    }

    pub fn with_acceptance(mut self, criteria: impl Into<String>) -> Self {
        self.service_clause = self.service_clause.with_acceptance(criteria);
        self
    }

    pub fn with_budget(mut self, budget: ResourceBudget) -> Self {
        self.resource_budget = budget;
        self
    }
    pub fn format_for_prompt(&self) -> String {
        let mut output = format!("## U.OBJECTIVE\nGOAL: {}\n", self.goal);
        
        if !self.service_clause.acceptance_spec.is_empty() {
            output.push_str("ACCEPTANCE CRITERIA (The 'Good'):\n");
            for (i, criteria) in self.service_clause.acceptance_spec.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, criteria));
            }
        }

        output.push_str(&format!("RESOURCE BUDGET: Max {} cycles / {}s timeout\n", 
            self.resource_budget.max_cycles, 
            self.resource_budget.max_time_seconds
        ));
        
        output
    }
}
