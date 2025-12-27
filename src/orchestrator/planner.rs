//! Planner - Task decomposition and planning
//! 
//! Breaks down complex queries into actionable steps.

use anyhow::Result;
use ollama_rs::Ollama;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use std::sync::Arc;

use crate::agent::{AgentType, LLMProvider, OllamaProvider};

/// A step in a plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanStep {
    /// Step number
    pub step_num: usize,
    /// Description of what needs to be done
    pub description: String,
    /// Which agent should handle this step
    pub agent_type: AgentType,
    /// Tools that might be needed
    pub suggested_tools: Vec<String>,
    /// Expected output from this step
    pub expected_output: String,
    /// Dependencies on other steps (by step number)
    pub depends_on: Vec<usize>,
    /// Whether this step is completed
    pub completed: bool,
    /// The actual output from execution
    pub output: Option<String>,
}

/// A plan for executing a complex task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Plan {
    /// The original goal/query
    pub goal: String,
    /// List of steps to achieve the goal
    pub steps: Vec<PlanStep>,
    /// Current step being executed
    pub current_step: usize,
    /// Whether the plan is complete
    pub is_complete: bool,
}

impl Plan {
    pub fn new(goal: impl Into<String>) -> Self {
        Self {
            goal: goal.into(),
            steps: Vec::new(),
            current_step: 0,
            is_complete: false,
        }
    }

    /// Get the next step to execute
    pub fn next_step(&self) -> Option<&PlanStep> {
        self.steps.get(self.current_step).filter(|s| !s.completed)
    }

    /// Get all steps that are ready for execution (not completed and dependencies met)
    pub fn ready_steps(&self) -> Vec<&PlanStep> {
        self.steps.iter()
            .filter(|s| !s.completed)
            .filter(|s| {
                s.depends_on.iter().all(|&dep_num| {
                    self.steps.iter().any(|prev| prev.step_num == dep_num && prev.completed)
                })
            })
            .collect()
    }

    /// Mark a specific step as complete
    pub fn complete_step(&mut self, step_num: usize, output: impl Into<String>) {
        if let Some(step) = self.steps.iter_mut().find(|s| s.step_num == step_num) {
            step.completed = true;
            step.output = Some(output.into());
        }
        
        // Update current_step index to the first incomplete step
        self.current_step = self.steps.iter().position(|s| !s.completed).unwrap_or(self.steps.len());
        
        if self.current_step >= self.steps.len() {
            self.is_complete = true;
        }
    }

    /// Mark the current step as complete
    pub fn complete_current_step(&mut self, output: impl Into<String>) {
        if let Some(step) = self.steps.get_mut(self.current_step) {
            step.completed = true;
            step.output = Some(output.into());
        }
        self.current_step += 1;
        if self.current_step >= self.steps.len() {
            self.is_complete = true;
        }
    }

    /// Get all completed steps
    pub fn completed_steps(&self) -> Vec<&PlanStep> {
        self.steps.iter().filter(|s| s.completed).collect()
    }

    /// Get progress as a percentage
    pub fn progress(&self) -> f32 {
        if self.steps.is_empty() {
            return 0.0;
        }
        (self.steps.iter().filter(|s| s.completed).count() as f32 / self.steps.len() as f32) * 100.0
    }

    /// Get a summary of the plan
    pub fn summary(&self) -> String {
        let step_summaries: Vec<String> = self.steps
            .iter()
            .map(|s| {
                let status = if s.completed { "✓" } else { " " };
                format!("[{}] {}. {} ({})", status, s.step_num, s.description, s.agent_type)
            })
            .collect();
        
        format!(
            "Goal: {}\nProgress: {:.0}%\nSteps:\n{}",
            self.goal,
            self.progress(),
            step_summaries.join("\n")
        )
    }
}

/// Planner for task decomposition
pub struct Planner {
    provider: Arc<dyn LLMProvider>,
    model: String,
}

impl Planner {
    pub fn new(ollama: Ollama) -> Self {
        Self {
            provider: Arc::new(OllamaProvider::new(ollama)),
            model: "qwen3:8b".to_string(),
        }
    }

