# GitHub Copilot instructions for contributors and AI agents

## Quick summary
- This repo is a Rust-based, semi-autonomous multi-agent system with a microservices architecture: **Nexus (orchestrator)**, **Memory**, **Speaker**, **Listener**. Startup is orchestrated by `./start_agency.sh` and can be selectively enabled via `.env` flags.
- Key design goals: ReAct reasoning, modular tools (`src/tools/`), dynamic MCP integration (`mcp_servers.json`), local model support (Candle/ONNX under `artifacts/chatterbox/`), and FPF capability scoping (`U.WorkScope`).

## What to read first (context files)
- The repository aggregates context files (for agent guidance) by walking up directories looking for: `AGENTS.md`, `CLAUDE.md`, `.cursorrules`, `.windsurfrules`. See `src/orchestrator/context.rs` (ContextLoader).
- Pack- and skill-level documentation lives under `Personal_AI_Infrastructure/Bundles/` and `skills/` — read these to learn agent persona, workflows, and builtin skills.

## Build & run (dev quick commands)
- Build only necessary binaries (release):
  ```bash
  # default build used by start script
  cargo build --release --bin memory_server --bin nexus_server
  # include speaker and listener when enabled
  cargo build --release --bin speaker_server --bin listener_server
  ```
- Start full system (build + launch microservices + healthchecks):
  ```bash
  ./start_agency.sh
  ```
  The script uses `.env` flags: `AGENCY_ENABLE_MOUTH=1` (speaker), `AGENCY_ENABLE_EARS=1` (listener), `AGENCY_SPEAKER_PORT`, `AGENCY_MEMORY_PORT`.
- Run a single component interactively:
  ```bash
  cargo run --bin nexus_server
  cargo run --bin speaker_server
  ```

## Key runtime conventions & health checks
- `start_agency.sh` performs health checks by curling `http://localhost:$PORT/health` (speaker) and `http://localhost:$MEM_PORT/health` (memory). Ensure ports are set in `.env`.
- Logs for services are written to `speaker_server.log`, `listener_server.log`, `memory_server.log`, and `nexus_server.log` when the start script runs.

## Where models & artifacts live
- Local model artifacts (ONNX/Safetensors) go under `artifacts/chatterbox/`.
- Dockerized Speaker expects `HF_TOKEN` (see `docker-compose.yml` and `Dockerfile.speaker`) and mounts `~/.cache/huggingface` to avoid repeated downloads.

## Tools, Skills, and Extensibility
- Tools live in `src/tools/` (e.g., `web_search`, `code_exec`, `memory_query`, `forge`, `mcp`). Follow the patterns there when adding tools.
- Static skill documentation is Markdown in `skills/` and `Personal_AI_Infrastructure/Packs/*/README.md` — use these to teach the agent new procedures or static workflows.
- External MCP servers are registered in `mcp_servers.json` — examples in README and `mcp_servers.json` show the `command` + `args` shape used by the runtime.

## Agent behavioral context (important for AI agents)
- The orchestrator will aggregate hierarchical guidance from `AGENTS.md` / `CLAUDE.md` files found in parent directories. When producing agent behavior or code, consult these files first for repository-specific style and constraints.
- Use `agency_profile.json` to understand persona, mission, and mission scope. Changes to persona should be conservative and documented.

## Observability & safety
- Tracing & metrics are enabled via OpenTelemetry; Jaeger is included in `docker-compose.yml` for local debugging (`16686` UI).
- The project uses process hardening and sandboxing (Podman) for code execution. Respect these safety mechanisms when adding new features.

## Tests, linting & CI
- Unit tests can be run with `cargo test`.
- Clippy configs exist in sub-crates (`clippy.toml`); prefer linting before submitting PRs.
- There are repository-level CI workflows that reference CLAUDE.md for automated code review guidance — align PR suggestions with those conventions.

## Common gotchas & non-obvious patterns
- The context loader collects `AGENTS.md`/`CLAUDE.md` from parent directories — adding or editing these files will change agent behavior during runtime.
- The start script chooses which binaries to build based on `.env` flags. If a service is enabled but the port is already occupied, the script assumes the service is already running.
- Candle crates are included as local path dependencies and sometimes require Metal/accelerate features on macOS — test locally with the same feature set in `Cargo.toml`.

## If you modify or add files
- Add documentation under `skills/` or pack README files to make your new capability discoverable by the context loader.
- When adding a service, expose a `/health` endpoint to integrate with `start_agency.sh`'s health checks.
- Add tests for behavior-driven pieces (integration tests under `tests/` and unit tests per crate).

---
If anything above is unclear or you want more detail on a specific area (eg: MCP integration, how to add a new tool, or how to wire up Candle models), please tell me which section to expand and I will iterate. 

*— GitHub Copilot (Raptor mini (Preview))*