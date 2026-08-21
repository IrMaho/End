#!/usr/bin/env bash
# 👑 Multi-Platform Release Package Builder for End Language v1.0.0 (POSIX)
set -e

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
VERSION="v1.0.0"

echo "👑 Building End Language $VERSION Release Packages..."
mkdir -p "$DIST_DIR"

cd "$ROOT_DIR/endc"
cargo build --release

# Linux x64 Staging
STAGE_DIR="$DIST_DIR/staging-linux-x64"
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR/bin"

cp "$ROOT_DIR/endc/target/release/endc" "$STAGE_DIR/bin/endc"
cp "$ROOT_DIR/endc/target/release/endc" "$STAGE_DIR/bin/end"
cp -r "$ROOT_DIR/std" "$STAGE_DIR/std"
cp "$ROOT_DIR/README.md" "$STAGE_DIR/README.md"
cp "$ROOT_DIR/LICENSE" "$STAGE_DIR/LICENSE"
cp "$ROOT_DIR/Architecture.toml" "$STAGE_DIR/Architecture.toml"

cd "$STAGE_DIR"
tar -czf "$DIST_DIR/end-$VERSION-linux-x64.tar.gz" *

# Generate Checksums
cd "$DIST_DIR"
if command -v sha256sum >/dev/null 2>&1; then
    sha256sum end-*.tar.gz >> SHA256SUMS.txt || true
elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 end-*.tar.gz >> SHA256SUMS.txt || true
fi

echo "✔ Release package created: $DIST_DIR/end-$VERSION-linux-x64.tar.gz"
