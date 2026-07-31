#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repository_root"

release_version="$(tr -d '[:space:]' < version.txt)"
if [[ ! "$release_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([+-][0-9A-Za-z.-]+)?$ ]]; then
  echo "version.txt does not contain a semantic version: $release_version" >&2
  exit 1
fi

manifest_version="$(jq -er '.["."]' .release-please-manifest.json)"
if [[ "$manifest_version" != "$release_version" ]]; then
  echo "release manifest version $manifest_version does not match version.txt $release_version" >&2
  exit 1
fi

jq -e '
  .["release-type"] == "simple" and
  .packages["."]["extra-files"][] == {
    "type": "toml",
    "path": "Cargo.toml",
    "jsonpath": "$.workspace.package.version"
  }
' release-please-config.json >/dev/null

mismatched_packages="$(
  cargo metadata --format-version 1 --no-deps |
    jq -r --arg version "$release_version" \
      '.packages[] | select(.source == null and .version != $version) | "\(.name) \(.version)"'
)"
if [[ -n "$mismatched_packages" ]]; then
  echo "workspace packages do not inherit release version $release_version:" >&2
  echo "$mismatched_packages" >&2
  exit 1
fi

echo "Release configuration is synchronized at $release_version"
