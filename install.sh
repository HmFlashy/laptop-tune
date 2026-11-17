#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-/usr/local}"
BIN_DIR="${PREFIX}/bin"
TARGET_BIN="target/release/laptop-tune"

if [[ "${1:-}" == "--uninstall" ]]; then
  echo "Removing ${BIN_DIR}/laptop-tune..."
  sudo rm -f "${BIN_DIR}/laptop-tune"
  echo "Uninstalled."
  exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "Error: cargo (Rust toolchain) is required to build from source." >&2
  echo "Install Rust via your package manager (e.g. 'sudo pacman -S rust' or 'sudo apt install cargo') or https://rustup.rs" >&2
  exit 1
fi

echo "Building laptop-tune (release)..."
cargo build --release

echo "Installing to ${BIN_DIR}/laptop-tune..."
sudo install -Dm755 "${TARGET_BIN}" "${BIN_DIR}/laptop-tune"

echo "Done! Run: laptop-tune status"
