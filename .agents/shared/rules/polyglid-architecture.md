---
paths:
  - 'crates/**'
  - 'plugins/**'
  - 'wit/**'
  - 'apps/desktop/**'
  - 'docs/architecture/**'
  - 'docs/security/**'
---

# PolyGlid Architecture

Use this for Rust workspace, WIT, runtime, plugin, CLI, Tauri, and security
workspace changes.

## Boundaries

- `polyglid-core` owns use cases, policies, permissions, orchestration.
- `polyglid-runtime` owns Wasmtime component loading, linking, execution, and
  runtime error translation.
- `polyglid-plugin-api` owns WIT-generated or mirrored shared types.
- `polyglid-config` owns config schema, defaults, validation, migrations, and
  persistence.
- `polyglid-events` owns typed events between host, UI, runtime, and plugins.
- `polyglid-cli` owns development harness commands.
- `apps/desktop` owns Tauri shell, command handlers, and frontend workspace UI.

## Contract Rules

- Change `wit/polyglid.wit` before changing host/plugin behavior.
- Plugin output must be structured data, not parsed terminal text.
- Keep report fields stable; add fields compatibly when possible.
- Regenerate/check host and plugin bindings after contract changes.

## Security Rules

- Plugins are untrusted by default.
- Capabilities are denied by default.
- Validate plugin manifests and targets before execution.
- Do not expose filesystem, process, environment, secret, or raw network access
  without a host capability and tests.
- Runtime failures must not crash the host process.

## Build Order

1. Contract.
2. Shared API types.
3. Runtime adapter.
4. CLI command.
5. Plugin implementation.
6. Tests.
7. Tauri/UI integration.
