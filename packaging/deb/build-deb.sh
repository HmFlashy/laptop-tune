#!/usr/bin/env bash
set -euo pipefail

VERSION="1.0.0"
ARCH="$(dpkg --print-architecture 2>/dev/null || echo amd64)"
PKG_NAME="laptop-tune_${VERSION}_${ARCH}"
BUILD_DIR="target/deb/${PKG_NAME}"

cargo build --release

rm -rf "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/DEBIAN" "${BUILD_DIR}/usr/bin" "${BUILD_DIR}/usr/share/doc/laptop-tune"

install -Dm755 target/release/laptop-tune "${BUILD_DIR}/usr/bin/laptop-tune"
install -Dm644 README.md "${BUILD_DIR}/usr/share/doc/laptop-tune/README.md"
install -Dm644 LICENSE "${BUILD_DIR}/usr/share/doc/laptop-tune/copyright"

cat > "${BUILD_DIR}/DEBIAN/control" <<EOF
Package: laptop-tune
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: Hugo Maitre <hugo.maitre69@gmail.com>
Description: Unified Linux laptop power, GPU runtime PM, and battery auditor
 Audits battery power draw and charge thresholds, hybrid GPU runtime PM,
 sleep states, and thermals, and applies CPU/PCIe profiles across Intel, AMD, and NVIDIA hardware.
EOF

dpkg-deb --build "${BUILD_DIR}" "target/deb/${PKG_NAME}.deb"
echo "Built: target/deb/${PKG_NAME}.deb"
