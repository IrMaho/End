#!/usr/bin/env bash
# ?? End Language Unix One-Line Installer (Linux & macOS)
# Usage: curl -sSf https://github.com/IrMaho/End/releases/latest/download/install.sh | sh
set -euo pipefail

INSTALL_DIR="$HOME/.end"
BIN_DIR="$INSTALL_DIR/bin"
VERSION="v0.4.0-alpha"

echo "?? Installing End Programming Language Platform ($VERSION)..."

mkdir -p "$BIN_DIR"

OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" = "Darwin" ]; then
    if [ "$ARCH" = "arm64" ]; then
        PKG_NAME="end-$VERSION-macos-arm64.tar.gz"
    else
        PKG_NAME="end-$VERSION-macos-x64.tar.gz"
    fi
else
    PKG_NAME="end-$VERSION-linux-x64.tar.gz"
fi

TAR_URL="https://github.com/IrMaho/End/releases/download/$VERSION/$PKG_NAME"
SHA_URL="https://github.com/IrMaho/End/releases/download/$VERSION/$PKG_NAME.sha256"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

TMP_TAR="$TMP_DIR/$PKG_NAME"
TMP_SHA="$TMP_DIR/$PKG_NAME.sha256"

echo "==> Downloading $PKG_NAME..."
if command -v curl >/dev/null 2>&1; then
    curl -sSL --fail "$TAR_URL" -o "$TMP_TAR" || true
    curl -sSL --fail "$SHA_URL" -o "$TMP_SHA" || true
elif command -v wget >/dev/null 2>&1; then
    wget -q "$TAR_URL" -O "$TMP_TAR" || true
    wget -q "$SHA_URL" -O "$TMP_SHA" || true
fi

if [ -f "$TMP_TAR" ] && [ -s "$TMP_TAR" ]; then
    # Cryptographic Checksum Verification
    if [ -f "$TMP_SHA" ] && [ -s "$TMP_SHA" ]; then
        echo "==> Verifying SHA-256 checksum..."
        EXPECTED_SHA="$(awk '{print $1}' "$TMP_SHA")"
        if command -v sha256sum >/dev/null 2>&1; then
            ACTUAL_SHA="$(sha256sum "$TMP_TAR" | awk '{print $1}')"
        elif command -v shasum >/dev/null 2>&1; then
            ACTUAL_SHA="$(shasum -a 256 "$TMP_TAR" | awk '{print $1}')"
        else
            ACTUAL_SHA=""
        fi

        if [ -n "$ACTUAL_SHA" ] && [ "$EXPECTED_SHA" = "$ACTUAL_SHA" ]; then
            echo "? SHA-256 Verified: $ACTUAL_SHA"
        elif [ -n "$ACTUAL_SHA" ]; then
            echo "? ERROR: SHA-256 Checksum Mismatch!"
            echo "Expected: $EXPECTED_SHA"
            echo "Actual:   $ACTUAL_SHA"
            exit 1
        fi
    fi

    echo "==> Extracting archive..."
    tar -xzf "$TMP_TAR" -C "$INSTALL_DIR"
    chmod +x "$BIN_DIR/end" "$BIN_DIR/endc" 2>/dev/null || true
    echo "?? SUCCESS: End Language $VERSION installed successfully into $INSTALL_DIR!"
else
    echo "? Pre-built binary package not found for $VERSION. You can build from source: cargo build --release"
fi
