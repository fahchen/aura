#!/bin/bash
set -e

APP_NAME="Aura"
BUNDLE_DIR="target/release/${APP_NAME}.app"

# Build release
cargo build --release -p aura-daemon

# Create bundle structure
mkdir -p "${BUNDLE_DIR}/Contents/MacOS"
mkdir -p "${BUNDLE_DIR}/Contents/Resources"

# Copy files
cp target/release/aura "${BUNDLE_DIR}/Contents/MacOS/"
cp crates/aura-daemon/assets/Info.plist "${BUNDLE_DIR}/Contents/"
cp crates/aura-daemon/assets/AppIcon.icns "${BUNDLE_DIR}/Contents/Resources/"

echo "✓ Bundle created at ${BUNDLE_DIR}"
