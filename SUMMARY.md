# PolyGlid Workspace Summary

This file describes the implementation that is tracked today. It does not
treat empty directories, design documents, or experimental tools as delivered
product features.

## Product Direction and Status

PolyGlid is a local-first security workspace with a small trusted Rust host and
sandboxed WebAssembly components. The native Dioxus desktop application is the
primary product client. The CLI is a development and regression harness, and
the server is a supporting API surface rather than the desktop MVP.

The principal desktop slices are connected: projects, plugin inspection and
lifecycle, per-run permission review, asynchronous executions, persisted
reports, and report export. Connected foundations do not mean the product is
ready to ship. Scoped durable permissions, signed first-party components,
normal platform packages, accessibility acceptance, and packaged end-to-end
tests remain open.

- [Current roadmap](docs/planning/ROADMAP.md)
- [Desktop MVP definition and checklist](docs/planning/MVP.md)
- [Desktop implementation status](apps/desktop/README.md)
- [Client architecture](docs/architecture/CLIENT_ARCHITECTURE.md)

## Runtime Shape

```text
Dioxus Desktop (primary client)
CLI harness / HTTP server (supporting clients)
                 |
                 v
Rust core services, persistence, policy, and typed events
                 |
                 v
Wasmtime component runtime -> WIT contract -> sandboxed WASM plugin
```

## Tracked Implementation Surfaces

| Path | Current responsibility |
| --- | --- |
| `apps/desktop/` | Primary Dioxus workbench and its local `ClientGateway` adapter |
| `apps/cli/` | Runtime/component developer harness; not the desktop MVP |
| `apps/server/` | HTTP and collaboration API foundation |
| `crates/core/` | Application services, execution control, security policy, and SQLite-backed stores |
| `crates/runtime/` | Wasmtime Component Model adapter and host capability linking |
| `crates/config/` | Configuration and plugin registry |
| `crates/events/` | Typed host and client events |
| `crates/plugin-api/` | Plugin-facing Rust types and capability declarations |
| `contracts/polyglid.wit` | Canonical host/plugin interface |
| `plugins/recon-probe/` | First-party diagnostic WASM component source and manifest |
| `site/` | Rust static-site generator and generated public download page |
| `sdk/` | Independent Rust plugin-template/examples workspace; top-level Go, Node, Python, and Rust directories are placeholders |
| `scripts/ops/` | Canonical local/CI task dispatcher, change detection, smoke test, and repository sync |
| `.github/workflows/` | Selective CI, Pages deployment, release publication, metadata sync, and cache maintenance |
| `docs/` | Architecture, security, planning, packaging, and development guidance |

`tools/ai/rust/` is an independent experimental workspace that CI can build. It
is not connected to the current desktop product navigation or MVP journey, and
the root automation does not advertise its design documents as delivered AI
features.

There are no tracked browser or IDE extension implementations. The top-level
Go, Node, Python, and Rust SDK directories and `tests/security/` are `.gitkeep`
placeholders; active Rust packages live under `sdk/plugin-template/` and
`sdk/examples/`.
The only tracked infrastructure implementation is the legacy
`infrastructure/wpm/init.sql`; there is no tracked Docker Compose, Kubernetes,
or Terraform deployment stack to operate today.

## Cargo Workspaces

There are three intentionally separate Cargo build roots:

| Build root | Scope |
| --- | --- |
| `Cargo.toml` | Product applications, shared crates, Recon Probe, and site |
| `sdk/Cargo.toml` | Plugin template and Rust SDK examples |
| `tools/ai/rust/Cargo.toml` | Isolated experimental tooling |

The operations CLI coordinates these roots where a repository-wide command
needs them. It does not pretend that `cargo --workspace` at the repository root
includes every Cargo project.

## Canonical Commands

Repository task behavior lives in `scripts/ops/polyglid-ops.mjs`. Use it
directly, through `npm run`, or through the small root Make compatibility layer.
The wrappers add names, not a second implementation of the tasks.

| Make command | npm command | Purpose |
| --- | --- | --- |
| `make help` | `npm run help` | Show canonical operations commands |
| `make init` | `npm run init` | Compatibility alias for the read-only prerequisite doctor |
| `make doctor` | `npm run doctor` | Verify workspace files and required development/delivery tools |
| `make dev` | `npm run dev` | Compatibility alias that starts the desktop client |
| `make desktop` | `npm run desktop` | Run the Dioxus desktop client from source |
| `make server` | `npm run server` | Run the supporting HTTP server from source |
| `make format` | `npm run format` | Format the maintained root and SDK workspaces |
| `make check` | `npm run check` | Type-check coordinated Rust workspaces |
| `make validate` | `npm run validate` | Run repository metadata, script, formatting, and workspace validation |
| `make build` | `npm run build` | Build coordinated workspaces |
| `make test` | `npm run test` | Test coordinated workspaces |
| `make clean` | `npm run clean` | Remove Cargo artifacts from the coordinated workspaces |
| `make detect BASE=main HEAD=HEAD` | `npm run detect -- main HEAD` | Classify changes between two Git revisions |
| `make graph` | `npm run graph` | Print one Cargo-metadata DOT graph for all three workspaces |
| `make site` | `npm run site` | Generate the static website |
| `make mvp` | `npm run mvp` | Run the real CLI-to-WASM runtime smoke test |
| `make repo-sync` | `npm run repo-sync` | Apply `repinfo.json` with an authenticated GitHub CLI |

Extra arguments can be forwarded through Make with `ARGS`, for example:

```bash
make build ARGS="--release"
```

For the raw dispatcher or npm, append arguments normally:

```bash
node scripts/ops/polyglid-ops.mjs detect main HEAD
npm run ops -- validate
```

The compatibility Makefile under `tools/automation/` is deprecated and only
delegates these commands back to the repository root. The old included Make
modules are not the source of truth for current automation.

## Delivery Reality

GitHub Actions owns the workflow graph and platform concerns that do not belong
in a local task runner: job permissions, selective routing, artifacts, native
release matrices, GitHub Pages, repository metadata, delivery result gates, and
cache cleanup. See [CI and Delivery](docs/development/CI_DELIVERY.md) for the
exact push, pull-request, preview, and tag-release paths.
