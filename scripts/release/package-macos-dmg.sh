#!/usr/bin/env bash
set -euo pipefail

archive="${1:?usage: package-macos-dmg.sh <archive.dmg>}"
app_bundle="PolyGlid.app"
staging="PolyGlid.dmg-staging"
version="$(tr -d '[:space:]' < version.txt)"

if [[ -e "$archive" || -e "$app_bundle" || -e "$staging" ]]; then
  echo "Refusing to overwrite an existing DMG, app bundle, or staging directory" >&2
  exit 1
fi

mkdir -p "$app_bundle/Contents/MacOS" "$app_bundle/Contents/Resources"
cp target/release/polyglid-desktop "$app_bundle/Contents/MacOS/PolyGlid"
chmod +x "$app_bundle/Contents/MacOS/PolyGlid"
cp README.md LICENSE-MIT LICENSE-APACHE "$app_bundle/Contents/Resources/"

cat > "$app_bundle/Contents/Resources/runtime-directories.md" <<'EOF'
# PolyGlid runtime directories

The first launch creates configuration, cache, logs, plugins, reports, the
database, and the default workspace. Existing installations are opened
idempotently and upgraded through the database migration system.

Set `POLYGLID_DATA_DIR` or `POLYGLID_WORKSPACE_ROOT` before launching to use
portable or isolated locations.
EOF

cat > "$app_bundle/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDisplayName</key>
  <string>PolyGlid</string>
  <key>CFBundleExecutable</key>
  <string>PolyGlid</string>
  <key>CFBundleIdentifier</key>
  <string>io.polyglid.PolyGlid</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>PolyGlid</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>$version</string>
  <key>CFBundleVersion</key>
  <string>$version</string>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.developer-tools</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
EOF

scripts/release/verify-macos-app.sh "$app_bundle"
mkdir -p "$staging"
mv "$app_bundle" "$staging/"
ln -s /Applications "$staging/Applications"
hdiutil create -volname PolyGlid -srcfolder "$staging" -ov -format UDZO "$archive"
