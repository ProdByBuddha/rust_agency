//! SOTA Semi-Autonomous Agency
//! 
//! A state-of-the-art Rust-based multi-agent system featuring:
//! - ReAct reasoning framework with tool calling
//! - Semantic vector memory (ChromaDB + fastembed)
//! - Multi-agent coordination with planning
//! - Self-reflection and error correction
//! - Safety guardrails
//! - Full session persistence

use rust_agency::tools::McpServer;
use anyhow::Result;
use ollama_rs::Ollama;
use std::sync::Arc;
use tracing::info;

use rust_agency::memory::{Memory, VectorMemory, MemoryManager};
use rust_agency::orchestrator::{Supervisor, SessionManager, Speaker, profile::ProfileManager};
use rust_agency::tools::{
    Tool, ToolRegistry, WebSearchTool, CodeExecTool, MemoryQueryTool, 
    KnowledgeGraphTool, ArtifactTool, SandboxTool, CodebaseTool, 
    SystemTool, ForgeTool, VisualizationTool, 
    SpeakerRsTool, ScienceTool, ModelManager
};

// ──────────────────────────────────────────────────────────────────────────────
// CONFIGURATION
// ──────────────────────────────────────────────────────────────────────────────

/// Configuration for the agency
struct AgencyConfig {
    /// Path to fallback memory file
    memory_file: String,
    /// Path to session persistence file
    session_file: String,
    /// Path to agency profile file
    profile_file: String,
}

impl Default for AgencyConfig {
    fn default() -> Self {
        Self {
            memory_file: "memory.json".to_string(),
            session_file: "session.json".to_string(),
            profile_file: "agency_profile.json".to_string(),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MAIN ENTRY POINT
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // SOTA: Apply Process Hardening (codex-inspired)
    rust_agency::safety::hardening::apply_hardening();

    // SOTA: Professional Observability (OpenTelemetry)
    let _otel_guard = rust_agency::utils::otel::init_telemetry("rust_agency")
        .expect("Failed to initialize OpenTelemetry");

    // Load environment variables IMMEDIATELY
    dotenv::dotenv().ok();

    // Check for CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--visualize" || args[1] == "-v") {
        let tool = VisualizationTool::new();
        let params = serde_json::json!({
            "output_file": args.get(2).map(|s| s.as_str()).unwrap_or("agency_isometric.json")
        });
        match tool.execute(params).await {
            Ok(res) => {
                println!("{}", res.summary);
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Error generating visualization: {}", e);
                std::process::exit(1);
            }
        }
    }

    println!("\n{}", "═".repeat(60));
    println!("🚀 SOTA Semi-Autonomous Agency v0.2.0");
    println!("{}", "═".repeat(60));
    println!("Features: ReAct | Vector Memory | Multi-Agent | Planning | Telemetry");
    println!("{}\n", "═".repeat(60));

    let config = AgencyConfig::default();
    
    // Initialize memory system
    let memory: Arc<dyn Memory> = Arc::new(
        VectorMemory::new(&config.memory_file)
            .expect("Failed to initialize memory system")
    );
    
    // Initialize MemoryManager for resource tracking
    let manager = Arc::new(MemoryManager::new(memory.clone()));

    // Primary LLM Provider: Use Remote Nexus (Llama 3.2 3B) to avoid reload lag
    println!("🌐 Connecting to Remote Nexus Model Server (Llama 3.2 3B)...");
    let provider: Arc<dyn rust_agency::agent::LLMProvider> = Arc::new(rust_agency::agent::RemoteNexusProvider::new());

    // Initialize session persistence
    let session_manager = SessionManager::new(&config.session_file);

    // Initialize shared Speaker engine (Deduplicated SOTA)
    let shared_speaker = Arc::new(tokio::sync::Mutex::new(Speaker::new()?));

    // Initialize tools
    let tools = Arc::new(ToolRegistry::default());
    
    // SOTA: Concurrent Tool Registration (FPF Principle: Rapid Capability Establishment)
    tokio::join!(
        tools.register_instance(WebSearchTool::new()),
        tools.register_instance(CodeExecTool::new()),
        tools.register_instance(MemoryQueryTool::new(memory.clone())),
        tools.register_instance(KnowledgeGraphTool::new(memory.clone())),
        tools.register_instance(ArtifactTool::default()),
        tools.register_instance(SandboxTool::default()),
        tools.register_instance(CodebaseTool::default()),
        tools.register_instance(ModelManager),
        tools.register_instance(SpeakerRsTool::new(shared_speaker.clone())),
        tools.register_instance(VisualizationTool::new()),
        tools.register_instance(ScienceTool::new()),
        tools.register_instance(ForgeTool::new("custom_tools", tools.clone())),
        tools.register_instance(SystemTool::new(manager.clone()))
    );

    // SOTA: Markdown-Based Skill Discovery (pi-mono-inspired)
    if let Ok(skills) = rust_agency::tools::SkillLoader::discover_skills("skills").await {
        for skill in skills {
            let name = skill.name();
            tools.register_instance(skill).await;
            println!("📚 Discovered Skill: {}", name);
        }
    }

    // SOTA: Dynamic MCP Server Integration
    if let Ok(content) = std::fs::read_to_string("mcp_servers.json") {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(servers) = config["servers"].as_array() {
                for server_cfg in servers {
                    let name = server_cfg["name"].as_str().unwrap_or("unnamed");
                    let command = server_cfg["command"].as_str().unwrap_or("");
                    let args: Vec<String> = server_cfg["args"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();

                    if !command.is_empty() {
                        match McpServer::spawn(name, command, &args).await {
                            Ok(server) => {
                                match tools.register_mcp_server(server).await {
                                    Ok(count) => println!("🔌 Connected to MCP Server '{}' ({} tools loaded)", name, count),
                                    Err(e) => tracing::warn!("Failed to register tools from MCP server '{}': {}", name, e),
                                }
                            }
                            Err(e) => tracing::warn!("Failed to spawn MCP server '{}': {}", name, e),
                        }
                    }
                }
            }
        }
    }

    // SOTA: Load all previously forged dynamic tools (Laboratory graduation)
    let _ = tools.load_dynamic_tools("standard_tools").await;
    if let Ok(count) = tools.load_dynamic_tools("custom_tools").await {
        if count > 0 {
            println!("🛠️  Loaded {} dynamic tools from laboratory ('custom_tools').", count);
        }
    }
    let profile_manager = ProfileManager::new(&config.profile_file);
    let profile = profile_manager.load().await.unwrap_or_default();
    println!("👤 Agency Profile loaded: {}", profile.name);

    // Initialize supervisor
    let mut supervisor = Supervisor::new_with_provider(provider, tools.clone())
        .with_memory(memory.clone())
        .with_session(session_manager)
        .with_profile(profile)
        .with_max_retries(2);

    // NOTE: Background thinking (CTM) is disabled by default to save resources on 16GB M2 Air.
    // To enable it, uncomment the following line or use the 'autonomous' command.
    // let _ = supervisor.activate_background_thinking().await;

    // Restore previous session
    if let Err(e) = supervisor.load_session().await {
        info!("Starting new session (previous session load failed or missing): {}", e);
    } else {
        println!("💾 Session restored from '{}'", config.session_file);
    }

    // Launch the professional FPF-aligned CLI with SHARED speaker
    let mut cli = rust_agency::orchestrator::cli::AgencyCLI::new(supervisor, shared_speaker);
    cli.run().await?;

    Ok(())
}