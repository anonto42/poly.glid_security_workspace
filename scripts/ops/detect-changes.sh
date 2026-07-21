#!/bin/bash
set -euo pipefail
# Detect which areas of the polyglid project changed between two git refs.
# Outputs a JSON object with boolean flags for each area.

BASE="${1:-main}"
HEAD="${2:-HEAD}"
invalid_base=false

if ! git rev-parse --verify "$BASE^{commit}" >/dev/null 2>&1; then
  BASE=$(git hash-object -t tree /dev/null)
  invalid_base=true
fi
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
unknown=false
full=$invalid_base

while IFS= read -r f; do
  [ -n "$f" ] || continue
  case "$f" in
    site/*)                 site=true ;;
    Cargo.lock | Cargo.toml) site=true; root=true ;;
    crates/config/*)        rust_core=true; configs=true ;;
    apps/* | crates/*)
      rust_core=true ;;
    plugins/* | contracts/* | *.wit | wit/*)
      wasm=true ;;
    docs/*)                 docs=true ;;
    configs/*)              configs=true ;;
    infrastructure/*)       infra=true ;;
    .github/*)              workflows=true; full=true ;;
    tools/ai/*)             ai_engine=true ;;
    sdk/*)                  sdk=true ;;
    scripts/ops/detect-changes.sh) scripts=true; full=true ;;
    scripts/*)              scripts=true ;;
    repinfo.json)           repinfo=true ;;
    README.md | ARCHITECTURE.md | SUMMARY.md)
      docs=true ;;
    Makefile | package.json | .gitignore | .gitattributes | .editorconfig)
      root=true ;;
    *)                       unknown=true ;;
  esac
done < <(git diff --name-only "$BASE" "$HEAD" --)

if [ "$unknown" = true ]; then
  full=true
fi

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
  --argjson unknown "$unknown" \
  --argjson full "$full" \
  '{site:$site, rust_core:$rust_core, wasm:$wasm, docs:$docs, configs:$configs, infra:$infra, workflows:$workflows, ai_engine:$ai_engine, sdk:$sdk, scripts:$scripts, repinfo:$repinfo, root:$root, unknown:$unknown, full:$full}'
