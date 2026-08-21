#!/usr/bin/env bash
# 👑 End Language Unix One-Line Installer (Linux & macOS)
# Usage: curl -sSf https://github.com/IrMaho/End/releases/latest/download/install.sh | sh
set -e

INSTALL_DIR="$HOME/.end"
BIN_DIR="$INSTALL_DIR/bin"
VERSION="v1.0.0"

echo "👑 Installing End Programming Language ($VERSION)..."

mkdir -p "$BIN_DIR"

OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" = "Darwin" ]; then
    if [ "$ARCH" = "arm64" ]; then
        TAR_URL="https://github.com/IrMaho/End/releases/download/$VERSION/end-$VERSION-macos-arm64.tar.gz"
    else
        TAR_URL="https://github.com/IrMaho/End/releases/download/$VERSION/end-$VERSION-macos-x64.tar.gz"
    fi
else
    TAR_URL="https://github.com/IrMaho/End/releases/download/$VERSION/end-$VERSION-linux-x64.tar.gz"
fi

TMP_TAR="/tmp/end-$VERSION.tar.gz"
if command -v curl >/dev/null 2>&1; then
    curl -sSL "$TAR_URL" -o "$TMP_TAR" || true
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$TMP_TAR" "$TAR_URL" || true
fi

if [ -f "$TMP_TAR" ]; then
    tar -xzf "$TMP_TAR" -C "$INSTALL_DIR"
    rm -f "$TMP_TAR"
fi

# PATH Export Configuration
PROFILE_FILE=""
if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
    PROFILE_FILE="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ] || [ -f "$HOME/.bashrc" ]; then
    PROFILE_FILE="$HOME/.bashrc"
else
    PROFILE_FILE="$HOME/.profile"
fi

if ! grep -q "$BIN_DIR" "$PROFILE_FILE" 2>/dev/null; then
    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$PROFILE_FILE"
    echo "✔ Added '$BIN_DIR' to $PROFILE_FILE"
fi

echo "👑 SUCCESS: End Language $VERSION installed successfully!"
echo "Restart your terminal or run: source $PROFILE_FILE"
