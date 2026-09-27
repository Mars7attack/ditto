#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mode="${1:-release}"
public_version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
build_id="$(cat BUILD_ID)"
case "$build_id" in X[0-9][0-9][0-9][0-9]) ;; *) echo 'Invalid BUILD_ID' >&2; exit 1 ;; esac
bundle_build="$(printf '%s' "$build_id" | sed 's/^X0*//')"
case "$mode" in release) cargo build --release --locked ;; debug) cargo build --locked ;; *) echo 'Usage: package-macos.sh [release|debug]' >&2; exit 2 ;; esac
app_path="${2:-dist/Ditto.app}"
mkdir -p "$app_path/Contents/MacOS" "$app_path/Contents/Resources"
cp "target/$mode/ditto" "$app_path/Contents/MacOS/ditto.next"
mv -f "$app_path/Contents/MacOS/ditto.next" "$app_path/Contents/MacOS/ditto"
cp LICENSE "$app_path/Contents/Resources/LICENSE"
cp BUILD_ID "$app_path/Contents/Resources/BUILD_ID"
cp assets/LICENSE-PCFACE.txt "$app_path/Contents/Resources/Font-License.txt"
cat > "$app_path/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>Ditto</string>
<key>CFBundleDisplayName</key><string>Ditto</string>
<key>CFBundleIdentifier</key><string>org.ditto.art</string>
<key>CFBundleExecutable</key><string>ditto</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>$public_version</string>
<key>CFBundleVersion</key><string>$bundle_build</string>
<key>DittoBuildID</key><string>$build_id</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
codesign --force --sign - "$app_path"
if [ "$mode" = release ] && [ "$app_path" = dist/Ditto.app ]; then
  /usr/bin/ditto -c -k --sequesterRsrc --keepParent "$app_path" dist/Ditto-macOS-arm64.zip
fi
printf 'Application: %s/%s\n' "$PWD" "$app_path"
