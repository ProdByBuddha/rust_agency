//! SOTA Semi-Autonomous Agency
//! 
//! A state-of-the-art Rust-based multi-agent system featuring:
//! - ReAct reasoning framework with tool calling
//! - Semantic vector memory (ChromaDB + fastembed)
//! - Multi-agent coordination with planning
//! - Self-reflection and error correction
//! - Safety guardrails
//! - Full session persistence

use anyhow::Result;
use ollama_rs::Ollama;
use std::io::{self, Write};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod agent;
mod memory;
mod orchestrator;
mod safety;
mod tools;

use memory::{Memory, VectorMemory, CodebaseIndexer, MemoryManager};
use orchestrator::{Supervisor, SessionManager};
use safety::SafetyGuard;
use tools::{ToolRegistry, WebSearchTool, CodeExecTool, MemoryQueryTool, KnowledgeGraphTool, ArtifactTool, SandboxTool, CodebaseTool, SystemTool, ForgeTool, BitNetInferenceTool};

// ──────────────────────────────────────────────────────────────────────────────
// CONFIGURATION
// ──────────────────────────────────────────────────────────────────────────────

/// Configuration for the agency
struct AgencyConfig {
    /// Path to fallback memory file
    memory_file: String,
    /// Path to session persistence file
    session_file: String,
}

impl Default for AgencyConfig {
    fn default() -> Self {
        Self {
            memory_file: "memory.json".to_string(),
            session_file: "session.json".to_string(),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MAIN ENTRY POINT
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    println!("\n{}", "═".repeat(60));
    println!("🚀 SOTA Semi-Autonomous Agency v0.2.0");
    println!("{}", "═".repeat(60));
    println!("Features: ReAct | Vector Memory | Multi-Agent | Planning | Persistence");
    println!("{}\n", "═".repeat(60));

    let config = AgencyConfig::default();
    
    // Initialize memory system
    let memory: Arc<dyn Memory> = Arc::new(
        VectorMemory::new(&config.memory_file)
            .expect("Failed to initialize memory system")
    );
    
    // Semantic codebase indexing - run in background to avoid blocking startup
    let indexer = CodebaseIndexer::new("src", memory.clone());
    tokio::spawn(async move {
        let _ = indexer.index_all().await;
    });

    let memory_count = memory.count().await.unwrap_or(0);
    info!("Memory initialized with {} entries", memory_count);
    println!("📚 Memory: {} stored entries", memory_count);

    // Initialize MemoryManager for resource tracking
    let manager = Arc::new(MemoryManager::new(memory.clone()));

    // Initialize tools
    let tools = Arc::new(ToolRegistry::new());
    tools.register_instance(WebSearchTool::new()).await;
    tools.register_instance(CodeExecTool::new()).await;
    tools.register_instance(MemoryQueryTool::new(memory.clone())).await;
    tools.register_instance(KnowledgeGraphTool::new(memory.clone())).await;
    tools.register_instance(ArtifactTool::default()).await;
    tools.register_instance(SandboxTool::default()).await;
    tools.register_instance(CodebaseTool::default()).await;
    tools.register_instance(BitNetInferenceTool::default()).await;
    tools.register_instance(ForgeTool::new("custom_tools", tools.clone())).await;
    tools.register_instance(SystemTool::new(manager.clone())).await;
    
    // Load existing custom tools
    let _ = tools.load_dynamic_tools("custom_tools").await;
    
    println!("🔧 Tools: {}", tools.tool_names().await.join(", "));

    // Initialize session persistence
    let session_manager = SessionManager::new(&config.session_file);

    // Initialize supervisor
    let ollama = Ollama::default();
    let mut supervisor = Supervisor::new(ollama, tools.clone())
        .with_memory(memory.clone())
        .with_session(session_manager)
        .with_max_retries(2);

    // Activate continuous thought machine (BitNet)
    let _ = supervisor.activate_background_thinking().await;

    // Restore previous session
    if let Err(e) = supervisor.load_session().await {
        info!("Starting new session (previous session load failed or missing): {}", e);
    } else {
        println!("💾 Session restored from '{}'", config.session_file);
    }

    // Initialize safety guard
    let mut safety = SafetyGuard::new();

    println!("\n💡 Commands: 'quit' | 'history' | 'clear' | 'autonomous' | 'bitnet'\n");

    // Main interaction loop
    loop {
        print!("🤖 You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let query = input.trim();

        if query.is_empty() {
            continue;
        }

        // Handle special commands
        match query.to_lowercase().as_str() {
            "quit" | "exit" | "q" => {
                println!("\n👋 Goodbye!\n");
                break;
            }
            "history" => {
                println!("\n📜 Conversation History:\n{}\n", supervisor.conversation_history());
                continue;
            }
            "clear" => {
                let _ = supervisor.clear_history().await;
                safety.reset();
                println!("\n🗑️  History and session cleared.\n");
                continue;
            }
            "autonomous" => {
                print!("🎯 Define the goal for the Continuous Thought Machine: ");
                io::stdout().flush()?;
                let mut goal = String::new();
                io::stdin().read_line(&mut goal)?;
                let goal = goal.trim();
                
                if goal.is_empty() { continue; }

                println!("\n🚀 Launching Autonomous Mode...\n");
                match supervisor.run_autonomous(goal).await {
                    Ok(result) => {
                        println!("✅ Final Autonomous Result:");
                        println!("{}", result.answer);
                    }
                    Err(e) => println!("❌ Autonomous Machine Error: {}", e),
                }
                continue;
            }
            "bitnet" => {
                print!("⚡ Quick thought prompt: ");
                io::stdout().flush()?;
                let mut p = String::new();
                io::stdin().read_line(&mut p)?;
                let p = p.trim();
                if p.is_empty() { continue; }

                println!("\n🚀 Fast BitNet Inference...\n");
                let call = crate::tools::ToolCall {
                    name: "bitnet_inference".to_string(),
                    parameters: serde_json::json!({ "prompt": p, "task_type": "logic" }),
                };
                match tools.execute(&call).await {
                    Ok(res) => println!("✅ BitNet Thought:\n{}", res.summary),
                    Err(e) => println!("❌ BitNet Error: {}", e),
                }
                continue;
            }
            _ => {}
        }

        // Validate input safety
        if let Err(e) = safety.validate_input(query) {
            println!("\n⚠️  {}\n", e);
            continue;
        }

        // Process the query
        println!("\n⚙️  Processing...\n");

        match supervisor.handle(query).await {
            Ok(result) => {
                // Show plan if used
                if let Some(ref plan) = result.plan {
                    println!("📊 Plan Progress: {:.0}%", plan.progress());
                    for step in &plan.steps {
                        let status = if step.completed { "✓" } else { "○" };
                        println!("   {} Step {}: {} ({})", 
                            status, step.step_num, 
                            step.description,
                            step.agent_type
                        );
                    }
                    println!();
                }

                // Show reflections if any
                if !result.reflections.is_empty() {
                    println!("🔄 Reflections:");
                    for reflection in &result.reflections {
                        println!("   • {}", reflection);
                    }
                    println!();
                }

                // Show the answer
                let status = if result.success { "✅" } else { "⚠️" };
                println!("{} Response:", status);
                println!("{}", "─".repeat(50));
                println!("{}", result.answer);
                println!("{}\n", "─".repeat(50));
            }
            Err(e) => {
                println!("❌ Error: {}\n", e);
            }
        }
    }

    Ok(())
}