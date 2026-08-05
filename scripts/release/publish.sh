#!/usr/bin/env bash
set -euo pipefail

: "${GH_TOKEN:?GH_TOKEN is required}"
: "${TAG:?TAG is required}"

assets_dir="${1:-release-assets}"

if [[ ! -d "$assets_dir" ]]; then
  echo "Release assets directory does not exist: $assets_dir" >&2
  exit 1
fi

(
  cd "$assets_dir"
  sha256sum polyglid-* | sort -k 2 > SHA256SUMS
)

scripts/release/generate-update-manifest.sh "$assets_dir" "$TAG"
scripts/release/verify-update-manifest.sh "$assets_dir" "$TAG"

gh release upload "$TAG" "$assets_dir"/* --clobber
gh release edit "$TAG" --draft=false
