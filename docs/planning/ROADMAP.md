# Roadmap

## Phase 0: Planning Foundation

Status: current.

- define system architecture
- define codebase layout
- define security model
- define MVP boundary
- decide first plugin and WIT contract

Exit condition: the repo explains what to build and where each piece belongs.

## Phase 1: Runtime Proof

Build the smallest end-to-end runtime.

- create Rust workspace
- create `contracts/polyglid.wit`
- create `polyglid-plugin-api`
- create `polyglid-runtime`
- create `polyglid-cli`
- create `plugins/recon_probe`
- run one plugin through the CLI harness

Exit condition: `polyglid-cli plugin run` executes a WASM component and prints a structured report.

## Phase 2: Permission Model

Add guardrails before adding powerful features.

- define capability enum
- define plugin manifest format
- require permission grants before host capability use
- add denied-by-default config
- add audit log events

Exit condition: a plugin cannot use a host capability unless the host grants it.

## Phase 3: Desktop Shell

Add the Tauri application shell.

- create `apps/desktop`
- wire Tauri commands to `polyglid-core`
- add settings persistence
- add event broadcast from host to frontend
- display plugin run output in the UI

Exit condition: GUI can start the demo plugin and render its report.

## Phase 4: Workspace UI

Build the operator workspace.

- left utility navigation
- center workspace panes
- bottom status footer
- plugin task panel
- structured report viewer
- basic settings page

Exit condition: the UI feels like the first usable PolyGlid workspace.

## Phase 5: Real Plugin Families

Only after runtime and permissions are stable, add real diagnostic families.

- reconnaissance helpers
- banner analysis
- local defensive audit checks
- report generation

Exit condition: plugins remain sandboxed, permissioned, testable, and replaceable.

## Phase 6: Distribution

Package the app.

- desktop bundles
- signed releases
- plugin packaging format
- plugin registry/index
- update strategy
- multi-platform CLI release binaries
- platform-specific app data layout

Exit condition: another machine can install PolyGlid and run approved plugins.
