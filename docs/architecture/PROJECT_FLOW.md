# PolyGlid Project Flow

This document records the canonical repository ownership model and shows what happens after each development or runtime action.

## Canonical Repository Layout

```text
polyglid/
├── apps/                 # Desktop, CLI, and server clients
├── crates/               # Reusable Rust engine libraries
├── contracts/            # Language-neutral WIT contracts
├── plugins/              # First-party sandboxed WASM plugins
├── site/                 # Public static website generator
├── sdk/                  # Plugin templates and language SDKs
├── tools/                # Internal AI and workspace automation
├── scripts/ops/          # Stable operations CLI
├── infrastructure/       # Deployment and external services
├── tests/                # Workspace-level tests
├── extensions/           # IDE and browser integrations
├── releases/             # Release and packaging definitions
└── docs/                 # Architecture and operating knowledge
```

The retired `slices/` tree must not be recreated. Vertical slices remain a development method, not a source-directory name. Every component has exactly one canonical location.

## Runtime Flow

```mermaid
flowchart TD
    User --> Client{Client surface}
    Client --> Desktop[apps/desktop]
    Client --> CLI[apps/cli]
    Client --> Server[apps/server]
    Desktop --> Core[crates/core]
    CLI --> Core
    Server --> Core
    Core --> Store[(SQLite workspace)]
    Core --> Security[Permission and trust checks]
    Security --> Runtime[crates/runtime]
    Runtime --> Contract[contracts/polyglid.wit]
    Contract --> Plugin[plugins/recon-probe]
    Plugin --> Report[Structured report]
    Report --> Core
    Core --> Store
    Core --> Client
```

## Feature Development Flow

```mermaid
flowchart LR
    Plan --> Contract[Define domain or WIT contract]
    Contract --> Core[Implement core behavior]
    Core --> Tests[Test policy and failure paths]
    Tests --> Adapter[Implement runtime or storage adapter]
    Adapter --> Client[Connect desktop, CLI, or server]
    Client --> E2E[Verify end to end]
    E2E --> Docs[Update documentation]
    Docs --> CI[Run CI quality gates]
```

The required dependency direction is `contract → core → adapter → client`. Clients must not bypass core services to access SQLite or Wasmtime directly.

## GitHub Automation Flow

```mermaid
flowchart TD
    Push[Push or pull request] --> Detect[Detect changed folders]

    Detect -->|apps, crates, or workspace| Format[Rust format check]
    Format --> Clippy[Rust Clippy]
    Clippy --> Build[Rust workspace build]
    Build --> Tests[Rust workspace tests]

    Detect -->|contracts or plugins| WasmBuild[Build Recon Probe WASM]
    WasmBuild --> WasmTest[Test Recon Probe]

    Detect -->|configuration| Config[Test polyglid-config]
    Detect -->|sdk| SDK[Validate plugin SDK]
    Detect -->|tools/ai| AI[Build AI engine]
    Detect -->|docs| Docs[Validate documentation]
    Detect -->|workflows or scripts| Ops[Validate operations scripts]
    Detect -->|infrastructure| Infra[Validate infrastructure layout]

    Detect -->|site| SiteBuild[Generate static website]
    SiteBuild -->|push to main| Pages[Deploy GitHub Pages]

    Detect -->|repinfo.json on main| Metadata[Sync repository metadata]

    Tests --> Result[CI result]
    WasmTest --> Result
    Config --> Result
    SDK --> Result
    AI --> Result
    Docs --> Result
    Ops --> Result
    Infra --> Result
    SiteBuild --> Result
```

- Each box above is a separate Actions job, so GitHub renders the same dependency graph in the workflow run overview.
- `ci.yml` detects changes and connects the validation, build, test, deployment, and final-result jobs.
- `deploy-site.yml` is a reusable workflow called by CI after a successful site build on `main`.
- `repo-sync.yml` is a reusable workflow called by CI when `repinfo.json` changes on `main`.
- `scripts/ops/polyglid-ops.mjs` is the shared local and CI entry point.

## Release Flow

```mermaid
flowchart LR
    Tag[Push version tag] --> Matrix[Native build matrix]
    Matrix --> Linux[Linux x86_64]
    Matrix --> Windows[Windows x86_64]
    Matrix --> MacIntel[macOS Intel]
    Matrix --> MacArm[macOS Apple Silicon]
    Tag --> Plugin[Recon Probe WASM component]
    Linux --> Release[GitHub Release + SHA256SUMS]
    Windows --> Release
    MacIntel --> Release
    MacArm --> Release
    Plugin --> Release
    Release --> Website[Refresh GitHub Pages downloads]
```

## Generated State

Runtime databases, reports, build output, caches, and local analytics are not source code. The root `.gitignore` excludes `polyglid.db`, `reports/`, `target/`, and local workspace data.
