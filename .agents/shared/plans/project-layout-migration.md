# Product-Oriented Project Layout

Status: implemented.
Updated: 2026-07-14.

## Decision direction

Organize `slices/` by product, service, library, or plugin. Do not group source
under `slices/rust`, `slices/node`, `slices/go`, or `slices/python`.
Languages are implementation details declared inside each project.

## Proposed target

```text
slices/
├── polyglid-desktop/          # Rust + Dioxus; current WPM UI
├── polyglid-cli/              # Rust terminal client
├── polyglid-server/           # Rust service/API
├── polyglid-core/             # domain and orchestration library
├── polyglid-runtime/          # Wasmtime runtime
├── polyglid-config/           # configuration library
├── polyglid-events/           # event contracts
├── polyglid-plugin-api/       # plugin/WIT-facing types
├── polyglid-contracts/        # canonical WIT contract
└── recon-probe/               # first-party WASM plugin
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

## Completed migration

1. Preserved the Dioxus baseline in commit `063723b`.
2. Renamed `slices/wpm` to `slices/polyglid-desktop`.
3. Moved Rust crates, WIT, and the plugin to direct product paths.
4. Updated Cargo members, path dependencies, scripts, docs, and agent references.
5. Removed the retired React and Tauri clients after selecting Dioxus as canonical.
6. Removed empty language grouping placeholders.

Every move must pass formatting, workspace metadata, compile/check, tests, and
focused launch verification. Path migration must not be mixed with behavior changes.
