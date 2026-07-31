#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repository_root"

release_branch="${1:-}"
if [[ -z "$release_branch" ]]; then
  echo "usage: $0 <release-please-branch>" >&2
  exit 2
fi

case "$release_branch" in
  release-please*) ;;
  *)
    echo "refusing to push lockfile outside a Release Please branch: $release_branch" >&2
    exit 2
    ;;
esac

cargo update --workspace

if git diff --quiet -- Cargo.lock; then
  echo "Cargo.lock already matches the workspace version"
  exit 0
fi

git diff --check -- Cargo.lock
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git add Cargo.lock
git commit -m "chore: synchronize Cargo.lock for release"
git push origin "HEAD:refs/heads/$release_branch"
