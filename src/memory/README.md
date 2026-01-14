# Memory System

Implements the "Intelligence Fabric" for long-term storage and real-time context retrieval.

## 💾 Semantic Vector Memory (`vector.rs`)

- **Fastembed Integration**: High-performance local embeddings using the `AllMiniLML6V2` model.
- **Microservice Ready**: Supports both local storage and remote `memory_server` backends via environment toggles.
- **Hash Deduplication**: Prevents redundant indexing of static codebase artifacts.

## 🕰️ Episodic Memory (`episodic.rs`)

- **Sliding Window**: Maintains recent conversation turns for prompt injection.
- **Context Compaction**: Uses a secondary LLM cycle to summarize long histories into "Distilled Decision Logs" when token limits are reached.

## 🧠 Memory Management (`manager.rs`)

- **Resource Awareness**: Monitors RAM/VRAM usage and triggers automatic cache hibernation during critical low-memory states.
- **Fact Distillation**: Periodically extracts long-term "facts" from chat history to promote them into the permanent vector store.

## 🔍 Codebase Indexer (`indexer.rs`)

Recursively crawls the project structure to build a semantic index of source code, enabling agents to "understand" their own implementation.
