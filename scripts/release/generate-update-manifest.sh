#!/usr/bin/env bash
set -euo pipefail

assets_dir="${1:?usage: generate-update-manifest.sh <assets-directory> <tag>}"
tag="${2:?usage: generate-update-manifest.sh <assets-directory> <tag>}"
: "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"

manifest="$assets_dir/polyglid-update.json"
version="${tag#v}"
assets_json='[]'

while IFS= read -r asset; do
  [[ -n "$asset" ]] || continue
  checksum="$(sha256sum "$assets_dir/$asset" | awk '{print $1}')"
  url="https://github.com/$GITHUB_REPOSITORY/releases/download/$tag/$asset"
  assets_json="$(jq --arg filename "$asset" --arg sha256 "$checksum" --arg url "$url" \
    '. + [{filename: $filename, sha256: $sha256, url: $url}]' <<<"$assets_json")"
done < <(find "$assets_dir" -maxdepth 1 -type f -name 'polyglid-*' \
  ! -name 'polyglid-update.json' -printf '%f\n' | sort)

jq -n \
  --argjson schema 1 \
  --arg version "$version" \
  --arg tag "$tag" \
  --arg repository "$GITHUB_REPOSITORY" \
  --argjson assets "$assets_json" \
  '{schema: $schema, version: $version, tag: $tag, repository: $repository, assets: $assets}' \
  > "$manifest"

echo "Generated update manifest: $manifest"
