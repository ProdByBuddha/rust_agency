//! SOTA Semi-Autonomous Agency
//! 
//! A state-of-the-art Rust-based multi-agent system with:
//! - Semantic memory (ChromaDB + fastembed)
//! - ReAct reasoning framework
//! - Structured tool calling
//! - Multi-agent coordination
//! - Safety guardrails

pub mod memory;
pub mod agent;
pub mod tools;
pub mod orchestrator;
pub mod safety;

// Re-exports for convenience
pub use agent::AgentType;
pub use memory::VectorMemory;
pub use orchestrator::Supervisor;
pub use tools::ToolRegistry;
