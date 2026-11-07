#!/usr/bin/env bash
set -e

REPO="hieudoanm/tracers"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

echo "Installing tracers..."

# Try to download prebuilt binary if available
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
if [ "$ARCH" = "x86_64" ]; then
  ARCH="x86_64"
elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
  ARCH="aarch64"
fi

BINARY_URL="https://github.com/${REPO}/releases/latest/download/tracers-${OS}-${ARCH}"
if command -v curl >/dev/null 2>&1; then
  if curl -fsSL "$BINARY_URL" -o /tmp/tracers 2>/dev/null; then
    chmod +x /tmp/tracers
    sudo mv /tmp/tracers "${INSTALL_DIR}/tracers" 2>/dev/null || mv /tmp/tracers "${INSTALL_DIR}/tracers"
    echo "tracers installed to ${INSTALL_DIR}/tracers"
    tracers --version 2>/dev/null || true
    exit 0
  fi
fi

# Fallback: build from source with cargo
if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust toolchain not found. Please install Rust from https://rustup.rs"
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
cd "$TMP_DIR"
git clone --depth 1 "https://github.com/${REPO}.git" tracers
cd tracers
cargo build --release
sudo cp target/release/tracers "${INSTALL_DIR}/tracers" 2>/dev/null || cp target/release/tracers "${INSTALL_DIR}/tracers"
echo "tracers installed to ${INSTALL_DIR}/tracers"
