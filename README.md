# PolyGlid

PolyGlid is being built one working product phase at a time. The current phase
contains only the cross-platform Dioxus desktop shell and local project
workspace.

## Current functionality

- Open the local PolyGlid workspace.
- Discover persisted projects.
- Create and select a project.
- Rename a project.
- Remove a project from the catalog.
- Permanently delete a project folder after explicit confirmation.
- Switch between configured local workspaces.

## Repository

```text
apps/desktop/       Dioxus desktop application
apps/website/       Dioxus Web product website
crates/client/      UI-independent controllers and local adapter
crates/config/      persisted configuration models
crates/core/        local project and application services
crates/events/      shared event types required by the client
crates/plugin-api/  shared domain types required by the core
crates/runtime/     runtime dependency required by the local client
scripts/            CI, verification, packaging, and release automation
```

Future product areas are intentionally absent. They will be introduced only
when their development phase begins.

## Development

```bash
cargo run -p polyglid-desktop
cargo check --workspace
cargo test --workspace
```

Moon can orchestrate the workspace checks without replacing Cargo:

```bash
moon run :format
moon run :check
moon run :test
moon run :build
moon run website:bundle
```

The same complete check used by CI is available as:

```bash
scripts/ci/verify-rust.sh
```

## CI and releases

Every branch push and every pull request into `main` uses Moon to run formatting,
checks, tests, builds, and automation validation for affected projects. Changes
to a shared crate also validate all of its downstream consumers. Pull request
titles declare release intent using Conventional Commits:

| Pull request title | Version result |
| --- | --- |
| `feat!: change a public contract` | Major |
| `feat(desktop): add project search` | Minor |
| `fix(core): reject an unsafe path` | Patch |
| `docs: explain workspace setup` | No product release |

A new app or crate is treated as a feature and requires a `feat:` title.
Breaking changes can use `!` after the type or a `BREAKING CHANGE:` commit
footer.

After changes land on `main` and CI succeeds, Release Please creates or updates
a release pull request containing the workspace version, lockfile, and
changelog updates. Merging that pull request creates a version tag and draft
GitHub release. GitHub Actions then builds Linux x86-64, Windows x86-64, and
macOS Apple Silicon archives, generates SHA-256 checksums, uploads the assets,
and publishes the release.

The website is also built and deployed to GitHub Pages only after CI succeeds
for `main`. Before its first deployment, set the repository's Pages source to
**GitHub Actions** in repository settings.

For release pull requests to trigger their own CI run, configure a repository
secret named `RELEASE_PLEASE_TOKEN` with a GitHub App token or fine-grained
personal access token. Without it, the workflow falls back to `GITHUB_TOKEN`.
Use squash merges and configure GitHub to use the pull request title as the
squash commit title so the validated intent is preserved on `main`.
