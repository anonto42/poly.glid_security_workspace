#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../../.." && pwd)
quiet=false

case "${1:-}" in
  "") ;;
  --quiet) quiet=true ;;
  *)
    echo "Usage: $0 [--quiet]" >&2
    exit 2
    ;;
esac

info() {
  if [ "$quiet" = false ]; then
    printf '%s\n' "$1"
  fi
}

pass() {
  if [ "$quiet" = false ]; then
    printf '  ✅ %s\n' "$1"
  fi
}

fail() {
  printf '  ❌ %s\n' "$1" >&2
}

info "🔍 Validating workspace..."

errors=0

required_dirs=(
  apps
  contracts
  crates
  docs
  plugins
  scripts
  sdk
  site
  tools/ai/rust
  tools/automation
)

required_files=(
  Cargo.toml
  Cargo.lock
  Makefile
  sdk/Cargo.toml
  sdk/Cargo.lock
  tools/ai/rust/Cargo.toml
  tools/ai/rust/Cargo.lock
)

for dir in "${required_dirs[@]}"; do
  if [ -d "$repo_root/$dir" ]; then
    pass "$dir exists"
  else
    fail "Missing directory: $dir"
    ((errors += 1))
  fi
done

for file in "${required_files[@]}"; do
  if [ -f "$repo_root/$file" ]; then
    pass "$file exists"
  else
    fail "Missing file: $file"
    ((errors += 1))
  fi
done

if ! command -v cargo >/dev/null 2>&1; then
  fail "cargo is required to validate Cargo workspaces"
  ((errors += 1))
else
  workspace_manifests=(
    Cargo.toml
    sdk/Cargo.toml
    tools/ai/rust/Cargo.toml
  )

  for manifest in "${workspace_manifests[@]}"; do
    if cargo metadata \
      --locked \
      --no-deps \
      --format-version 1 \
      --manifest-path "$repo_root/$manifest" \
      >/dev/null; then
      pass "$manifest has valid locked metadata"
    else
      fail "$manifest failed locked Cargo metadata validation"
      ((errors += 1))
    fi
  done
fi

if ((errors > 0)); then
  printf '❌ Workspace validation failed with %d error(s).\n' "$errors" >&2
  exit 1
fi

info "✅ Workspace validation succeeded!"
