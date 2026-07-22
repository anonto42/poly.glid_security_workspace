# Automation Compatibility Area

The canonical repository task surface is
`scripts/ops/polyglid-ops.mjs`. The root `Makefile`, `package.json`, and the
deprecated `tools/automation/Makefile` only delegate to it.

The active files in this directory are:

- `scripts/validate-workspace.sh`, which validates the real repository layout
  and all three Cargo metadata roots;
- `scripts/generate-graph.sh`, which derives one DOT dependency graph from
  those Cargo manifests and lockfiles.

The experimental AI Make-template generator reads `templates/` and may write
generated project fragments under `includes/projects/`. The other top-level
files under `includes/` are dormant legacy Make modules with no active caller.
None of these files are loaded by the root Makefile, and they must not become a
second source of repository build behavior.

## Orchestrator decision

Moon and Just are intentionally not dependencies today. The substantive build
surface is still Rust, Cargo already owns package dependency resolution,
`rust-cache` owns compiled-artifact caching in Actions, and the explicit CI jobs
provide the visible delivery graph used to diagnose releases. Adding another
task graph now would duplicate those responsibilities. Revisit an affected-task
orchestrator when a second language has real buildable projects, or when measured
CI time shows that path routing plus Rust caching is no longer sufficient.