    pub fn with_provider(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.provider = provider;
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Decompose a complex query into a plan
    pub async fn decompose(&self, query: &str) -> Result<Plan> {
        info!("Planning task decomposition for: {}", query);

        let prompt = format!(
            r#"q → decompose(3-5 steps) → steps
steps | map(s → {{desc, agent, tools, expected}})

Rules:
1. Search CODEBASE? Step 1 = codebase_explorer | memory_query
2. READ files first.

q = "{}"
"#,
            query
        );

        let system = Some(super::sns::get_sns_system_prompt());
        let content = self.provider.generate(&self.model, prompt, system).await?;

        self.parse_plan(query, &content)
    }

    fn parse_plan(&self, goal: &str, response: &str) -> Result<Plan> {
        let mut plan = Plan::new(goal);
        
        // Try parsing as JSON array of steps first (SNS output)
        if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                let json_str = &response[start..=end];
                if let Ok(steps_val) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(steps_arr) = steps_val.as_array() {
                        for (idx, v) in steps_arr.iter().enumerate() {
                            let step_num = idx + 1;
                            let description = v["desc"].as_str()
                                .or_else(|| v["description"].as_str())
                                .unwrap_or("Action step")
                                .to_string();
                            
                            let agent_str = v["agent"].as_str()
                                .map(|s| s.to_lowercase())
                                .unwrap_or_else(|| "reasoner".to_string());
                            
                            let agent_type = match agent_str.as_str() {
                                "general_chat" | "chat" => AgentType::GeneralChat,
                                "coder" | "programmer" => AgentType::Coder,
                                "researcher" | "research" => AgentType::Researcher,
                                "planner" | "planning" => AgentType::Planner,
                                _ => AgentType::Reasoner,
                            };

                            let suggested_tools = if let Some(tools_v) = v.get("tools") {
                                if let Some(tools_arr) = tools_v.as_array() {
                                    tools_arr.iter()
                                        .filter_map(|t| t.as_str())
                                        .map(|t| t.to_lowercase())
                                        .collect()
                                } else if let Some(tools_str) = tools_v.as_str() {
                                    tools_str.split(',')
                                        .map(|t| t.trim().to_lowercase())
                                        .filter(|t| !t.is_empty() && t != "none")
                                        .collect()
                                } else {
                                    Vec::new()
                                }
                            } else {
                                Vec::new()
                            };

                            let expected_output = v["expected"].as_str()
                                .or_else(|| v["output"].as_str())
                                .unwrap_or("Success")
                                .to_string();

                            plan.steps.push(PlanStep {
                                step_num,
                                description,
                                agent_type,
                                suggested_tools,
                                expected_output,
                                depends_on: if step_num > 1 { vec![step_num - 1] } else { vec![] },
                                completed: false,
                                output: None,
                            });
                        }
                        
                        if !plan.steps.is_empty() {
                            return Ok(plan);
                        }
                    }
                }
            }
        }

        // Fallback to previous Regex parsing
        let agent_re = Regex::new(r"(?i)AGENT:\s*(\w+)")?;
        let tools_re = Regex::new(r"(?i)TOOLS:\s*(.+)")?;
        let expected_re = Regex::new(r"(?i)EXPECTED:\s*(.+)")?;
        let step_marker_re = Regex::new(r"(?i)STEP\s*\[?(\d+)\]?:\s*(.+)")?;

        let mut current_step: Option<PlanStep> = None;

        for line in response.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(caps) = step_marker_re.captures(line) {
                // Save previous step if exists
                if let Some(step) = current_step.take() {
                    plan.steps.push(step);
                }

                let step_num = caps.get(1).map(|m| m.as_str().parse().unwrap_or(0)).unwrap_or(0);
                let description = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();

                current_step = Some(PlanStep {
                    step_num,
                    description,
                    agent_type: AgentType::Reasoner,
                    suggested_tools: Vec::new(),
                    expected_output: String::new(),
                    depends_on: if step_num > 1 { vec![step_num - 1] } else { vec![] },
                    completed: false,
                    output: None,
                });
            } else if let Some(ref mut step) = current_step {
                if let Some(caps) = agent_re.captures(line) {
                    let agent_str = caps.get(1).map(|m| m.as_str().to_lowercase()).unwrap_or_default();
                    step.agent_type = match agent_str.as_str() {
                        "general_chat" | "generalchat" | "chat" => AgentType::GeneralChat,
                        "coder" | "programmer" => AgentType::Coder,
                        "researcher" | "research" => AgentType::Researcher,
                        "planner" | "planning" => AgentType::Planner,
                        _ => AgentType::Reasoner,
                    };
                } else if let Some(caps) = tools_re.captures(line) {
                    step.suggested_tools = caps.get(1).map(|m| {
                        m.as_str()
                            .split(',')
                            .map(|t| t.trim().to_lowercase())
                            .filter(|t| !t.is_empty() && t != "none")
                            .collect()
                    }).unwrap_or_default();
                } else if let Some(caps) = expected_re.captures(line) {
                    step.expected_output = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                }
            }
        }

        // Push last step
        if let Some(step) = current_step {
            plan.steps.push(step);
        }

        // If no steps were parsed, create a simple single-step plan
        if plan.steps.is_empty() {
            plan.steps.push(PlanStep {
                step_num: 1,
                description: goal.to_string(),
                agent_type: AgentType::Reasoner,
                suggested_tools: vec![],
                expected_output: "Complete the task".to_string(),
                depends_on: vec![],
                completed: false,
                output: None,
            });
        }

        debug!("Created plan with {} steps", plan.steps.len());
        Ok(plan)
    }

    /// Refine a plan based on execution feedback
    pub async fn refine(&self, plan: &Plan, feedback: &str) -> Result<Plan> {
        let prompt = format!(
            r#"You are refining an existing plan based on execution feedback.

Original Goal: {}

Current Plan:
{}

Feedback from execution:
{}

Provide a refined plan that addresses the feedback. Use the same format as before:
STEP [N]: [Description]
AGENT: [agent_type]
TOOLS: [tool1, tool2] or NONE
EXPECTED: [expected output]
"#,
            plan.goal,
            plan.summary(),
            feedback
        );

        let content = self.provider.generate(&self.model, prompt, None).await?;

        self.parse_plan(&plan.goal, &content)
    }

    /// Check if a query is simple enough to skip planning
    pub fn should_skip_planning(&self, query: &str) -> bool {
        // Skip planning for simple queries
        let simple_indicators = [
            query.len() < 50,
            query.split_whitespace().count() < 10,
            query.contains("?") && query.matches("?").count() == 1,
        ];
        
        let complex_indicators = [
            query.contains(" and "),
            query.contains(" then "),
            query.contains("multiple"),
            query.contains("steps"),
            query.contains("first") && query.contains("then"),
        ];

        // Skip if mostly simple indicators and no complex ones
        let simple_score: usize = simple_indicators.iter().filter(|&&b| b).count();
        let complex_score: usize = complex_indicators.iter().filter(|&&b| b).count();

        simple_score >= 2 && complex_score == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_progress() {
        let mut plan = Plan::new("Test goal");
        plan.steps.push(PlanStep {
            step_num: 1,
            description: "Step 1".to_string(),
            agent_type: AgentType::Reasoner,
            suggested_tools: vec![],
            expected_output: "Output 1".to_string(),
            depends_on: vec![],
            completed: true,
            output: Some("Done".to_string()),
        });
        plan.steps.push(PlanStep {
            step_num: 2,
            description: "Step 2".to_string(),
            agent_type: AgentType::Coder,
            suggested_tools: vec![],
            expected_output: "Output 2".to_string(),
            depends_on: vec![1],
            completed: false,
            output: None,
        });

        assert_eq!(plan.progress(), 50.0);
    }

    #[test]
    fn test_should_skip_planning() {
        let planner = Planner::new(Ollama::default());
        
        assert!(planner.should_skip_planning("What is 2+2?"));
        assert!(planner.should_skip_planning("Hi"));
        assert!(!planner.should_skip_planning(
            "First research the topic, then write a summary, and finally create a presentation"
        ));
    }
}
