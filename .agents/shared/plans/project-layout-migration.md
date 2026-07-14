# Product-Oriented Project Layout

Status: proposed for discussion.
Updated: 2026-07-14.

## Decision direction

Organize `projects/` by product, service, library, or plugin. Do not group source
under `projects/rust`, `projects/node`, `projects/go`, or `projects/python`.
Languages are implementation details declared inside each project.

## Proposed target

```text
projects/
├── polyglid-desktop/          # Rust + Dioxus; current WPM UI
├── polyglid-cli/              # Rust terminal client
├── polyglid-server/           # Rust service/API
├── polyglid-core/             # domain and orchestration library
├── polyglid-runtime/          # Wasmtime runtime
├── polyglid-config/           # configuration library
├── polyglid-events/           # event contracts
├── polyglid-plugin-api/       # plugin/WIT-facing types
├── recon-probe/               # first-party WASM plugin
├── polyglid-web-legacy/       # current React web UI until retirement
└── polyglid-desktop-legacy/   # current Tauri/React UI until parity
```

Empty language placeholders should be removed. A mixed-language project may keep
language-specific internal folders only when its own build requires them.

## Benefits

- A developer finds a capability by product name, without knowing its language.
- Each project owns its source, tests, assets, manifest, and documentation.
- Replacing a language does not change the repository's top-level information model.
- Automation and agents can discover project boundaries from manifests consistently.
- The Dioxus desktop becomes the clear product entry point instead of a prototype
  hidden behind the name `wpm`.

## Migration safety

1. Commit or preserve the current dirty implementation before moving paths.
2. Rename `projects/wpm` to `projects/polyglid-desktop` first.
3. Move one Rust crate or plugin per verified change using history-preserving moves.
4. Update Cargo members, path dependencies, scripts, docs, and `.agents` references.
5. Keep legacy React/Tauri projects until Dioxus engine parity is proven.
6. Remove language grouping folders only when empty.

Every move must pass formatting, workspace metadata, compile/check, tests, and
focused launch verification. Path migration must not be mixed with behavior changes.
