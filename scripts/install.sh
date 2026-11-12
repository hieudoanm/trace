#!/usr/bin/env bash
set -e

REPO="hieudoanm/tracers"
# Default to a user-writable directory; override with INSTALL_DIR=/path.
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

echo "Installing tracers to ${INSTALL_DIR}..."

install_binary() {
  local src="$1"
  mkdir -p "${INSTALL_DIR}"
  if [ -w "${INSTALL_DIR}" ]; then
    mv "${src}" "${INSTALL_DIR}/tracers"
  elif command -v sudo >/dev/null 2>&1; then
    sudo mv "${src}" "${INSTALL_DIR}/tracers"
  else
    echo "error: ${INSTALL_DIR} is not writable; set INSTALL_DIR to a writable directory" >&2
    exit 1
  fi
  chmod +x "${INSTALL_DIR}/tracers"
}

# Try to download a prebuilt binary if a release provides one.
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
if [ "$ARCH" = "x86_64" ]; then
  ARCH="x86_64"
elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
  ARCH="aarch64"
fi

case "$OS" in
  linux)  TRIPLE="${ARCH}-unknown-linux-gnu" ;;
  darwin) TRIPLE="${ARCH}-apple-darwin" ;;
  *)      TRIPLE="x86_64-pc-windows-msvc" ;;
esac

BINARY_URL="https://github.com/${REPO}/releases/latest/download/tracers-${TRIPLE}"
[[ "$OS" != "linux" && "$OS" != "darwin" ]] && BINARY_URL="${BINARY_URL}.exe"
TMP_BIN="$(mktemp -t tracers)"
if command -v curl >/dev/null 2>&1 \
  && curl -fsSL "$BINARY_URL" -o "${TMP_BIN}" 2>/dev/null \
  && chmod +x "${TMP_BIN}" \
  && "${TMP_BIN}" --version >/dev/null 2>&1; then
  install_binary "${TMP_BIN}"
  echo "tracers installed to ${INSTALL_DIR}/tracers"
  "${INSTALL_DIR}/tracers" --version || true
  exit 0
fi
rm -f "${TMP_BIN}"

# Fallback: build from source with cargo.
if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust toolchain not found. Please install Rust from https://rustup.rs"
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT
cd "${TMP_DIR}"
git clone --depth 1 "https://github.com/${REPO}.git" tracers
cd tracers
cargo build --release
install_binary target/release/tracers
echo "tracers installed to ${INSTALL_DIR}/tracers"
