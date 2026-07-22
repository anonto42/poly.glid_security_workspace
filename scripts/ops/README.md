# PolyGlid Operations

`polyglid-ops.mjs` is the canonical entry point for repository automation. It
coordinates the root Cargo workspace, the independent Rust SDK workspace, and
the independent AI-engine workspace. Thin wrappers such as `npm` or `make`
should delegate here instead of carrying a second build plan.

```bash
npm run ops -- help
npm run ops -- doctor
npm run ops -- format
npm run ops -- check
npm run ops -- build --release
npm run ops -- test
npm run ops -- desktop
```

## Command groups

Compilation commands coordinate all three Cargo workspaces. Formatting is
deliberately limited to the maintained root and SDK workspaces: the isolated AI
engine is experimental and carries pre-existing formatting debt that is outside
the desktop-MVP automation gate.

| Command | Operation |
| --- | --- |
| `format [args]` | Format the maintained root and SDK workspaces |
| `check [args]` | Run locked checks, targeting the SDK at `wasm32-wasip1` |
| `build [args]` | Run locked builds, targeting the SDK at `wasm32-wasip1` |
| `test [args]` | Run root, SDK, and AI tests, then all-feature-check the SDK for WASM |
| `clean [args]` | Clean workspace build outputs |
| `graph` | Generate one DOT dependency graph for all workspaces |

`validate` checks `repinfo.json`, shell and Node operations scripts, the
operations regression tests, the workspace-layout validator, maintained-code
formatting, and locked Cargo checks in every workspace. `doctor` checks the
repository manifests and the command-line tools used by local automation and
delivery.

Product and routing commands stay focused:

| Command | Operation |
| --- | --- |
| `desktop [args]` | Run the primary Dioxus desktop client |
| `server [args]` | Run the optional backend server |
| `detect [base] [head]` | Classify changed paths as JSON |
| `site-build [args]` | Generate the public static website |
| `mvp-smoke` | Exercise the fixed real CLI-to-WASM regression path |
| `repo-sync` | Apply `repinfo.json` to GitHub metadata |

Commands marked with `[args]` forward every trailing argument as its own process
argument. For example, `build --release` passes `--release` to each Cargo
workspace, and `desktop -- --example` passes `--example` to the desktop
executable. `test` forwards the same Cargo and test-binary arguments to the
native root, SDK, and AI test suites, then runs a fixed all-features SDK check
for `wasm32-wasip1`. `doctor`, `graph`, `mvp-smoke`, and `repo-sync` accept no
arguments, while `detect` accepts at most two.

## Safe command-plan inspection

Set `POLYGLID_OPS_DRY_RUN=1` to print each process invocation as a JSON array
without executing it:

```bash
POLYGLID_OPS_DRY_RUN=1 npm run ops -- build --release
```

The JSON-array format preserves exact argument boundaries and makes the command
plans regression-testable.

## Ownership and side effects

- `polyglid-ops.mjs` dispatches commands and propagates the first failure.
- `detect-changes.sh` classifies changed paths without performing work.
- `sync-repo.mjs` applies `repinfo.json` through the GitHub CLI.
- `deploy-site.yml` owns GitHub Pages deployment.
- `repo-sync.yml` owns repository metadata synchronization.
- `ci.yml` owns validation and build/test routing.

Repository synchronization is intentionally available only through the
explicit `repo-sync` command. It requires `GITHUB_REPOSITORY` and an
authenticated `gh` CLI. In Actions, authentication comes from the `GH_PAT`
repository secret. No validation, build, or test command changes repository
metadata.
