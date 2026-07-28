# Domain and Core Scope

## Layer Responsibilities

- `crates/client` exposes UI-independent models, controllers, and
  `ClientGateway`.
- `LocalClient` composes local services and translates domain types for clients.
- `crates/core` owns services, persistence, security decisions, and execution.
- `crates/config`, `events`, and `plugin-api` hold shared contracts.
- `crates/runtime` hosts Wasmtime components.
- `contracts/polyglid.wit` is the host/plugin ABI.

## Rules

- Keep the gateway independent from Dioxus and any future UI framework.
- Return typed or contextual errors across each boundary.
- Keep database and filesystem operations out of view components.
- Use transactions for multi-step persistence changes.
- Validate names as one safe path segment before filesystem operations.
- Maintain project deletion, capability, signature, output-path, and runtime
  isolation guarantees.
- Treat WIT changes as cross-layer contract changes and test both host and
  plugin-facing conversions.
