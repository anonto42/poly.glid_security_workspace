#!/usr/bin/env bash
set -euo pipefail

target="${1:-localhost}"
component="target/wasm32-wasip1/debug/recon_probe.component.wasm"
module="target/wasm32-wasip1/debug/recon_probe.wasm"

echo "==> Building recon_probe for wasm32-wasip1"
cargo build -p recon-probe --target wasm32-wasip1

echo "==> Componentizing recon_probe"
cargo run -p polyglid-cli -- plugin componentize "$module" "$component"

echo "==> Running PolyGlid MVP against target: $target"
cargo run -p polyglid-cli -- plugin run "$component" \
  --target "$target" \
  --allow dns-resolve \
  --allow report-write

echo "==> Report output directory: reports/"
