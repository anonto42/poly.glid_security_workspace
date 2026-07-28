#!/usr/bin/env bash
set -euo pipefail

if ! command -v dx >/dev/null 2>&1; then
  echo "The Dioxus CLI is required. Install dioxus-cli 0.7 before building the website." >&2
  exit 1
fi

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
website_output="$repository_root/target/dx/polyglid-website/release/web/public"

if [[ -d "$website_output" ]]; then
  rm -rf -- "$website_output"
fi

(
  cd "$repository_root/apps/website"
  dx build --locked --release --platform web --debug-symbols false
)
