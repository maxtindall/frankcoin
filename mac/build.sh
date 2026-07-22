#!/bin/bash
# Builds "frankcoin miner.app". Needs only the Swift toolchain that ships with
# the Xcode Command Line Tools -- no Xcode project, no third-party packages.
set -euo pipefail
cd "$(dirname "$0")"

APP="build/frankcoin miner.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

echo "compiling…"
swiftc -O -parse-as-library \
  Sources/FrankMinerCore/*.swift Sources/FrankMiner/App.swift \
  -o "$APP/Contents/MacOS/frankminer"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>              <string>frankcoin miner</string>
  <key>CFBundleDisplayName</key>       <string>frankcoin miner</string>
  <key>CFBundleIdentifier</key>        <string>website.0state.frankminer</string>
  <key>CFBundleExecutable</key>        <string>frankminer</string>
  <key>CFBundlePackageType</key>       <string>APPL</string>
  <key>CFBundleShortVersionString</key><string>1.0</string>
  <key>CFBundleVersion</key>           <string>1</string>
  <key>LSMinimumSystemVersion</key>    <string>13.0</string>
  <key>NSHighResolutionCapable</key>   <true/>
  <key>LSApplicationCategoryType</key> <string>public.app-category.utilities</string>
</dict>
</plist>
PLIST

echo 'APPL????' > "$APP/Contents/PkgInfo"

# Ad-hoc signature. Enough for the app to run once the user allows it in
# System Settings; it is NOT notarisation. See README for what that needs.
codesign --force --deep --sign - "$APP" 2>/dev/null || \
  echo "note: codesign unavailable — the app still runs, Gatekeeper will ask twice"

echo "built  $APP"
