# PolyGlid Security Workspace

PolyGlid is a cross-platform security workspace built around a Rust host engine, a Dioxus desktop workbench, CLI and server clients, and sandboxed WebAssembly plugins.

The goal is not to create one large tangled application. The goal is to create a small trusted host that manages windows, state, permissions, and execution, while feature plugins run behind explicit contracts and capability boundaries.

## Product Shape

```text
Desktop / CLI / HTTP API
        |
        | typed core services and events
        v
Rust Host Engine
Tokio, SQLite, config, permissions, scheduling
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
- [CLI Technology Decision](docs/architecture/CLI_TECH_DECISION.md)
- [Repository Layout](docs/architecture/REPOSITORY_LAYOUT.md)
- [Project And Automation Flow](docs/architecture/PROJECT_FLOW.md)
- [System Architecture](docs/architecture/SYSTEM.md)
- [Development Targets](docs/planning/DEVELOPMENT_TARGETS.md)
- [Roadmap](docs/planning/ROADMAP.md)
- [MVP Definition](docs/planning/MVP.md)
- [Security Model](docs/security/SECURITY_MODEL.md)
- [Development Workflow](docs/development/WORKFLOW.md)
- [MVP Runbook](docs/development/MVP_RUNBOOK.md)
- [Step-By-Step Development Flow](docs/development/STEP_BY_STEP_DEVELOPMENT_FLOW.md)
- [Packaging And Distribution](docs/development/PACKAGING.md)
- [Brand Guide](docs/branding/README.md)

## Repository Layout

```text
poly.glid_security_workspace/
├── apps/
│   ├── desktop/              # Dioxus desktop workbench
│   ├── cli/                  # terminal client
│   └── server/               # HTTP/WebSocket API
├── crates/
│   ├── core/                 # application use cases and persistence
│   ├── runtime/              # Wasmtime execution adapter
│   ├── plugin-api/           # plugin-facing Rust types
│   ├── config/               # configuration and registry
│   └── events/               # typed host events
├── contracts/
│   └── polyglid.wit          # canonical host/plugin contract
├── plugins/
│   └── recon-probe/          # first-party WASM plugin
├── site/                     # public website generator
├── sdk/                      # plugin SDK and examples
├── tools/                    # AI and workspace automation
├── scripts/ops/              # stable operations CLI
├── infrastructure/          # deployment and service definitions
├── tests/                    # workspace-level tests
└── docs/                     # architecture and operating guides
```

## Build Order

1. Define the WIT contract.
2. Build the Rust plugin runtime and CLI harness.
3. Build one harmless diagnostic plugin.
4. Add config and permission models.
5. Add the Tauri shell.
6. Add the workspace UI.
7. Add richer plugins only after the boundary is tested.

## Run The Current MVP

```bash
rustup target add wasm32-wasip1
scripts/run-mvp.sh
```

The MVP runs the CLI host, componentizes `recon_probe`, grants `dns-resolve` and
`report-write` for the run, and writes output under `reports/`.

## Download Releases

Published builds are available from the [latest GitHub release](https://github.com/anonto42/poly.glid_security_workspace/releases/latest). Each release contains Linux, Windows, Intel macOS, and Apple Silicon macOS archives, the Recon Probe WASM component, and `SHA256SUMS`.

Maintainers publish a release by updating the workspace version, committing it,
and pushing the matching tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions builds on each operating system, publishes the release, and then
redeploys the website with links to the latest artifacts.
