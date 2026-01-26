#!/bin/bash
set -e

APP_NAME="Aura"
BUNDLE_DIR="target/release/${APP_NAME}.app"

# Build release binaries
cargo build --release -p aura-daemon -p aura-claude-code-hook

# Create bundle structure
mkdir -p "${BUNDLE_DIR}/Contents/MacOS"
mkdir -p "${BUNDLE_DIR}/Contents/Resources"

# Copy daemon binary
cp target/release/aura "${BUNDLE_DIR}/Contents/MacOS/"

# Copy hook binary (for CLI installation)
cp target/release/aura-claude-code-hook "${BUNDLE_DIR}/Contents/MacOS/"

# Copy resources
cp crates/aura-daemon/assets/Info.plist "${BUNDLE_DIR}/Contents/"
cp crates/aura-daemon/assets/AppIcon.icns "${BUNDLE_DIR}/Contents/Resources/"

echo "✓ Bundle created at ${BUNDLE_DIR}"
