#!/bin/bash
set -e

APP_NAME="Aura"
BUNDLE_DIR="target/release/${APP_NAME}.app"

# Build release binary
cargo build --release -p aura

# Create bundle structure
mkdir -p "${BUNDLE_DIR}/Contents/MacOS"
mkdir -p "${BUNDLE_DIR}/Contents/Resources"

# Copy binary
cp target/release/aura "${BUNDLE_DIR}/Contents/MacOS/"

# Copy resources
cp crates/aura/assets/Info.plist "${BUNDLE_DIR}/Contents/"
cp crates/aura/assets/AppIcon.icns "${BUNDLE_DIR}/Contents/Resources/"

echo "✓ Bundle created at ${BUNDLE_DIR}"
