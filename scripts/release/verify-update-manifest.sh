#!/usr/bin/env bash
set -euo pipefail

assets_dir="${1:?usage: verify-update-manifest.sh <assets-directory> <tag>}"
tag="${2:?usage: verify-update-manifest.sh <assets-directory> <tag>}"
: "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"

manifest="$assets_dir/polyglid-update.json"
checksums="$assets_dir/SHA256SUMS"
test -f "$manifest"
test -f "$checksums"

(cd "$assets_dir" && sha256sum -c SHA256SUMS >/dev/null)
jq -e --arg tag "$tag" --arg repository "$GITHUB_REPOSITORY" \
  '.schema == 1 and .tag == $tag and .repository == $repository and (.assets | length > 0)' \
  "$manifest" >/dev/null

while IFS=$'\t' read -r filename expected_hash url; do
  test -n "$filename"
  test -f "$assets_dir/$filename"
  actual_hash="$(sha256sum "$assets_dir/$filename" | awk '{print $1}')"
  test "$actual_hash" = "$expected_hash"
  expected_url="https://github.com/$GITHUB_REPOSITORY/releases/download/$tag/$filename"
  test "$url" = "$expected_url"
done < <(jq -r '.assets[] | [.filename, .sha256, .url] | @tsv' "$manifest")

echo "Update manifest validation passed: $manifest"
