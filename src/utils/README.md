# Utilities

Shared helper functions and cross-cutting concerns.

## 🧱 Key Utilities

- **Text Truncation (`truncate.rs`)**: Robust UTF-8 aware truncation. Supports "Double-Ended" truncation (preserving prefix and suffix) to keep the most important context.
- **Observability (`otel.rs`)**: Integration with OpenTelemetry. Provides distributed tracing and span exporters for deep system debugging.
- **Environment Management**: Helpers for loading `.env` files and managing hardware-specific toggles (e.g., `FORCE_CPU`).
