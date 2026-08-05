#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
validator="$repository_root/scripts/ci/validate-release-intent.sh"

run_title_check() {
  local base_sha="$1"
  local head_sha="$2"
  local title="$3"
  local expected_status="$4"
  local actual_status=0

  BASE_SHA="$base_sha" HEAD_SHA="$head_sha" PR_TITLE="$title" \
    "$validator" >/dev/null 2>&1 || actual_status=$?
  if [[ "$actual_status" -ne "$expected_status" ]]; then
    echo "unexpected validation status $actual_status for title '$title'" >&2
    exit 1
  fi
}

run_title_check HEAD HEAD "test/release package dry run" 0
run_title_check HEAD HEAD "ci(release): validate package artifacts" 0
run_title_check HEAD HEAD "release package dry run" 1

fixture_repository="$(mktemp -d)"
cleanup() { rm -rf "$fixture_repository"; }
trap cleanup EXIT
git -C "$fixture_repository" init -q
git -C "$fixture_repository" config user.name "PolyGlid CI test"
git -C "$fixture_repository" config user.email "ci-test@example.invalid"
printf '%s\n' fixture > "$fixture_repository/README.md"
git -C "$fixture_repository" add README.md
git -C "$fixture_repository" commit -q -m "chore: create validator fixture"
base_sha="$(git -C "$fixture_repository" rev-parse HEAD)"
mkdir -p "$fixture_repository/apps/example"
printf '%s\n' '[package]' > "$fixture_repository/apps/example/Cargo.toml"
git -C "$fixture_repository" add apps/example/Cargo.toml
git -C "$fixture_repository" commit -q -m "feat: add fixture app"
head_sha="$(git -C "$fixture_repository" rev-parse HEAD)"

(
  cd "$fixture_repository"
  run_title_check "$base_sha" "$head_sha" "test/release package dry run" 1
  run_title_check "$base_sha" "$head_sha" "feat(release): add fixture package" 0
)

echo "release-intent validator tests passed"
