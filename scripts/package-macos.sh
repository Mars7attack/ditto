#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mode="${1:-release}"
case "$mode" in release) cargo build --release --locked ;; debug) cargo build --locked ;; *) echo 'Usage: package-macos.sh [release|debug]' >&2; exit 2 ;; esac
app_path="${2:-dist/Ditto.app}"
mkdir -p "$app_path/Contents/MacOS" "$app_path/Contents/Resources"
cp "target/$mode/ditto" "$app_path/Contents/MacOS/ditto.next"
mv -f "$app_path/Contents/MacOS/ditto.next" "$app_path/Contents/MacOS/ditto"
cp assets/LICENSE-PCFACE.txt "$app_path/Contents/Resources/Font-License.txt"
cat > "$app_path/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>Ditto</string>
<key>CFBundleDisplayName</key><string>Ditto</string>
<key>CFBundleIdentifier</key><string>org.ditto.art</string>
<key>CFBundleExecutable</key><string>ditto</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.5.0</string>
<key>CFBundleVersion</key><string>18</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
codesign --force --sign - "$app_path"
if [ "$mode" = release ] && [ "$app_path" = dist/Ditto.app ]; then
  /usr/bin/ditto -c -k --sequesterRsrc --keepParent "$app_path" dist/Ditto-macOS-arm64.zip
fi
printf 'Application: %s/%s\n' "$PWD" "$app_path"
