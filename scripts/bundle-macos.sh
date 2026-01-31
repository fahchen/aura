#!/bin/bash
set -e

APP_NAME="Aura"
BUNDLE_DIR="target/release/${APP_NAME}.app"
HOOK_CRATE_DIR="crates/aura-claude-code-hook"
HOOK_BIN="target/release/aura-claude-code-hook"
INCLUDE_HOOK=false

if [ -d "${HOOK_CRATE_DIR}" ]; then
  INCLUDE_HOOK=true
fi

# Build release binaries
if [ "${INCLUDE_HOOK}" = true ]; then
  cargo build --release -p aura-daemon -p aura-claude-code-hook
else
  cargo build --release -p aura-daemon
fi

# Create bundle structure
mkdir -p "${BUNDLE_DIR}/Contents/MacOS"
mkdir -p "${BUNDLE_DIR}/Contents/Resources"

# Copy daemon binary
cp target/release/aura "${BUNDLE_DIR}/Contents/MacOS/"

# Copy hook binary (for CLI installation)
if [ "${INCLUDE_HOOK}" = true ] && [ -f "${HOOK_BIN}" ]; then
  cp "${HOOK_BIN}" "${BUNDLE_DIR}/Contents/MacOS/"
else
  echo "i Skipping hook binary: ${HOOK_CRATE_DIR} not found"
fi

# Copy resources
cp crates/aura-daemon/assets/Info.plist "${BUNDLE_DIR}/Contents/"
cp crates/aura-daemon/assets/AppIcon.icns "${BUNDLE_DIR}/Contents/Resources/"

echo "✓ Bundle created at ${BUNDLE_DIR}"
