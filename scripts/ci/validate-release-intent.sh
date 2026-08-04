#!/usr/bin/env bash
set -euo pipefail

: "${BASE_SHA:?BASE_SHA is required}"
: "${HEAD_SHA:?HEAD_SHA is required}"
: "${PR_TITLE:?PR_TITLE is required}"

conventional_pattern='^(feat|fix|perf|refactor|revert|build|ci|docs|style|test|chore)(\([a-z0-9._/-]+\))?(!)?: .+'
feature_pattern='^feat(\([a-z0-9._/-]+\))?(!)?: .+'
test_pattern='^[Tt]est([/:_-]).+'

if [[ ! "$PR_TITLE" =~ $conventional_pattern && ! "$PR_TITLE" =~ $test_pattern ]]; then
  echo "::error::PR title must use Conventional Commits, or start with 'test/' for validation-only work"
  exit 1
fi

added_packages="$(
  git diff --diff-filter=A --name-only "$BASE_SHA" "$HEAD_SHA" -- \
    'apps/*/Cargo.toml' \
    'crates/*/Cargo.toml'
)"

if [[ -n "$added_packages" && ! "$PR_TITLE" =~ $feature_pattern ]]; then
  echo "::error::A new app or crate is a feature; use a 'feat:' PR title"
  printf '%s\n' "$added_packages"
  exit 1
fi
