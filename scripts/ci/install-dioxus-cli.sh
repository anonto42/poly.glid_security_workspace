#!/usr/bin/env bash
set -euo pipefail

dioxus_version="0.7.9"
dioxus_archive="dx-x86_64-unknown-linux-gnu.zip"
dioxus_sha256="587b426bb83623408af6cc80054a190a337406f9cb29fd4a469742d2207acefe"

if command -v dx >/dev/null 2>&1 && dx --version | grep -q "dioxus $dioxus_version"; then
  exit 0
fi

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "x86_64" ]]; then
  echo "The pinned CI installer supports Linux x86-64 only." >&2
  exit 1
fi

installation_directory="${CARGO_HOME:-$HOME/.cargo}/bin"
temporary_directory="$(mktemp -d)"
trap 'rm -rf -- "$temporary_directory"' EXIT

download_url="https://github.com/DioxusLabs/dioxus/releases/download/v${dioxus_version}/${dioxus_archive}"
archive_path="$temporary_directory/$dioxus_archive"

curl \
  --fail \
  --location \
  --proto '=https' \
  --retry 3 \
  --show-error \
  --silent \
  --tlsv1.2 \
  --output "$archive_path" \
  "$download_url"

echo "$dioxus_sha256  $archive_path" | sha256sum --check --status
unzip -q "$archive_path" -d "$temporary_directory"
install -d "$installation_directory"
install -m 0755 "$temporary_directory/dx" "$installation_directory/dx"

"$installation_directory/dx" --version | grep -q "dioxus $dioxus_version"
