# PolyGlid System Architecture (v0.9.0)

This document is the permanent living reference for PolyGlid's architectural principles, crate responsibilities, execution pipelines, and security designs.

---

## 1. Crate Architecture & Design Philosophy
PolyGlid enforces strict decoupling through compiler-enforced crate boundaries:

- **`polyglid-cli`**: Terminal user interfaces including the Ratatui Dashboard (TUI) and standard CLI matchers. Does not import `rusqlite` or access SQLite directly.
- **`polyglid-desktop`**: Rust/Dioxus desktop UI and application adapter. It
  consumes typed core services/events and does not import `rusqlite`, manipulate
  the database directly, or invoke Wasmtime directly.
- **`polyglid-core`**: Core domain logic and business rules. Houses:
  - `plugin_manager`: Lifecycle actions (validate, install, uninstall).
  - `execution`: Task scheduling, job queues, and serialization format exporters (JSON, HTML, Markdown, SARIF).
  - `store`: SQLite schema repositories and user version database migrations.
  - `security`: Cryptographic checks (Ed25519 verifiers, trust stores, permission engines, audit loggers).
- **`polyglid-runtime`**: Guest component execution runtime. Integrates `wasmtime` and WASI reactor environments.
- **`polyglid-events`**: Decoupled events propagation and dispatching hooks.
- **`polyglid-plugin-api`**: Domain definitions of capabilities, versions, and WIT exports/imports.

---

## 2. Decoupled Execution Pipeline
Execution flows downwards from the UI layer to sandboxed guest execution:

```text
[ UI/CLI Frontend ] ──► [ ExecutionManager ] ──► [ CoreEngine::run_plugin ] ──► [ WasmRuntime ] ──► [ Guest WASM ]
```

1. **Submission**: User launches a scan target.
2. **Scheduling**: `ExecutionManager` allocates a job ID and queues the execution in the SQLite history database.
3. **Database Checks**: `CoreEngine::run_plugin` reads `polyglid.db` to verify:
   - Plugin status is not `Disabled`.
   - Plugin signature matches the active `SecurityProfile` requirement.
   - Capability permissions are consented and not expired.
4. **Sandboxed Run**: Wasmtime loads the component, configures WASI file/DNS/network capabilities, and runs the guest code.

---

## 3. Cryptographic Signature & Verification Stages
Validation consists of three independent serial checkpoints:

```text
WASM Component File ──► [ 1. Signature check ] ──► [ 2. Publisher trust check ] ──► [ 3. Capability approval ]
```

- **Stage 1 (Authenticity)**: Verifies detached Ed25519 `.sig` signatures.
- **Stage 2 (Identity)**: Checks publisher fingerprints in `trusted_publishers` table.
- **Stage 3 (Permission Consent)**: Resolves user capability approvals and expiration intervals from SQLite.
- **Security Profiles**: Policies (`Strict`, `Balanced`, `Development`) dynamically control validation strictness and sandbox fuel resource limits.
- **Audit Trails**: Events write structured metadata payloads directly to `audit_logs`.
