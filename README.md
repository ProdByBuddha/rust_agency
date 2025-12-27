# 🚀 SOTA Semi-Autonomous Agency (Rust)

A state-of-the-art, semi-autonomous multi-agent system built in Rust. This agency features a ReAct reasoning framework, semantic vector memory, multi-agent coordination, and a comprehensive tool suite designed to assist users with technical tasks, code analysis, and autonomous problem-solving.

## ✨ Key Features

- **🧠 ReAct Reasoning Framework**: Implements the Reason+Act paradigm, allowing agents to think, plan, act, and observe results iteratively.
- **📚 Semantic Memory**: Integrates **ChromaDB** and **fastembed** for high-performance vector storage and retrieval, enabling long-term memory and context-aware responses.
- **🤖 Multi-Agent Coordination**: Orchestrates specialized agents (e.g., Worker, Reflector, Planner) via a central Supervisor to tackle complex tasks.
- **🛡️ Safety Guardrails**: Built-in input validation, content filtering, and rate limiting to ensure safe and reliable operation.
- **💾 Session Persistence**: Automatically saves and restores conversation history and state, allowing for long-running interactions.
- **⚡ Continuous Thought Machine**: Features a background "BitNet" inference engine for rapid logic processing and entropy-based decision making.
- **🛠️ Extensible Tool System**: A robust registry of built-in and dynamic tools, including web search, code execution, and sandboxed environments.

## 🏗️ Architecture

The system is modular and composed of several core crates:

### 1. Orchestrator
The brain of the operation.
- **Supervisor**: Manages the agent lifecycle, routes queries, and handles tool execution.
- **SessionManager**: Handles serialization and persistence of session data.
- **Planner**: Decomposes complex user requests into executable steps.

### 2. Agents
Specialized entities that perform tasks.
- **ReActAgent**: The primary worker agent that uses tools to solve problems.
- **Reflector**: critiques and improves the output of other agents.
- **AutonomousMachine**: A mode for self-directed goal achievement.

### 3. Memory
- **VectorMemory**: Interfaces with ChromaDB for semantic search.
- **CodebaseIndexer**: Automatically indexes the local codebase for semantic retrieval.
- **MemoryManager**: Tracks resource usage and manages memory entry lifecycles.

### 4. Tools
A diverse set of capabilities available to agents:
- **`web_search`**: Performs internet searches.
- **`code_exec`**: Executes code snippets in a secure environment.
- **`memory_query`**: Retrieves information from the agency's long-term memory.
- **`knowledge_graph`**: Manages structured relationships between data points.
- **`sandbox`**: Provides an isolated environment for file operations and testing.
- **`codebase`**: Analyzes and navigates the local project structure.
- **`bitnet_inference`**: Fast, logic-optimized inference for quick decisions.
- **`forge`**: Allows creation of custom tools on the fly.

## 🚀 Getting Started

### Prerequisites
- **Rust Toolchain**: [Install Rust](https://www.rust-lang.org/tools/install) (1.75+ recommended).
- **Ollama**: [Install Ollama](https://ollama.ai/) and ensure it's running locally (default port 11434).
- **ChromaDB**: Ensure a ChromaDB instance is accessible (if not using an embedded/local setup handled by the crate).

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/yourusername/rust_agency.git
    cd rust_agency
    ```

2.  **Environment Setup:**
    Create a `.env` file in the root directory (copy from `.env.example` if available) or set the necessary variables:
    ```bash
    # Example .env
    RUST_LOG=info
    OLLAMA_HOST=http://localhost:11434
    CHROMA_URL=http://localhost:8000
    ```

3.  **Build the project:**
    ```bash
    cargo build --release
    ```

## 🎮 Usage

Run the agency interactively:

```bash
cargo run --release
```

### Interactive Commands

Once the CLI is running, you can use the following commands:

-   **`autonomous`**: Enter Autonomous Mode. You will be prompted to define a high-level goal, and the agency will attempt to achieve it without further user intervention.
-   **`bitnet`**: Trigger a quick "thought" or logic check using the optimized BitNet inference tool.
-   **`history`**: View the current conversation history.
-   **`clear`**: Wipe the current session history and memory context.
-   **`quit`** / **`exit`**: Save the session and exit the program.

### Example Workflow

**User:** "Analyze the `src/agent/mod.rs` file and explain how the `Agent` trait is defined."

**Agency:**
1.  **Thought:** I need to read the file `src/agent/mod.rs`.
2.  **Action:** Call `codebase_tool` with `read_file`.
3.  **Observation:** Receives file content.
4.  **Response:** "The `Agent` trait is defined as an async trait requiring implementation of `agent_type`, `name`, `system_prompt`, `model`, and an `execute` method..."

## 🔧 Configuration

The agency uses `agency_profile.json` to define its persona:

```json
{
  "name": "The Agency",
  "mission": "To assist the user through specialized multi-agent coordination.",
  "traits": ["efficient", "technical", "autonomous"]
}
```

Modify this file to change how the agent introduces itself and behaves.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is open-source.
