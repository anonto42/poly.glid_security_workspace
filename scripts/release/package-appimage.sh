#!/usr/bin/env bash
set -euo pipefail

archive="${1:?usage: package-appimage.sh <archive.AppImage>}"
appdir="PolyGlid.AppDir"
appimagetool="appimagetool-x86_64.AppImage"
version="$(tr -d '[:space:]' < version.txt)"
release_date="$(date -u +%Y-%m-%d)"

if [[ -e "$archive" || -e "$appdir" ]]; then
  echo "Refusing to overwrite an existing archive or AppDir" >&2
  exit 1
fi

mkdir -p "$appdir/usr/bin"
cp target/release/polyglid-desktop "$appdir/usr/bin/"
mkdir -p "$appdir/usr/share/doc/polyglid" "$appdir/usr/share/metainfo"
cp README.md LICENSE-MIT LICENSE-APACHE "$appdir/usr/share/doc/polyglid/"

cat > "$appdir/usr/share/doc/polyglid/runtime-directories.md" <<'EOF'
# PolyGlid runtime directories

On first launch, PolyGlid creates its configuration, cache, logs, plugins,
reports, database, and default workspace directories. Existing installations
are opened idempotently and upgraded through the database migration system.

Set `POLYGLID_DATA_DIR` or `POLYGLID_WORKSPACE_ROOT` before launching to use
portable or isolated locations.
EOF

cat > "$appdir/polyglid-desktop.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=PolyGlid
Exec=polyglid-desktop
Icon=polyglid-desktop
Categories=Development;
Terminal=false
StartupWMClass=PolyGlid
EOF

cat > "$appdir/usr/share/metainfo/io.polyglid.PolyGlid.appdata.xml" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>io.polyglid.PolyGlid</id>
  <name>PolyGlid</name>
  <summary>Local-first security workspace</summary>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>MIT OR Apache-2.0</project_license>
  <description>
    <p>Manage local security projects and workspaces with explicit control over application data and plugins.</p>
  </description>
  <launchable type="desktop-id">polyglid-desktop.desktop</launchable>
  <categories>
    <category>Development</category>
    <category>Utility</category>
  </categories>
  <releases>
    <release version="$version" date="$release_date" />
  </releases>
</component>
EOF

# TODO: replace with real PolyGlid branding once artwork is available.
# AppImage requires a top-level icon; a generic placeholder unblocks
# packaging without inventing unapproved branding.
icon="$appdir/polyglid-desktop.png"
base64 -d > "$icon" <<'EOF'
iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=
EOF

ln -s usr/bin/polyglid-desktop "$appdir/AppRun"

scripts/release/verify-appimage-appdir.sh "$appdir"

if [[ ! -x "$appimagetool" ]]; then
  curl --fail --location --output "$appimagetool" \
    "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
  chmod +x "$appimagetool"
fi

ARCH=x86_64 "./$appimagetool" "$appdir" "$archive"
