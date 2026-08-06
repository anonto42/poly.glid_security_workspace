# Contributing to PolyGlid

Thank you for helping improve PolyGlid, a local-first, cross-platform desktop
application for managing local project workspaces.

## Ways to Contribute

- Report bugs with clear reproduction steps.
- Suggest focused product or developer-experience improvements.
- Improve documentation, installation instructions, and examples.
- Fix bugs or add tests around existing behavior.
- Help improve desktop, website, packaging, and release workflows.

## Local Development

Install [Nix](https://nixos.org/download/) with flakes enabled, then enter the
reproducible development environment:

```bash
nix develop
```

Run the desktop application and common checks:

```bash
cargo run -p polyglid-desktop
cargo check --workspace
cargo test --workspace
```

Moon can also run the repository tasks:

```bash
moon run :format
moon run :check
moon run :test
moon run :build
```

## Pull Request Guidelines

- Open an issue first for large changes or significant product-direction
  decisions.
- Keep each pull request focused on one logical change.
- Do not commit secrets, tokens, private project data, generated runtime data,
  or local configuration files.
- Follow the existing architecture: runnable products belong in `apps/` and
  reusable logic belongs in `crates/`.
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
create a product release.

## Architecture and Safety Notes

- The active product phase covers project and workspace management. Do not
  restore future product areas without a requested phase.
- Preserve explicit confirmation and direct-child validation for permanent
  project-folder deletion.
- Preserve least-privilege capability checks, plugin signature verification,
  Wasmtime limits, and safe output-path validation.
- Propagate useful failure messages to the desktop UI.
- Rust 2021 is used throughout the workspace and unsafe code is forbidden.

## Security

Please do not open public issues for vulnerabilities. Follow
[SECURITY.md](SECURITY.md) instead.
