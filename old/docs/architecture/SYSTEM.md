# System Architecture

PolyGlid is planned as a multi-layer workspace:

```text
+------------------------------------------------------------------------+
|                         Frontend Workspace                             |
| TypeScript, React or SolidJS, Tailwind, multi-window workspace views    |
+-----------------------------------+------------------------------------+
                                    |
                                    | Tauri IPC, commands, events
                                    v
+------------------------------------------------------------------------+
|                         Rust Host Engine                               |
| config, permissions, task queue, event router, plugin registry          |
+-----------------------------------+------------------------------------+
                                    |
                                    | Wasmtime Component Model
                                    v
+------------------------------------------------------------------------+
|                         WASM Plugin Runtime                            |
| isolated plugin components, WIT contracts, capability-based host calls  |
+------------------------------------------------------------------------+
```

## Frontend Workspace

The frontend is the operator surface. It manages views, tabs, panels, settings pages, and live output displays.

Responsibilities:

- workspace navigation
- split panes and detached windows
- plugin marketplace or plugin list
- scan/task forms
- structured report rendering
- live logs and progress views
- settings and permissions UI

The frontend should never run security logic directly. It asks the host engine to start tasks.

## Rust Host Engine

The host engine is the trusted center of the application.

Responsibilities:

- read and validate app configuration
- load plugin metadata
- enforce permission decisions
- execute plugins through the runtime
- own task scheduling and cancellation
- emit events to all active windows
- persist settings

The host should be small, explicit, and boring. The plugins can grow, but the host boundary must stay stable.

## WASM Plugin Runtime

Plugins are compiled as WebAssembly components and loaded through Wasmtime.

Responsibilities:

- run plugins in isolated memory
- expose only approved host capabilities
- convert plugin output into typed events
- stop plugins on cancellation, timeout, or policy failure
- report runtime errors without crashing the host

## Contract Boundary

The WIT contract is the rulebook between host and plugin.

```text
Host knows:
- what function a plugin exports
- what input shape it accepts
- what output shape it returns

Plugin knows:
- what data it receives
- what report/events it must return
- what host capabilities it may request
```

## Event Flow

```text
User starts task in UI
        |
        v
Tauri command enters host core
        |
        v
Core validates target and permissions
        |
        v
Runtime starts WASM plugin
        |
        v
Plugin emits structured progress/report events
        |
        v
Host broadcasts events to frontend windows
```

## Configuration Flow

```text
User changes setting
        |
        v
Frontend calls Tauri command
        |
        v
Host validates config update
        |
        v
Config store persists setting
        |
        v
Event bus broadcasts config-changed
        |
        v
All windows update without restart
```

