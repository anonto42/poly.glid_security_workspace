# Contributing to PolyGlid

Thank you for helping improve PolyGlid, a local-first, cross-platform desktop
application for organizing local project workspaces with explicit permissions.

## Ways to Contribute

- Report bugs with clear, minimal reproduction steps.
- Suggest focused improvements to the product or developer experience.
- Improve documentation, installation instructions, and examples.
- Fix bugs or add tests around existing behavior.
- Help improve the desktop shell, website, packaging, or release workflows.

## Local Development

PolyGlid is a Rust workspace. Install [Nix](https://nixos.org/download/) with
flakes enabled, then enter the reproducible development environment:

```bash
nix develop
```

The shell provides the pinned Rust toolchain (1.96.0), the WebAssembly target,
Moon, the Dioxus CLI, and the native Linux libraries required by the desktop
build.

Common commands from the repository root:

```bash
cargo run -p polyglid-desktop
cargo check --workspace
cargo test --workspace
```

Moon orchestrates the same checks without replacing Cargo:

```bash
moon run :format
moon run :check
moon run :test
moon run :build
```

The exact verification used by CI is available as:

```bash
scripts/ci/verify-rust.sh
```

## Pull Request Guidelines

- Open an issue first for large changes or product-direction decisions.
- Keep each pull request focused on a single logical change.
- Do not commit secrets, tokens, private project data, generated runtime data,
  or local configuration files.
- Follow the existing architecture: runnable products live in `apps/` and
  reusable logic lives in `crates/`.
- Keep the desktop application on Dioxus unless an explicit architecture
  decision changes that direction.
- Update documentation when setup, behavior, or public interfaces change.
- Run the narrowest relevant checks before submitting.

## Commit and Pull Request Titles

Pull request titles declare release intent using Conventional Commits:

```text
type(scope): short imperative subject
```

Examples:

- `fix(core): reject an unsafe output path`
- `docs(setup): clarify Nix development setup`
- `feat(desktop): add project search`

Use `!` after the type or scope, or a `BREAKING CHANGE:` footer, for breaking
changes. Documentation-only changes should use a `docs:` title and do not
trigger a product release.

## Architecture and Safety Notes

- The active product phase covers local project and workspace management plus
  plugin execution. Do not restore future product areas without a requested
  phase.
- Preserve explicit confirmation and direct-child validation for permanent
  project-folder deletion.
- Preserve least-privilege capability checks, plugin signature verification,
  Wasmtime runtime limits, and safe output-path validation.
- Propagate useful failure messages to the desktop UI.
- Rust 2021 is used throughout the workspace and unsafe code is forbidden.

## Security

Please do not open public issues for vulnerabilities. Follow
[SECURITY.md](SECURITY.md) instead.
