# Codebase Architecture

PolyGlid should be organized as a Rust workspace with a separate frontend app. The architecture follows a hexagonal style inside the Rust backend: the core owns decisions, while adapters handle external systems such as Wasmtime, Tauri, storage, networking, and the filesystem.

## High-Level Layout

```text
apps/desktop
  Tauri application shell and frontend workspace.

crates/polyglid-core
  Pure application logic. Owns use cases, workflows, scheduling decisions,
  permission checks, and task orchestration.

crates/polyglid-runtime
  Wasmtime adapter. Loads components, links host capabilities, invokes plugin
  functions, streams plugin output, and translates runtime errors.

crates/polyglid-plugin-api
  Shared host/plugin types. Generated or mirrored from WIT contracts.

crates/polyglid-config
  Config schemas, defaults, migrations, validation, and persistence.

crates/polyglid-events
  Typed events used between UI, host, runtime, and background tasks.

crates/polyglid-cli
  Development harness. Runs plugins from the terminal before the GUI exists.

plugins/*
  Independent WASM component plugins.

wit
  WebAssembly Interface Type contracts.
```

## Hexagonal Backend

```text
                         Tauri Commands
                              |
                              v
                    +--------------------+
                    |  polyglid-core    |
                    |  use cases        |
                    +--------------------+
                      ^    ^     ^    ^
                      |    |     |    |
        ConfigPort ---+    |     |    +--- EventBusPort
                           |     |
             RuntimePort --+     +--- PermissionPort
                           |
                     StoragePort
```

The core should depend on traits, not concrete external tools.

Example ports:

```rust
pub trait PluginRuntime {
    fn inspect(&self, plugin: PluginRef) -> Result<PluginManifest>;
    fn execute(&self, request: PluginRunRequest) -> Result<PluginRunHandle>;
}

pub trait PermissionStore {
    fn is_allowed(&self, plugin_id: &PluginId, capability: Capability) -> Result<bool>;
}

pub trait EventSink {
    fn emit(&self, event: PolyGlidEvent) -> Result<()>;
}

pub trait ConfigStore {
    fn load(&self) -> Result<AppConfig>;
    fn save(&self, config: &AppConfig) -> Result<()>;
}
```

## Dependency Rule

The dependency direction should move inward:

```text
apps/desktop -> crates/polyglid-core
apps/desktop -> crates/polyglid-runtime
crates/polyglid-runtime -> crates/polyglid-plugin-api
crates/polyglid-core -> crates/polyglid-config
crates/polyglid-core -> crates/polyglid-events
```

The core must not directly depend on Tauri, Wasmtime, or frontend code. That keeps the business logic testable from normal Rust unit tests.

## First Rust Workspace Plan

```toml
[workspace]
members = [
  "crates/polyglid-core",
  "crates/polyglid-runtime",
  "crates/polyglid-plugin-api",
  "crates/polyglid-config",
  "crates/polyglid-events",
  "crates/polyglid-cli",
  "plugins/recon_probe",
]
resolver = "2"
```

## Crate Responsibilities

| Crate | Owns | Does Not Own |
| --- | --- | --- |
| `polyglid-core` | use cases, policies, task orchestration | Tauri commands, Wasmtime internals |
| `polyglid-runtime` | component loading, linking, execution | UI state, product workflows |
| `polyglid-plugin-api` | shared types and WIT-generated bindings | runtime policy |
| `polyglid-config` | config schema and validation | plugin execution |
| `polyglid-events` | typed event names and payloads | transport implementation |
| `polyglid-cli` | dev harness commands | product UI |

## Codebase Rule

When adding a feature, ask:

1. Is this product behavior? Put it in `polyglid-core`.
2. Is this a host/plugin contract? Put it in `wit` and `polyglid-plugin-api`.
3. Is this Wasmtime-specific? Put it in `polyglid-runtime`.
4. Is this UI-only? Put it in `apps/desktop`.
5. Is this only for development/testing? Put it in `polyglid-cli`.

