#!/usr/bin/env bash
# Build CloudNode in release mode and package it for the system this runs on, into dist/:
#
#   macOS    cloudnode-<ver>-macos-universal.dmg          CloudNode.app (Apple silicon + Intel)
#            cloudnode-tui-<ver>-macos-universal.tar.gz   the console app
#   Windows  cloudnode-<ver>-windows-x64-portable.zip     cloudnode.exe and cloudnode-tui.exe
#   Linux    cloudnode-<ver>-linux-<arch>.tar.gz          bin/ and share/ (desktop entry, icons)
#
# Usage: packaging/package.sh
#
# .github/workflows/release.yml runs it on each platform; it works the same on a developer
# machine (on Windows, from Git Bash). The version comes from [workspace.package] in Cargo.toml.
# The builds are not signed with a developer certificate or notarized yet (ROADMAP.md, M7).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)"
if [ -z "$VERSION" ]; then
  echo "error: no version in [workspace.package] in Cargo.toml" >&2
  exit 1
fi
APP_ID=com.tradecloud.cloudnode
DIST="$ROOT/dist"
WORK="$ROOT/target/package"
DOCS=(LICENSE-MIT LICENSE-APACHE NOTICE README.md)

rm -rf "$DIST" "$WORK"
mkdir -p "$DIST" "$WORK"

build() {
  cargo build --release --locked -p cloudnode -p cloudnode-tui "$@"
}

package_macos() {
  # Keep in sync with LSMinimumSystemVersion in packaging/macos/Info.plist.in.
  export MACOSX_DEPLOYMENT_TARGET=11.0
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  build --target aarch64-apple-darwin
  build --target x86_64-apple-darwin

  local bin
  mkdir -p "$WORK/universal"
  for bin in cloudnode cloudnode-tui; do
    lipo -create -output "$WORK/universal/$bin" \
      "target/aarch64-apple-darwin/release/$bin" "target/x86_64-apple-darwin/release/$bin"
  done

  local app="$WORK/dmg/CloudNode.app"
  mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
  cp "$WORK/universal/cloudnode" "$app/Contents/MacOS/CloudNode"
  cp assets/app-icon/cloudnode.icns "$app/Contents/Resources/CloudNode.icns"
  sed -e "s/@VERSION@/$VERSION/g" -e "s/@SHORT_VERSION@/${VERSION%%-*}/g" \
    packaging/macos/Info.plist.in > "$app/Contents/Info.plist"
  # Ad-hoc signatures: Apple silicon runs no unsigned code. Without notarization Gatekeeper still
  # asks once before the first start (README, Downloads).
  codesign --force --sign - "$app"
  codesign --force --sign - "$WORK/universal/cloudnode-tui"

  ln -s /Applications "$WORK/dmg/Applications"
  # hdiutil now and then fails with "Resource busy" on CI machines; a retry is enough.
  local attempt
  for attempt in 1 2 3; do
    if hdiutil create -volname CloudNode -srcfolder "$WORK/dmg" -ov -format UDZO \
      "$DIST/cloudnode-$VERSION-macos-universal.dmg"; then
      break
    fi
    if [ "$attempt" = 3 ]; then
      echo "error: hdiutil failed three times" >&2
      exit 1
    fi
    sleep 5
  done

  local tui="cloudnode-tui-$VERSION-macos-universal"
  mkdir -p "$WORK/$tui"
  cp "$WORK/universal/cloudnode-tui" "${DOCS[@]}" "$WORK/$tui/"
  tar -C "$WORK" -czf "$DIST/$tui.tar.gz" "$tui"
}

package_windows() {
  build
  local name="cloudnode-$VERSION-windows-x64-portable"
  mkdir -p "$WORK/$name"
  cp target/release/cloudnode.exe target/release/cloudnode-tui.exe "${DOCS[@]}" "$WORK/$name/"
  if command -v 7z > /dev/null; then
    (cd "$WORK" && 7z a -tzip "$DIST/$name.zip" "$name" > /dev/null)
  else
    powershell.exe -NoProfile -Command \
      "Compress-Archive -Path '$(cygpath -w "$WORK/$name")' -DestinationPath '$(cygpath -w "$DIST/$name.zip")'"
  fi
}

package_linux() {
  build
  local arch name tree
  arch="$(uname -m)"
  name="cloudnode-$VERSION-linux-$arch"
  tree="$WORK/$name"
  mkdir -p "$tree/bin" "$tree/share/applications" "$tree/share/icons" "$tree/share/doc/cloudnode"
  cp target/release/cloudnode target/release/cloudnode-tui "$tree/bin/"
  cp "packaging/linux/$APP_ID.desktop" "$tree/share/applications/"
  cp -R assets/app-icon/hicolor "$tree/share/icons/"
  cp "${DOCS[@]}" "$tree/share/doc/cloudnode/"
  tar -C "$WORK" -czf "$DIST/$name.tar.gz" "$name"
}

case "$(uname -s)" in
  Darwin) package_macos ;;
  Linux) package_linux ;;
  MINGW* | MSYS* | CYGWIN*) package_windows ;;
  *)
    echo "error: no packaging for $(uname -s)" >&2
    exit 1
    ;;
esac

echo "==> CloudNode $VERSION in dist/:"
ls -l "$DIST"
