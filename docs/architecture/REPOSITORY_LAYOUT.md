# Repository Layout

PolyGlid uses a monorepo layout because the project contains multiple related pieces that must evolve together:

- desktop app
- Rust engine
- plugin runtime
- CLI client
- WIT contracts
- first-party plugins
- documentation

## Layout

```text
poly.glid_security_workspace/
├── apps/
│   └── desktop/
├── crates/
│   ├── polyglid-cli/
│   ├── polyglid-core/
│   ├── polyglid-runtime/
│   ├── polyglid-plugin-api/
│   ├── polyglid-config/
│   └── polyglid-events/
├── plugins/
│   └── recon_probe/
├── wit/
│   └── polyglid.wit
└── docs/
```

## `apps`

Contains user-facing applications.

For now:

```text
apps/desktop
```

This will become the Tauri app. It should contain frontend code, Tauri commands, and app shell wiring. It should not contain core plugin execution logic.

## `crates`

Contains Rust crates that form the engine.

### `polyglid-cli`

The first client interface.

It owns:

- command parsing
- terminal output
- developer-friendly plugin testing commands
- calls into `polyglid-core`

It does not own:

- Wasmtime internals
- business policy
- plugin permission rules

### `polyglid-core`

The application brain.

It owns:

- use cases
- permission decisions
- task orchestration
- engine-level validation
- business rules

It does not own:

- terminal UI
- Tauri UI
- Wasmtime implementation details

### `polyglid-runtime`

The WASM execution adapter.

It owns:

- Wasmtime setup
- component loading
- runtime limits
- plugin execution
- runtime error mapping

### `polyglid-plugin-api`

Shared contract types.

It owns:

- generated WIT bindings
- shared report/event structs
- plugin-facing types where needed

### `polyglid-config`

Configuration and settings.

It owns:

- config structs
- default values
- config validation
- config migrations later

### `polyglid-events`

Typed events.

It owns:

- event names
- event payloads
- task progress events
- report events

## `plugins`

Contains first-party plugins. Each plugin must remain independently buildable as a WASM component.

First plugin:

```text
plugins/recon_probe
```

## `wit`

Contains the WebAssembly Component Model contract.

The WIT file is the language-agnostic rulebook between the host engine and plugins. If the WIT contract changes, both host and plugin bindings must be updated.

## `docs`

Contains project planning, architecture, security, development, and brand documentation.

Docs are part of the product. They explain the reasoning behind the codebase so future implementation does not drift.

Important development docs:

- `docs/development/WORKFLOW.md`
- `docs/development/PACKAGING.md`

## Why This Layout Works

The CLI, desktop app, and future clients can all reuse the same core:

```text
polyglid-cli   ----\
                   +--> polyglid-core --> polyglid-runtime --> plugins
apps/desktop   ----/
```

This prevents duplicated logic and keeps the project testable while it grows.
