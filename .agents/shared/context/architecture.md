# Current Architecture

## Request Flow

```text
Dioxus views and AppState
        |
        v
DesktopControllers
        |
        v
ClientGateway trait
        |
        v
LocalClient
        |
        +--> core services --> SQLite stores
        +--> filesystem workspace/project operations
        +--> permission and signature services
        +--> Wasmtime runtime --> contracts/polyglid.wit
```

The synchronous `ClientGateway` keeps UI code independent from domain
implementation. Dioxus moves blocking filesystem and database calls to
`tokio::task::spawn_blocking` and maps every result back into visible state.

## Desktop State

`AppState` contains:

- shell visibility, width, and resize state;
- workspace and project catalog data;
- selected and active identifiers;
- loading, operation, notice, and error state.

The root app bootstraps through `ApplicationController`. Project mutations go
through `ProjectsController`, then trigger a catalog refresh.

## Project Persistence

Workspace and project metadata is stored in SQLite. Discovery considers only
direct, non-symlink child folders and identifies Rust, Node, Python, or general
projects from marker files.

Removing a project archives and excludes it from the catalog. Permanent
deletion additionally resolves the workspace and project paths and requires the
project to be a non-symlink direct child of the workspace root.

## Plugin Security Boundary

Plugins use the component contract in `contracts/polyglid.wit`. The runtime:

- verifies manifests and structured capability requests;
- runs Wasmtime with fuel, epoch interruption, and memory limits;
- creates a restricted WASI context;
- checks effective grants before host operations;
- validates report paths against absolute paths and traversal;
- supports Ed25519 signature verification and publisher trust records.
