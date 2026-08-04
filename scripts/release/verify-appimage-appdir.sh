#!/usr/bin/env bash
set -euo pipefail

appdir="${1:?usage: verify-appimage-appdir.sh <AppDir>}"

test -x "$appdir/usr/bin/polyglid-desktop"
test -L "$appdir/AppRun"
test -x "$appdir/AppRun"
test -f "$appdir/polyglid-desktop.desktop"
test -f "$appdir/polyglid-desktop.png"
test -f "$appdir/usr/share/metainfo/io.polyglid.PolyGlid.appdata.xml"
test -f "$appdir/usr/share/doc/polyglid/runtime-directories.md"

grep -Fqx 'Exec=polyglid-desktop' "$appdir/polyglid-desktop.desktop"
grep -Fqx '<launchable type="desktop-id">polyglid-desktop.desktop</launchable>' \
  "$appdir/usr/share/metainfo/io.polyglid.PolyGlid.appdata.xml"
grep -Fq 'POLYGLID_DATA_DIR' "$appdir/usr/share/doc/polyglid/runtime-directories.md"

echo "AppDir validation passed: $appdir"
