# Tools Module

The "Physical Hand" of the agency. Implements structured tool calling and self-expansion capabilities.

## 🛠️ Built-in Tools

- **`codebase_explorer.rs`**: High-fidelity file reading and directory traversal with integrated safety whitelists.
- **`code_exec.rs`**: Sandboxed execution of Python, Rust, and Node.js.
- **`web_search.rs`**: Real-time information retrieval using DuckDuckGo.
- **`artifact_manager.rs`**: Persistent storage for agent-generated outputs.

## 🔨 Tool Forging (`dynamic.rs`)

The agency features **Self-Expansion**:
- **Forge Tool**: Allows agents to write new Rust/Python scripts and dynamically register them as permanent tools in the registry.
- **Laboratory Promotion**: New tools start in an experimental "laboratory" state and are promoted to "standard" only after successful validation.

## 🔌 Integration Standards

- **Model Context Protocol (MCP)**: Implements the MCP client spec, allowing the agency to connect to external tool servers (e.g., SQLite, GitHub, Brave Search) over stdio.
- **Markdown Skills**: Discovers new capabilities by reading `.md` files containing YAML frontmatter instructions.
- **Security Oracles**: Every tool implements a `security_oracle` gate to validate parameters before execution.
