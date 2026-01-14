# Safety Module

"Adversarial-Grade" protection for autonomous agent operations.

## 🛡️ Core Protections

- **Content Filtering (`content_filter.rs`)**: Uses regex-based patterns to detect and block prompt injection, role override attempts, and dangerous code snippets (e.g., fork bombs).
- **Command Safety (`command.rs`)**: A strict whitelist/blacklist heuristic for shell commands. Blocks destructive operations like `rm -rf /` or `git reset --hard` unless specifically authorized.
- **Assurance Scoring (`assurance.rs`)**: Real-time F-G-R calculation for every tool call. Blocks execution if the reliability score drops below the trust threshold.

## 🔒 Process Hardening (`hardening.rs`)

Implements OS-level security features:
- Disables core dumps.
- Prevents ptrace attachment (on macOS).
- Scrubs dangerous environment variables (e.g., `DYLD_` / `LD_`).

## 🚦 Operational Controls

- **Rate Limiter (`rate_limiter.rs`)**: Token-bucket algorithm to prevent resource abuse.
- **Human-in-the-Loop (HITL)**: Automatically pauses execution and requests manual approval for high-risk operations or low-assurance plans.
