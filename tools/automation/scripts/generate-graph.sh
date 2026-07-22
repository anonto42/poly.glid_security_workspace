#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../../.." && pwd)

for command_name in cargo jq; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    printf 'Error: %s is required to generate the workspace graph.\n' \
      "$command_name" >&2
    exit 1
  fi
done

metadata_dir=$(mktemp -d)
trap 'rm -rf "$metadata_dir"' EXIT

cargo metadata \
  --locked \
  --no-deps \
  --format-version 1 \
  --manifest-path "$repo_root/Cargo.toml" \
  >"$metadata_dir/root.json"
cargo metadata \
  --locked \
  --no-deps \
  --format-version 1 \
  --manifest-path "$repo_root/sdk/Cargo.toml" \
  >"$metadata_dir/sdk.json"
cargo metadata \
  --locked \
  --no-deps \
  --format-version 1 \
  --manifest-path "$repo_root/tools/ai/rust/Cargo.toml" \
  >"$metadata_dir/ai.json"

jq -r -s '
  def workspace_names: ["root", "sdk", "ai"];
  def workspace_labels: ["Root workspace", "Plugin SDK workspace", "AI engine workspace"];
  def package_dir: .manifest_path | sub("/Cargo[.]toml$"; "");
  def node_id($workspace; $package): $workspace + "::" + $package.name;

  . as $workspaces
  | [
      "digraph polyglid_workspaces {",
      "  rankdir=LR;",
      "  graph [fontname=\"sans-serif\"];",
      "  node [shape=box, style=filled, fillcolor=lightblue, fontname=\"sans-serif\"];",
      "  edge [fontname=\"sans-serif\"];",
      (
        $workspaces
        | to_entries[]
        | .key as $workspace_index
        | .value as $metadata
        | (workspace_names[$workspace_index]) as $workspace_name
        | "  subgraph cluster_\($workspace_name) {",
          "    label=\(workspace_labels[$workspace_index] | @json);",
          (
            $metadata.packages[]
            | "    \(node_id($workspace_name; .) | @json) [label=\(.name | @json)];"
          ),
          "  }"
      ),
      (
        $workspaces
        | to_entries[]
        | .key as $workspace_index
        | .value as $metadata
        | (workspace_names[$workspace_index]) as $workspace_name
        | $metadata.packages[] as $source
        | $source.dependencies[]
        | select(.path != null)
        | . as $dependency
        | (
            $metadata.packages
            | map(select(package_dir == $dependency.path))
            | first
          ) as $target
        | select($target != null)
        | "  \(node_id($workspace_name; $source) | @json) -> \(node_id($workspace_name; $target) | @json);"
      ),
      "}"
    ]
  | .[]
' \
  "$metadata_dir/root.json" \
  "$metadata_dir/sdk.json" \
  "$metadata_dir/ai.json"
