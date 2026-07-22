#!/usr/bin/env bash
set -euo pipefail

test_dir=$(mktemp -d)
trap 'rm -rf "$test_dir"' EXIT
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

repo="$test_dir/repo"
mkdir -p "$repo"
git -C "$repo" init -q
git -C "$repo" config user.name "PolyGlid CI"
git -C "$repo" config user.email "ci@polyglid.invalid"

touch "$repo/.gitkeep"
git -C "$repo" add .gitkeep
git -C "$repo" commit -qm "baseline"

commit_file() {
  local path=$1
  mkdir -p "$(dirname "$repo/$path")"
  printf 'change\n' > "$repo/$path"
  git -C "$repo" add "$path"
  git -C "$repo" commit -qm "change $path"
}

assert_flag() {
  local output=$1
  local key=$2
  local expected=$3
  jq -e --arg key "$key" --argjson expected "$expected" \
    '.[$key] == $expected' <<<"$output" >/dev/null
}

detect_script="$script_dir/detect-changes.sh"

commit_file README.md
result=$(cd "$repo" && bash "$detect_script" HEAD^ HEAD)
assert_flag "$result" docs true
assert_flag "$result" full false

commit_file apps/cli/example.rs
result=$(cd "$repo" && bash "$detect_script" HEAD^ HEAD)
assert_flag "$result" rust_core true
assert_flag "$result" full false

commit_file .github/workflows/example.yml
result=$(cd "$repo" && bash "$detect_script" HEAD^ HEAD)
assert_flag "$result" workflows true
assert_flag "$result" full true

commit_file Makefile
result=$(cd "$repo" && bash "$detect_script" HEAD^ HEAD)
assert_flag "$result" scripts true
assert_flag "$result" full false

commit_file package.json
result=$(cd "$repo" && bash "$detect_script" HEAD^ HEAD)
assert_flag "$result" scripts true
assert_flag "$result" full false

commit_file tools/automation/scripts/example.sh
result=$(cd "$repo" && bash "$detect_script" HEAD^ HEAD)
assert_flag "$result" scripts true
assert_flag "$result" full false

commit_file unclassified/example.txt
result=$(cd "$repo" && bash "$detect_script" HEAD^ HEAD)
assert_flag "$result" unknown true
assert_flag "$result" full true

result=$(cd "$repo" && bash "$detect_script" 0000000000000000000000000000000000000000 HEAD)
assert_flag "$result" full true

echo "Change detection tests passed"
