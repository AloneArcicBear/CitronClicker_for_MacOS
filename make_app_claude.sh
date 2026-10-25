#claude
# baut Citron.app aus diesem Projekt (nur macOS-Bordmittel: sips, iconutil, codesign).
# ausfuehren mit:  bash make_app_claude.sh
set -euo pipefail
cd "$(dirname "$0")"
if [ -f "$HOME/.cargo/env" ]; then . "$HOME/.cargo/env"; fi

APP="Citron.app"
BIN="citron_v2"
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)

echo "==> baue release (v$VERSION)"
cargo build --release

echo "==> erstelle $APP"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "target/release/$BIN" "$APP/Contents/MacOS/$BIN"

# icon: 1024px png -> .iconset in allen groessen -> .icns
TMP=$(mktemp -d)
ICONSET="$TMP/AppIcon.iconset"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  sips -z "$s" "$s" assets/AppIcon_claude.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  sips -z $((s * 2)) $((s * 2)) assets/AppIcon_claude.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"
rm -rf "$TMP"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>$BIN</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.citron.v2</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Citron</string>
    <key>CFBundleDisplayName</key>
    <string>Citron</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.utilities</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

# ad-hoc signatur, damit macOS die app startet und die berechtigungen der app zuordnet
codesign --force --deep --sign - "$APP"

echo "==> fertig: $(pwd)/$APP"
echo "    zum installieren in den Programme-Ordner ziehen, oder: open \"$APP\""
