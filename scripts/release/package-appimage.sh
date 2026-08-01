#!/usr/bin/env bash
set -euo pipefail

archive="${1:?usage: package-appimage.sh <archive.AppImage>}"
appdir="PolyGlid.AppDir"
appimagetool="appimagetool-x86_64.AppImage"

if [[ -e "$archive" || -e "$appdir" ]]; then
  echo "Refusing to overwrite an existing archive or AppDir" >&2
  exit 1
fi

mkdir -p "$appdir/usr/bin"
cp target/release/polyglid-desktop "$appdir/usr/bin/"

cat > "$appdir/polyglid-desktop.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=PolyGlid
Exec=polyglid-desktop
Icon=polyglid-desktop
Categories=Development;
Terminal=false
EOF

# TODO: replace with real PolyGlid branding once artwork is available.
# AppImage requires a top-level icon; a generic placeholder unblocks
# packaging without inventing unapproved branding.
icon="$appdir/polyglid-desktop.png"
base64 -d > "$icon" <<'EOF'
iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=
EOF

ln -s usr/bin/polyglid-desktop "$appdir/AppRun"

if [[ ! -x "$appimagetool" ]]; then
  curl --fail --location --output "$appimagetool" \
    "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
  chmod +x "$appimagetool"
fi

ARCH=x86_64 "./$appimagetool" "$appdir" "$archive"
