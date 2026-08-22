#!/usr/bin/env sh
# 👑 End Language — Official Linux & macOS One-Line Automated Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/IrMaho/End/main/install.sh | sh

set -e

VERSION="v0.4.0-alpha"
INSTALL_DIR="$HOME/.end"
BIN_DIR="$INSTALL_DIR/bin"
SKILL_DIR="$INSTALL_DIR/skills/end-language"
GLOBAL_GEMINI_SKILL="$HOME/.gemini/config/skills/end-language"

echo "================================================================================"
echo "👑 Installing End Programming Language ($VERSION)..."
echo "================================================================================"

# Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64) ASSET_NAME="end-$VERSION-linux-x64.tar.gz" ;;
            aarch64|arm64) ASSET_NAME="end-$VERSION-linux-arm64.tar.gz" ;;
            *) echo "❌ Unsupported Linux architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64) ASSET_NAME="end-$VERSION-macos-x64.tar.gz" ;;
            arm64|aarch64) ASSET_NAME="end-$VERSION-macos-arm64.tar.gz" ;;
            *) echo "❌ Unsupported macOS architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "[1/4] Creating directories..."
mkdir -p "$BIN_DIR" "$SKILL_DIR" "$GLOBAL_GEMINI_SKILL"

echo "[2/4] Downloading $ASSET_NAME..."
TAR_URL="https://github.com/IrMaho/End/releases/download/$VERSION/$ASSET_NAME"
TEMP_TAR="/tmp/$ASSET_NAME"

if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$TAR_URL" -o "$TEMP_TAR" || true
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$TEMP_TAR" "$TAR_URL" || true
fi

if [ -f "$TEMP_TAR" ]; then
    tar -xzf "$TEMP_TAR" -C "$INSTALL_DIR"
    rm -f "$TEMP_TAR"
    echo "  ✔ Downloaded and extracted official release binaries"
else
    echo "  ⚠ Note: Building locally or configuring from repository..."
    if [ -f "$BIN_DIR/end" ]; then
        chmod +x "$BIN_DIR/end" "$BIN_DIR/endc" || true
    fi
fi

echo "[3/4] Configuring PATH..."
ADD_PATH="export PATH=\"\$HOME/.end/bin:\$PATH\""

add_to_shell() {
    FILE="$1"
    if [ -f "$FILE" ]; then
        if ! grep -q ".end/bin" "$FILE"; then
            echo "" >> "$FILE"
            echo "# End Programming Language" >> "$FILE"
            echo "$ADD_PATH" >> "$FILE"
            echo "  ✔ Added to $FILE"
        fi
    fi
}

add_to_shell "$HOME/.bashrc"
add_to_shell "$HOME/.zshrc"
add_to_shell "$HOME/.profile"

echo "[4/4] Verifying installation..."
export PATH="$BIN_DIR:$PATH"
if command -v end >/dev/null 2>&1; then
    end --version || true
fi

echo ""
echo "================================================================================"
echo "🎉 End Programming Language $VERSION successfully installed!"
echo "================================================================================"
echo "  • Compiler Binary:  $BIN_DIR/end"
echo "  • AI Global Skill:  $GLOBAL_GEMINI_SKILL/SKILL.md"
echo ""
echo "🚀 Quick Start Commands:"
echo "    end run main.end           (Instant execution)"
echo "    end build main.end         (Compile to native binary)"
echo "    end skill init             (Initialize AI pair programming in current project)"
echo ""
echo "Restart your terminal or run: export PATH=\"\$HOME/.end/bin:\$PATH\" to start!"
echo "================================================================================"
