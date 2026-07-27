#!/usr/bin/env bash
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

if [ "${SKIP_POLYGLID_PRECOMMIT:-0}" = "1" ]; then
  echo "PolyGlid pre-commit checks skipped by explicit request"
  exit 0
fi

echo "PolyGlid pre-commit: formatting"
cargo fmt --all --check

echo "PolyGlid pre-commit: locked workspace metadata"
cargo metadata --locked --no-deps --format-version 1 >/dev/null

workspace_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
plugin_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' plugins/recon-probe/polyglid.toml | head -n 1)
test "$workspace_version" = "$plugin_version" || {
  echo "Workspace version $workspace_version does not match Recon manifest version $plugin_version" >&2
  exit 1
}

echo "PolyGlid pre-commit: operations scripts"
node --check scripts/ops/polyglid-ops.mjs
node --check scripts/ops/sync-repo.mjs
node scripts/ops/test-polyglid-ops.mjs
bash -n scripts/ops/detect-changes.sh
bash -n scripts/ops/test-detect-changes.sh
bash -n scripts/ops/mvp-smoke.sh
bash scripts/ops/test-detect-changes.sh

echo "PolyGlid pre-commit: GitHub Actions"
if command -v actionlint >/dev/null 2>&1; then
  actionlint
elif command -v go >/dev/null 2>&1; then
  go run github.com/rhysd/actionlint/cmd/actionlint@v1.7.12
else
  echo "actionlint skipped locally because neither actionlint nor Go is installed"
  echo "GitHub CI will still enforce workflow validation"
fi

echo "PolyGlid pre-commit checks passed"
