#!/bin/bash
set -euo pipefail
# Detect which areas of the polyglid project changed between two git refs.
# Outputs a JSON object with boolean flags for each area.

BASE="${1:-main}"
HEAD="${2:-HEAD}"

changed=$(git diff --name-only "$BASE..$HEAD" 2>/dev/null || echo "")

# Initialize all flags
site=false
rust_core=false
wasm=false
docs=false
configs=false
infra=false
workflows=false
ai_engine=false
sdk=false
scripts=false
repinfo=false
root=false

for f in $changed; do
  case "$f" in
    slices/site/*)          site=true ;;
    Cargo.lock | Cargo.toml) site=true; root=true ;;
    slices/apps/* | slices/engine/* | slices/configs/config/* | slices/contracts/* | slices/plugins/*)
      rust_core=true ;;
    slices/plugins/recon-probe/* | *.wit | wit/*)
      wasm=true ;;
    docs/*)                 docs=true ;;
    configs/*)              configs=true ;;
    infrastructure/*)       infra=true ;;
    .github/*)              workflows=true ;;
    .workspace/ai/*)        ai_engine=true ;;
    sdk/*)                  sdk=true ;;
    scripts/*)              scripts=true ;;
    repinfo.json)           repinfo=true ;;
    Makefile | package.json | .gitignore | .gitattributes | .editorconfig)
      root=true ;;
  esac
done

jq -n \
  --argjson site "$site" \
  --argjson rust_core "$rust_core" \
  --argjson wasm "$wasm" \
  --argjson docs "$docs" \
  --argjson configs "$configs" \
  --argjson infra "$infra" \
  --argjson workflows "$workflows" \
  --argjson ai_engine "$ai_engine" \
  --argjson sdk "$sdk" \
  --argjson scripts "$scripts" \
  --argjson repinfo "$repinfo" \
  --argjson root "$root" \
  '{site:$site, rust_core:$rust_core, wasm:$wasm, docs:$docs, configs:$configs, infra:$infra, workflows:$workflows, ai_engine:$ai_engine, sdk:$sdk, scripts:$scripts, repinfo:$repinfo, root:$root}'
