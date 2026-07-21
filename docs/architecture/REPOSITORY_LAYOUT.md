# Repository Layout

The layout below remains the monorepo map. Client priority and boundaries are
defined in [Client Architecture](CLIENT_ARCHITECTURE.md); earlier CLI-first and
Tauri references are superseded by the Dioxus desktop-first direction.

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
│   ├── desktop/
│   ├── cli/
│   └── server/
├── crates/
│   ├── core/
│   ├── runtime/
│   ├── plugin-api/
│   ├── config/
│   └── events/
├── contracts/
│   └── polyglid.wit
├── plugins/
│   └── recon-probe/
├── site/
├── sdk/
├── tools/
├── scripts/
├── infrastructure/
├── tests/
└── docs/
```

## `apps`

Contains user-facing applications.

For now:

```text
apps/desktop
```

The Dioxus desktop app contains workbench presentation and backend adapters. It does not own core plugin execution policy.

## `crates`

Contains Rust crates that form the engine.

### `apps/cli`

The frozen development and regression harness.

It owns:

- command parsing
- terminal output
- developer-friendly plugin testing commands
- calls into `polyglid-core`

It does not own:

- Wasmtime internals
- business policy
- plugin permission rules

### `crates/core`

The application brain.

It owns:

- use cases
- permission decisions
- task orchestration
- engine-level validation
- business rules

It does not own:

- terminal UI
- Dioxus UI
- Wasmtime implementation details

### `crates/runtime`

The WASM execution adapter.

It owns:

- Wasmtime setup
- component loading
- runtime limits
- plugin execution
- runtime error mapping

### `crates/plugin-api`

Shared contract types.

It owns:

- generated WIT bindings
- shared report/event structs
- plugin-facing types where needed

### `crates/config`

Configuration and settings.

It owns:

- config structs
- default values
- config validation
- config migrations later

### `crates/events`

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
plugins/recon-probe
```

## `contracts`

Contains the WebAssembly Component Model contract.

The WIT file is the language-agnostic rulebook between the host engine and plugins. If the WIT contract changes, both host and plugin bindings must be updated.

## `docs`

Contains project planning, architecture, security, development, and brand documentation.

Docs are part of the product. They explain the reasoning behind the codebase so future implementation does not drift.

Important development docs:

- `docs/development/WORKFLOW.md`
- `docs/development/PACKAGING.md`

## Why This Layout Works

The desktop app is the primary client. The CLI harness and future clients reuse
the same core boundaries:

```text
apps/desktop   ----\
                   +--> polyglid-core --> polyglid-runtime --> plugins
apps/cli       ----/
```

This prevents duplicated logic and keeps the project testable while it grows.
