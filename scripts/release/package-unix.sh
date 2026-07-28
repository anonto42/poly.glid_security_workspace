#!/usr/bin/env bash
set -euo pipefail

archive="${1:?usage: package-unix.sh <archive>}"
package_dir="package"

if [[ -e "$archive" || -e "$package_dir" ]]; then
  echo "Refusing to overwrite an existing archive or package directory" >&2
  exit 1
fi

mkdir "$package_dir"
cp target/release/polyglid-desktop "$package_dir/"
cp README.md LICENSE-MIT LICENSE-APACHE "$package_dir/"
tar -C "$package_dir" -czf "$archive" .
