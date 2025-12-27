use serde::{Deserialize, Serialize};
use crate::orchestrator::profile::AgencyProfile;

/// Types of specialized agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    GeneralChat,
    Reasoner,
    Coder,
    Researcher,
    Planner,
    Reviewer,
    BitNet, // Ultra-low-bit specialized agent
}

impl AgentType {
    /// Get the default model for this agent type
    pub fn default_model(&self) -> &'static str {
        match self {
            AgentType::GeneralChat => "llama3.2:3b",
            AgentType::Reasoner => "deepseek-r1:8b",
            AgentType::Coder => "qwen2.5-coder:7b",
            AgentType::Researcher => "qwen3:8b",
            AgentType::Planner => "qwen3:8b",
            AgentType::Reviewer => "deepseek-r1:8b",
            AgentType::BitNet => "llama3.2:1b",
        }
    }

    /// Generate a system prompt based on agent type and agency profile
    pub fn generate_system_prompt(&self, profile: &AgencyProfile) -> String {
        let base = match self {
            AgentType::GeneralChat => 
                format!("You are '{}', a SOTA semi-autonomous multi-agent system. \
                 You are friendly, helpful, and technically capable. \
                 Answer questions directly and concisely. \
                 If asked about your identity, you are {}, built in Rust.", profile.name, profile.name),
            
            AgentType::Reasoner => 
                "You are a logical reasoning assistant. Think step by step through problems. \
                 Break down complex questions into smaller parts. Show your reasoning process. \
                 Be precise and analytical in your responses.".to_string(),
            
            AgentType::Coder => 
                "You are an expert programmer. You are proficient in Rust, Python, JavaScript, and Shell scripting. \
                 CRITICAL RULES: \
                 1. If the user asks about the CURRENT CODEBASE, you MUST use 'codebase_explorer' or 'memory_query' to READ the files first. \
                 2. If you need a capability that does not exist (e.g. specialized analysis, new API interaction), you MUST use 'forge_tool' to create it. \
                 3. NEVER assume a file exists or pretend to know its content without reading it. \
                 4. NEVER provide placeholders like '<result>' or '<value>'. You MUST actually perform the calculation or action using tools. \
                 5. Your final [ANSWER] must be the ACTUAL literal result, not a description of what you 'would' do. \
                 6. Minimize conversational filler and focus on technical accuracy.".to_string(),
            
            AgentType::Researcher => 
                "You are a research assistant with access to web search. \
                 Gather information from multiple sources, synthesize findings, \
                 and provide well-cited responses. Be thorough but concise. \
                 Distinguish between facts and interpretations.".to_string(),
            
            AgentType::Planner => 
                "You are a planning and task decomposition specialist. \
                 Break down complex goals into actionable steps. \
                 Consider dependencies, resources, and potential obstacles. \
                 Create clear, sequential plans that can be executed.".to_string(),

            AgentType::Reviewer =>
                "You are a technical reviewer and fact-checker. Your job is to verify the accuracy of the Assistant's responses. \
                 CRITICAL RULES: \
                 1. Compare the [OBSERVATION] from tools with the [ANSWER]. \
                 2. If there is a contradiction (e.g., tool saw .rs files but answer says .py), you MUST flag it as a HALLUCINATION. \
                 3. Be strict. If the assistant guessed something it didn't actually see, mark it as unverified. \
                 4. Your goal is to ensure the agency never provides false technical information.".to_string(),

            AgentType::BitNet =>
                "You are a high-speed, low-resource reasoning core for a Rust-based multi-agent system. \
                 Your task is to provide rapid thoughts and insights. \
                 Focus on raw logic, Rust idioms, and efficient architectural patterns. \
                 Be extremely concise. NEVER suggest non-Rust solutions like Node.js or Python unless explicitly asked.".to_string(),
        };

        format!("{}\n\nAGENCY CONTEXT:\n- Name: {}\n- Mission: {}\n- Traits: {}", 
            base, profile.name, profile.mission, profile.traits.join(", "))
    }
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::GeneralChat => write!(f, "GeneralChat"),
            AgentType::Reasoner => write!(f, "Reasoner"),
            AgentType::Coder => write!(f, "Coder"),
            AgentType::Researcher => write!(f, "Researcher"),
            AgentType::Planner => write!(f, "Planner"),
            AgentType::Reviewer => write!(f, "Reviewer"),
            AgentType::BitNet => write!(f, "BitNet"),
        }
    }
}

/// Configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub model: String,
    pub system_prompt: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    /// Which tools this agent can use
    pub allowed_tools: Vec<String>,
    /// Max iterations for ReAct loop
    pub max_iterations: usize,
    /// Optional URL for OpenAI-compatible provider (e.g. vLLM)
    pub provider_url: Option<String>,
}

impl AgentConfig {
    pub fn new(agent_type: AgentType, profile: &AgencyProfile) -> Self {
        let allowed_tools = match agent_type {
            AgentType::GeneralChat => vec!["agency_control".to_string()],
            AgentType::Reasoner => vec!["memory_query".to_string(), "knowledge_graph_viewer".to_string(), "agency_control".to_string()],
            AgentType::Coder => vec![
                "codebase_explorer".to_string(), 
                "code_exec".to_string(), 
                "sandbox".to_string(), 
                "artifact_manager".to_string(), 
                "forge_tool".to_string(), 
                "system_monitor".to_string(),
                "agency_control".to_string()
            ],
            AgentType::Researcher => vec!["web_search".to_string(), "memory_query".to_string()],
            AgentType::Planner => vec![],
            AgentType::Reviewer => vec![],
            AgentType::BitNet => vec!["memory_query".to_string()],
        };

        Self {
            agent_type,
            model: agent_type.default_model().to_string(),
            system_prompt: agent_type.generate_system_prompt(profile),
            temperature: 0.7,
            max_tokens: None,
            allowed_tools,
            max_iterations: 5,
            provider_url: None,
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self::new(AgentType::Reasoner, &AgencyProfile::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_default_model() {
        assert_eq!(AgentType::GeneralChat.default_model(), "llama3.2:3b");
        assert_eq!(AgentType::Coder.default_model(), "qwen2.5-coder:7b");
    }

    #[test]
    fn test_generate_system_prompt() {
        let profile = AgencyProfile::default();
        let prompt = AgentType::Reasoner.generate_system_prompt(&profile);
        assert!(prompt.contains("logical reasoning assistant"));
        assert!(prompt.contains("AGENCY CONTEXT"));
        assert!(prompt.contains(&profile.name));
    }

    #[test]
    fn test_agent_config_new() {
        let profile = AgencyProfile::default();
        let config = AgentConfig::new(AgentType::Coder, &profile);
        assert_eq!(config.agent_type, AgentType::Coder);
        assert!(config.allowed_tools.contains(&"code_exec".to_string()));
        assert_eq!(config.max_iterations, 5);
    }
}

