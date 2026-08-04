#!/usr/bin/env bash
set -euo pipefail

package_dir="${1:?usage: verify-unix-package.sh <package-directory>}"

test -x "$package_dir/polyglid-desktop"
test -f "$package_dir/README.md"
test -f "$package_dir/LICENSE-MIT"
test -f "$package_dir/LICENSE-APACHE"
test -f "$package_dir/runtime-directories.md"
grep -Fq 'POLYGLID_DATA_DIR' "$package_dir/runtime-directories.md"

echo "Unix package validation passed: $package_dir"
