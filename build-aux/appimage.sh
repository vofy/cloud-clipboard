#!/usr/bin/env bash
# Builds Cloud_Clipboard-x86_64.AppImage: the app with GTK, plus rclone, wl-copy and pdftoppm
# (the last two from this system: install wl-clipboard and poppler-utils first).
set -euo pipefail
cd "$(dirname "$0")/.."
ID=io.github.vofy.CloudClipboard
RCLONE=v1.76.0
RCLONE_SHA256=b00f00b3266c3764413afe599d3ec316f6f43b3e0ea8a0bc1d186d2133e46df7
TOOLS=target/appimage-tools
export VERSION=${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}
export APPIMAGE_EXTRACT_AND_RUN=1 DEPLOY_GTK_VERSION=4 LDAI_OUTPUT=Cloud_Clipboard-x86_64.AppImage
# Lets AppImageUpdate and AppImage managers (AM, AppImagePool, Gear Lever) find new releases;
# linuxdeploy also writes the matching .zsync file, which the release workflow uploads.
export LDAI_UPDATE_INFORMATION="gh-releases-zsync|vofy|cloud-clipboard|latest|Cloud_Clipboard-x86_64.AppImage.zsync"

get() {
    [[ -x $TOOLS/$1 ]] && return
    curl -fsSLo "$TOOLS/$1" "$2" && chmod +x "$TOOLS/$1"
}
mkdir -p "$TOOLS"
get linuxdeploy https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
get linuxdeploy-plugin-gtk.sh https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh
if [[ ! -x $TOOLS/rclone ]]; then
    curl -fsSLo "$TOOLS/rclone.zip" "https://downloads.rclone.org/$RCLONE/rclone-$RCLONE-linux-amd64.zip"
    echo "$RCLONE_SHA256  $TOOLS/rclone.zip" | sha256sum -c --quiet
    unzip -qjo "$TOOLS/rclone.zip" '*/rclone' -d "$TOOLS"
fi

cargo build --release --locked
rm -rf AppDir
install -Dm755 "$TOOLS/rclone" AppDir/usr/bin/rclone
install -Dm644 data/$ID.metainfo.xml AppDir/usr/share/metainfo/$ID.appdata.xml
PATH=$PWD/$TOOLS:$PATH linuxdeploy --appdir AppDir \
    --executable target/release/cloud-clipboard \
    --executable "$(command -v wl-copy)" \
    --executable "$(command -v pdftoppm)" \
    --desktop-file data/$ID.desktop \
    --icon-file data/$ID.svg \
    --plugin gtk --output appimage
