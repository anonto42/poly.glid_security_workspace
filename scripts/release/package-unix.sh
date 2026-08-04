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
cat > "$package_dir/runtime-directories.md" <<'EOF'
# PolyGlid runtime directories

The first launch creates the configuration, cache, logs, plugins, reports,
database, and default workspace directories. Existing installations are
opened idempotently and upgraded through the database migration system.

Set `POLYGLID_DATA_DIR` or `POLYGLID_WORKSPACE_ROOT` before launching to use
portable or isolated locations.
EOF
scripts/release/verify-unix-package.sh "$package_dir"
tar -C "$package_dir" -czf "$archive" .
