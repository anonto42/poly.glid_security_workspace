# CI and Release Process

## Continuous Integration

Every branch push and pull request into `main` runs `moon ci`. Moon selects
affected project tasks from Git history, includes upstream dependencies and all
transitive downstream consumers, and restores cacheable task results. The
checkout must retain full history so change detection can compare revisions.

Rust projects inherit package-scoped format, locked check, locked test, and
locked build tasks. Website changes also build the optimized static bundle.
Workflow, Moon, or script changes run the automation validator, including shell
syntax checks and Actionlint. Global Cargo or Moon configuration changes
intentionally invalidate all relevant projects.

Adding `apps/*/Cargo.toml` or `crates/*/Cargo.toml` requires a `feat:` title.
Workflow YAML contains orchestration only. Reusable commands belong under
`scripts/ci/`, `scripts/site/`, and `scripts/release/`. Cargo's `target`
directory is not a Moon cache output; only the final static website bundle is
declared as an output.

## Version Detection

- A type with `!` or a `BREAKING CHANGE:` footer produces a major bump.
- `feat:` produces a minor bump.
- `fix:` produces a patch bump.
- Documentation, tests, CI, build, style, and chore-only commits do not produce
  a product release.

Release detection starts after commit
`df1fc661643bb9c2ea32c40a7ae67dcd7e4ab6c0`; older reference-line commits are
not release inputs.

## Delivery

After CI succeeds for a push to `main`, Release Please updates a release pull
request. Merging that pull request updates `Cargo.toml`, `Cargo.lock`, and
`CHANGELOG.md`, creates the version tag, and creates a draft GitHub release.

Set the `RELEASE_PLEASE_TOKEN` repository secret to a GitHub App token or
fine-grained personal access token so release pull requests trigger CI. The
workflow falls back to `GITHUB_TOKEN`, whose generated pull requests do not
start other workflows.

Use squash merges and configure the repository to use the pull request title
as the squash commit title. This preserves the validated Conventional Commit
intent on `main`, where Release Please reads it.

Release jobs build and archive:

- Linux x86-64;
- Windows x86-64;
- macOS Apple Silicon.

The release is published only after all three archives and `SHA256SUMS` upload
successfully.

Reference branches never publish active PolyGlid releases.
