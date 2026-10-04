#!/bin/sh
set -e

# Easy Docker 1-Line Installer for macOS & Linux

REPO="Brunopaun/easy-docker"
INSTALL_DIR="/usr/local/bin"

echo "🐳 Installing easy-docker..."

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    Darwin)
        if [ "$ARCH" = "arm64" ]; then
            TARGET="aarch64-apple-darwin"
        else
            TARGET="x86_64-apple-darwin"
        fi
        ;;
    *)
        echo "❌ Unsupported Operating System: $OS"
        exit 1
        ;;
esac

TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$TAG" ]; then
    echo "❌ Failed to fetch latest release tag for $REPO"
    exit 1
fi

URL="https://github.com/$REPO/releases/download/$TAG/easy-docker-$TARGET.tar.gz"

echo "📦 Downloading easy-docker $TAG for $TARGET..."
TMP_DIR=$(mktemp -d)
curl -sSL "$URL" | tar -xz -C "$TMP_DIR"

if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/easy-docker" "$INSTALL_DIR/easy-docker"
else
    echo "🔑 Sudo permissions required to install to $INSTALL_DIR:"
    sudo mv "$TMP_DIR/easy-docker" "$INSTALL_DIR/easy-docker"
fi

rm -rf "$TMP_DIR"
echo "✅ easy-docker $TAG successfully installed to $INSTALL_DIR/easy-docker!"
echo "🚀 Run 'easy-docker' to launch the TUI!"
