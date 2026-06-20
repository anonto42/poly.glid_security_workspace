# PolyGlid Security Workspace

PolyGlid is planned as a cross-platform security workspace built around a Rust host engine, a web-based multi-window UI, and sandboxed WebAssembly plugins.

The goal is not to create one large tangled application. The goal is to create a small trusted host that manages windows, state, permissions, and execution, while feature plugins run behind explicit contracts and capability boundaries.

## Product Shape

```text
Frontend Workspace
React or SolidJS, TypeScript, Tailwind
        |
        | Tauri IPC and event streams
        v
Rust Host Engine
Tauri v2, Tokio, config, permissions, scheduling
        |
        | Wasmtime Component Model
        v
Sandboxed WASM Plugins
Recon, audit, reporting, diagnostics
```

## Core Principles

- The host is trusted. Plugins are not trusted by default.
- Plugins never directly own filesystem, process, or network access.
- Every host/plugin boundary is described through a stable contract.
- Every plugin returns structured data instead of free-form terminal text.
- The full GUI is not required to test plugin behavior. A CLI harness comes first.
- Security features must be designed for authorized testing and defensive validation.

## First Documentation Pass

- [Codebase Architecture](docs/architecture/CODEBASE.md)
- [Repository Layout](docs/architecture/REPOSITORY_LAYOUT.md)
- [System Architecture](docs/architecture/SYSTEM.md)
- [Development Targets](docs/planning/DEVELOPMENT_TARGETS.md)
- [Roadmap](docs/planning/ROADMAP.md)
- [MVP Definition](docs/planning/MVP.md)
- [Security Model](docs/security/SECURITY_MODEL.md)
- [Development Workflow](docs/development/WORKFLOW.md)
- [Packaging And Distribution](docs/development/PACKAGING.md)
- [Brand Guide](docs/branding/README.md)

## Planned Repository Layout

```text
poly.glid_security_workspace/
├── apps/
│   └── desktop/              # Tauri desktop/mobile app shell
├── crates/
│   ├── polyglid-core/        # application use cases and orchestration
│   ├── polyglid-runtime/     # Wasmtime runtime and plugin execution
│   ├── polyglid-plugin-api/  # shared Rust types generated from WIT
│   ├── polyglid-config/      # config loading, validation, persistence
│   ├── polyglid-events/      # event model between host/UI/plugins
│   └── polyglid-cli/         # development harness and maintenance CLI
├── plugins/
│   └── recon_probe/          # first safe example plugin
├── wit/
│   └── polyglid.wit          # host/plugin contract
└── docs/
```

## Build Order

1. Define the WIT contract.
2. Build the Rust plugin runtime and CLI harness.
3. Build one harmless diagnostic plugin.
4. Add config and permission models.
5. Add the Tauri shell.
6. Add the workspace UI.
7. Add richer plugins only after the boundary is tested.
