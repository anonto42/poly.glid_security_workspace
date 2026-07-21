# PolyGlid Desktop

PolyGlid Desktop is the primary PolyGlid product client. It is a local-first
native Rust application built with Dioxus Desktop and connected to PolyGlid core
services, SQLite, the plugin registry, the execution manager, and Wasmtime.

Read the canonical [Client Architecture](../../docs/architecture/CLIENT_ARCHITECTURE.md)
before changing client boundaries. The detailed component map and implementation
status are in [Desktop UI](../../docs/architecture/DESKTOP_UI.md).

## Current Product Journey

```text
Projects -> New scan -> Permission review -> Executions -> Reports
                           |
                           +-> deny without starting
```

The activity rail exposes only real product areas: Projects, New scan,
Executions, Reports, Plugins, and Settings. Earlier Work Tracks, Automation, AI
Agents, source preview, and terminal implementations were removed from the
desktop source instead of being presented as unfinished product functions.

## What Works Today

- Native Dioxus workbench with product navigation, contextual sidebars, open
  views, resizable panes, command palette, settings, activity, and shortcuts.
- UI-safe DTOs, typed `ClientError`, a cloneable `ClientGateway`, concrete
  `LocalClient`, bootstrap snapshot, and typed execution subscription.
- SQLite workspace/project catalog with project discovery, create, rename,
  remove-only, and confirmed filesystem deletion.
- Persistent, validated local scan targets.
- Real WASM component validation, manifest inspection, installation,
  enable/disable, and uninstall operations.
- Separate allow-once review for plugin execution. Nothing is preselected; the
  local client re-inspects the executable and rejects missing or unexpected
  capability kinds before starting it.
- Asynchronous execution submission returning a `JobId`, local state/history,
  event-driven refresh, and cancellation requests.
- Persisted report history with real findings and JSON, Markdown, and SARIF
  file export.
- Persisted sidebar visibility, panel visibility, and pane sizes.
- Clear loading, empty, error, disabled, busy, execution-status, and
  permission-review presentation styles.

## Remaining Product Work

- Permission review currently grants capability kinds for one run. Stable
  approval IDs, exact resource-scope enforcement, expiration, revocation, and
  session/workspace decisions remain.
- Several feature wiring components call `LocalClient` directly. Feature
  controllers should become the only gateway callers and store writers.
- Current state is split into four useful stores, but Scanner, Executions,
  Reports, and Settings should separate further as their behavior grows.
- Project selection is not yet a required scan/report context.
- Execution progress is state-oriented; richer typed stages and reconnect
  behavior remain.
- The execution fuel draft is session-only, and host-health settings need a
  complete persisted model.
- Plugin publisher/signature trust and normal platform installers remain
  release-hardening work.
- The v0.10.0 release contains `recon-probe.component.wasm` without the adjacent
  `.component.sig` required by default Balanced policy. The safe fix is a new
  release signed by a long-lived offline Ed25519 identity with its seed kept in
  CI secrets and its public fingerprint pinned for trust bootstrap. Do not
  commit a signing key or silently fall back to Development policy.

## Current Module Map

```text
src/
├── main.rs                    Dioxus window and application launch
├── client/
│   ├── models.rs              UI-safe client DTOs and stable IDs
│   ├── error.rs               typed client failures
│   ├── gateway.rs             ClientGateway and execution subscription
│   └── local.rs               local core/storage/runtime adapter
└── ui/
    ├── app.rs                 contexts, bootstrap, event watch, shell composition
    ├── state.rs               Shell, Catalog, Plugin, and Run stores
    ├── models.rs              navigation, overlays, permission review
    ├── commands.rs            shortcuts and shell actions
    ├── top_bar.rs             brand, workspace picker, command center, status
    ├── shell.rs               product rail and status bar
    ├── sidebar.rs             context navigation and target/plugin entry
    ├── editor.rs              product-view routing and current gateway wiring
    ├── bottom_panel.rs        persisted findings and local activity
    ├── overlays.rs            settings, commands, install/permission review
    ├── components.rs          reusable visual primitives
    └── features/
        ├── projects.rs        workspace project management
        ├── scanner.rs         scan draft and permission summary
        ├── executions.rs      job history, state, cancellation, report routing
        ├── reports.rs         persisted detail and export
        └── plugins.rs         plugin registry management

assets/
├── theme.css                  design and semantic status tokens
├── main.css                   forms, dialogs, permissions, runs, reports
├── shell.css                  persistent workbench and responsive layout
└── projects.css               project feature styles
```

The current client boundary is deliberately local and in-process:

```text
Dioxus view wiring -> LocalClient / ClientGateway -> core services
                  <- UI-safe result and execution event
```

The next separation is:

```text
view -> feature controller -> ClientGateway -> application/core service
view <- feature store      <- typed result/event
```

## Run from Source

```bash
cargo run -p polyglid-desktop
```

Optional local paths:

```bash
POLYGLID_DATA_DIR=/path/to/data \
POLYGLID_WORKSPACE_ROOT=/path/to/projects \
cargo run -p polyglid-desktop
```

Without overrides, application data is stored under `~/.polyglid`, and the
default workspace root is `~/polyglid-projects`.

## Run a Release Archive

From the extracted Linux archive:

```bash
chmod +x polyglid-desktop
./polyglid-desktop
```

Dioxus Desktop uses native Linux WebView/GTK libraries. If the loader reports
`libxdo.so.3` on Debian or Ubuntu:

```bash
sudo apt update
sudo apt install libxdo3
./polyglid-desktop
```

On Arch Linux or CachyOS, `xdotool` provides `libxdo`:

```bash
sudo pacman -S --needed xdotool
./polyglid-desktop
```

To identify another unresolved native library:

```bash
ldd ./polyglid-desktop | grep "not found"
```

Native installers that declare platform dependencies remain a distribution
milestone; see [Packaging](../../docs/development/PACKAGING.md).

## Keyboard Controls

| Shortcut | Action |
| --- | --- |
| `Ctrl+P` or `F1` | Open command palette |
| `Ctrl+1` | Open Projects |
| `Ctrl+2` | Open New scan |
| `Ctrl+3` | Open Executions |
| `Ctrl+4` | Open Reports |
| `Ctrl+5` | Open Plugins |
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+J` | Toggle Findings/Activity panel |
| `Ctrl+W` | Close the active product view |
| `Escape` | Close the active overlay |

## Verify

```bash
cargo fmt --all -- --check
cargo check -p polyglid-desktop
cargo test -p polyglid-desktop
```

Manual product checks:

- bootstrap renders loading, empty, ready, and failure states clearly;
- project create, rename, remove-only, and delete-files actions have accurate
  outcomes;
- saved targets survive restart and invalid targets show actionable errors;
- component validation happens before install review;
- installation and enablement never bypass per-run permission review;
- permission review starts empty, denial starts nothing, and approval returns a
  visible execution record;
- running work remains cancellable and terminal states are not shown as active;
- a completed execution creates a persisted report that survives restart;
- JSON, Markdown, and SARIF exports contain real report values;
- keyboard focus and shell layout remain usable at the 900 x 620 minimum size.
