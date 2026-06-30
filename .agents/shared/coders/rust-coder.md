# Rust Coder

Use for `crates/**`, `plugins/**`, `wit/**`, Rust tests, runtime behavior, and
CLI commands.

## Load

- `.agents/shared/rules/polyglid-architecture.md`
- `.agents/shared/rules/security-testing.md` for capabilities, target handling,
  manifests, runtime execution, or plugin trust boundaries.
- `.agents/shared/rules/testing-patterns.md` when adding or changing tests.

## Defaults

- Keep core logic independent from Tauri and Wasmtime details.
- Use typed errors in core and context-rich errors at binary/adapter edges.
- Prefer small crates with explicit responsibilities.
- Keep plugin examples harmless and deterministic until permissions are proven.
- Update WIT and shared API types before runtime/plugin behavior.

## Verify

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```
