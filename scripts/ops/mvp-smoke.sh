#!/usr/bin/env bash
set -euo pipefail

audit_dir=$(mktemp -d)
trap 'rm -rf "$audit_dir"' EXIT

workspace_dir="$audit_dir/workspace"
config_path="$audit_dir/polyglid.toml"
component_path="$audit_dir/recon-probe.component.wasm"
expected_report="$audit_dir/expected-report.txt"
mkdir -p "$workspace_dir"

cat > "$config_path" <<EOF
plugin_dir = "$workspace_dir/plugins"
reports_dir = "$workspace_dir/reports"
max_wasm_fuel = 25000000
default_capabilities = []
EOF

echo "==> Building the CLI host and Recon Probe"
cargo build --locked -p polyglid-cli
cargo build --locked -p recon-probe --target wasm32-wasip1

echo "==> Componentizing the Recon Probe module"
target/debug/polyglid plugin componentize \
  target/wasm32-wasip1/debug/recon_probe.wasm \
  "$component_path"
test -s "$component_path"

echo "==> Inspecting embedded component metadata"
inspect=$(POLYGLID_CONFIG="$config_path" target/debug/polyglid plugin inspect "$component_path")
printf '%s\n' "$inspect"
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
grep -Fxq "id: polyglid.recon_probe" <<<"$inspect"
grep -Fxq "version: $version" <<<"$inspect"
grep -Fxq -- "- dns-resolve" <<<"$inspect"
grep -Fxq -- "- report-write" <<<"$inspect"

echo "==> Executing the real host-to-component path"
output=$(POLYGLID_CONFIG="$config_path" target/debug/polyglid plugin run \
  "$component_path" \
  --target localhost \
  --allow dns-resolve \
  --allow report-write)
printf '%s\n' "$output"

grep -Fxq "plugin: PolyGlid Recon Probe" <<<"$output"
grep -Fxq "target: localhost" <<<"$output"
grep -Fxq "summary: 1 demo observation(s) reported for localhost." <<<"$output"
grep -Fxq -- "- [info] Loopback target" <<<"$output"
printf 'PolyGlid Recon Probe\nTarget: localhost\n' > "$expected_report"
cmp "$expected_report" "$workspace_dir/reports/recon-probe-localhost.txt"

echo "MVP smoke test passed"
