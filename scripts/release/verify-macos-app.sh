#!/usr/bin/env bash
set -euo pipefail

app_bundle="${1:?usage: verify-macos-app.sh <app-bundle>}"
contents="$app_bundle/Contents"

test -x "$contents/MacOS/PolyGlid"
test -f "$contents/Info.plist"
test -f "$contents/Resources/README.md"
test -f "$contents/Resources/LICENSE-MIT"
test -f "$contents/Resources/LICENSE-APACHE"
test -f "$contents/Resources/runtime-directories.md"
if command -v plutil >/dev/null && [[ -x /usr/libexec/PlistBuddy ]]; then
  plutil -lint "$contents/Info.plist"
  test "$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$contents/Info.plist")" = "PolyGlid"
else
  xmllint --noout "$contents/Info.plist"
  grep -Fq '<key>CFBundleExecutable</key>' "$contents/Info.plist"
  grep -Fq '<string>PolyGlid</string>' "$contents/Info.plist"
fi
grep -Fq 'POLYGLID_DATA_DIR' "$contents/Resources/runtime-directories.md"

echo "macOS app validation passed: $app_bundle"
