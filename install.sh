#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-/usr/local}"
BIN_DIR="${PREFIX}/bin"
APP_DIR="${PREFIX}/share/applications"
ICON_DIR="${PREFIX}/share/icons/hicolor/scalable/apps"
POLKIT_DIR="/usr/share/polkit-1/actions"
TARGET_BIN="target/release/laptop-tune"

if [[ "${1:-}" == "--uninstall" ]]; then
  echo "Removing ${BIN_DIR}/laptop-tune, launcher, icon, udev/modprobe/env rules, and polkit policy..."
  sudo rm -f "${BIN_DIR}/laptop-tune" "${APP_DIR}/laptop-tune.desktop" "${ICON_DIR}/laptop-tune.svg" "${POLKIT_DIR}/org.laptop-tune.policy" /etc/tlp.d/99-laptop-tune-override.conf /etc/udev/rules.d/90-laptop-tune-gpu.rules /etc/modprobe.d/90-laptop-tune-gpu.conf /etc/environment.d/90-laptop-tune-igpu.conf
  sudo udevadm control --reload-rules 2>/dev/null || true
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
sudo install -Dm644 "packaging/laptop-tune.desktop" "${APP_DIR}/laptop-tune.desktop"
sudo install -Dm644 "packaging/laptop-tune.svg" "${ICON_DIR}/laptop-tune.svg"
sudo install -Dm644 "packaging/org.laptop-tune.policy" "${POLKIT_DIR}/org.laptop-tune.policy"

if [[ -d /etc/tlp.d ]]; then
  sudo install -Dm644 "packaging/99-laptop-tune-override.conf" "/etc/tlp.d/99-laptop-tune-override.conf"
fi

sudo "${BIN_DIR}/laptop-tune" setup-gpu
sudo udevadm control --reload-rules 2>/dev/null || true
sudo udevadm trigger --subsystem-match=pci --subsystem-match=drm 2>/dev/null || true

pkill -f "laptop-tune gui" 2>/dev/null || true

echo "Done! Run: laptop-tune status (or launch 'Laptop Tune' from your app menu)"
