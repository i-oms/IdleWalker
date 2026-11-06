#!/bin/sh
# Builds the one macOS download: a zip holding IdleWalker.app with the README and licenses,
# whose single executable runs natively on both Apple Silicon and Intel.
#   package-macos.sh VERSION ARM64_BINARY X64_BINARY OUT_DIR
# Runs on macOS: it needs lipo, sips, iconutil, codesign and ditto.
set -e

version=$1
arm64=$2
x64=$3
out=$4
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
name="idlewalker-$version-macos"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
stage="$work/$name"
app="$stage/IdleWalker.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" "$stage" "$out"

lipo -create -output "$app/Contents/MacOS/idlewalker" "$arm64" "$x64"
chmod +x "$app/Contents/MacOS/idlewalker"

iconset="$work/IdleWalker.iconset"
mkdir "$iconset"
for size in 16 32 128 256; do
    sips -z "$size" "$size" "$root/assets/logo.png" --out "$iconset/icon_${size}x${size}.png" > /dev/null
    double=$((size * 2))
    sips -z "$double" "$double" "$root/assets/logo.png" --out "$iconset/icon_${size}x${size}@2x.png" > /dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/IdleWalker.icns"

# LSUIElement: a status bar app, so no Dock icon even while it starts.
# NSHighResolutionCapable: without it, a bundled app is drawn blurry on Retina screens.
# NSAppleEventsUsageDescription: the terminal prompt window sets Terminal's font through
# AppleScript. Without this text macOS refuses every Apple event the app sends, with no
# prompt (-1743); with it, macOS asks once to let IdleWalker control Terminal.
cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>IdleWalker</string>
    <key>CFBundleDisplayName</key><string>IdleWalker</string>
    <key>CFBundleIdentifier</key><string>com.idlewalker.app</string>
    <key>CFBundleExecutable</key><string>idlewalker</string>
    <key>CFBundleIconFile</key><string>IdleWalker</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>CFBundleShortVersionString</key><string>$version</string>
    <key>CFBundleVersion</key><string>$version</string>
    <key>LSUIElement</key><true/>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSAppleEventsUsageDescription</key><string>IdleWalker sets the Nerd Font in Terminal's profile so the terminal prompt's icons show.</string>
</dict>
</plist>
EOF

# The documents that must travel with every download.
cp "$root/README.md" "$root/LICENSE" "$root/THIRD_PARTY_NOTICES.md" "$stage/"
cp -R "$root/LICENSES" "$stage/"

# A copy inside the bundle too, so the licenses stay with the app if it is moved alone.
cat "$root/LICENSE" "$root/THIRD_PARTY_NOTICES.md" "$root"/LICENSES/* > "$app/Contents/Resources/licenses.txt"

# An ad-hoc signature: not trusted by Gatekeeper, but Apple Silicon needs a valid one, and
# joining the two binaries with lipo is not guaranteed to leave one valid for the bundle.
codesign --force --deep --sign - "$app"
codesign --verify --deep --strict "$app"

zip="$out/$name.zip"
# ditto keeps the bundle's permissions and signature, and --keepParent keeps the top folder.
ditto -c -k --sequesterRsrc --keepParent "$stage" "$zip"

# Check what was actually shipped, not the folder it was made from.
verify="$work/verify"
mkdir "$verify"
ditto -x -k "$zip" "$verify"
shipped="$verify/$name/IdleWalker.app"
status=0
"$shipped/Contents/MacOS/idlewalker" --version || status=1
archs=$(lipo -archs "$shipped/Contents/MacOS/idlewalker")
case "$archs" in
    *arm64*x86_64* | *x86_64*arm64*) ;;
    *) echo "Expected a universal binary, got: $archs" >&2; status=1 ;;
esac
codesign --verify --deep --strict "$shipped" || status=1
for doc in README.md LICENSE THIRD_PARTY_NOTICES.md; do
    [ -f "$verify/$name/$doc" ] || { echo "Missing $doc from the zip" >&2; status=1; }
done
[ -d "$verify/$name/LICENSES" ] || { echo "Missing LICENSES from the zip" >&2; status=1; }
exit "$status"
