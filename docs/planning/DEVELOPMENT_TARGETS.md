# Development Targets

This document defines how PolyGlid should be developed step by step. The first version is intentionally CLI-first so the core engine and plugin system can be tested before the desktop UI exists.

## Main Development Idea

PolyGlid will support multiple client-side interaction surfaces:

```text
CLI client first
Desktop UI later
Possible web/mobile surfaces later
```

All clients should talk to the same backend concepts. The CLI is not a throwaway script. It is the first real client for the PolyGlid engine.

```text
polyglid-cli
     |
     v
polyglid-core
     |
     v
polyglid-runtime
     |
     v
WASM plugins
```

Later, the Tauri desktop app will use the same core:

```text
Tauri desktop UI
     |
     v
polyglid-core
     |
     v
polyglid-runtime
     |
     v
WASM plugins
```

## Target 1: Organize The Codebase

Before writing feature code, the repository needs the permanent shape:

```text
apps/
  desktop/

crates/
  polyglid-cli/
  polyglid-core/
  polyglid-runtime/
  polyglid-plugin-api/
  polyglid-config/
  polyglid-events/

plugins/
  recon_probe/

wit/
  polyglid.wit

docs/
```

Why:

- `apps/desktop` keeps Tauri UI code away from core engine code.
- `polyglid-cli` becomes the first interface for humans and developers.
- `polyglid-core` owns product behavior and use cases.
- `polyglid-runtime` owns Wasmtime and plugin execution.
- `polyglid-plugin-api` owns shared host/plugin contracts.
- `polyglid-config` owns settings and validation.
- `polyglid-events` owns typed events between layers.
- `plugins/*` keeps each plugin independent.
- `wit` is the source of truth for host/plugin communication.

## Target 2: Design The CLI Interface

The CLI should first show what PolyGlid is and how the engine will behave.

First commands:

```bash
polyglid --help
polyglid doctor
polyglid plugin list
polyglid plugin inspect ./plugin.wasm
polyglid plugin run ./plugin.wasm --target example.com
polyglid config show
polyglid config validate
```

The first CLI output should feel like a real product shell:

```text
PolyGlid Security Workspace

Engine:    not initialized
Runtime:   wasmtime unavailable
Plugins:   0 discovered
Config:    valid

Next:
  polyglid plugin run ./plugins/recon_probe.wasm --target example.com
```

This stage proves the command shape and user experience before complex internals exist.

## Target 3: Build The Core Engine

After the CLI shape is clear, build the core engine behind it.

Core use cases:

- inspect plugin
- validate config
- prepare plugin run request
- check permissions
- start plugin execution
- receive plugin report
- convert output into events

The CLI should call core use cases. It should not directly call Wasmtime.

## Target 4: Build The Plugin Runtime

The runtime is the adapter between `polyglid-core` and WebAssembly.

Responsibilities:

- load `.wasm` components
- bind WIT interfaces
- apply runtime limits
- execute plugin entrypoints
- return structured reports
- convert runtime failures into safe errors

The runtime should be tested through the CLI before the desktop app is started.

## Target 5: Build The First Plugin

The first plugin should be safe and simple.

Recommended plugin: `recon_probe`.

Initial behavior:

- accepts a target string
- validates the target shape
- returns a structured report
- does not perform aggressive scanning
- does not require privileged host access

The goal is to prove the plugin contract, not to build a powerful scanner in version one.

## Target 6: Add The Desktop UI

Only after the CLI can run the engine and plugin system should the Tauri UI begin.

Desktop first features:

- workspace shell
- plugin run form
- report viewer
- settings page
- permission display
- live event output

The desktop UI should reuse the same core use cases already proven through the CLI.

## Version 0.1 Success Criteria

Version 0.1 is complete when:

- the Rust workspace builds
- `polyglid --help` works
- `polyglid doctor` shows local readiness
- `polyglid config validate` validates app config
- `polyglid plugin run` can execute the first WASM plugin
- plugin output is structured
- denied permissions are handled cleanly
- the CLI and docs explain the architecture clearly

