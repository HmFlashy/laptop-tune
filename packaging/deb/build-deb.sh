#!/usr/bin/env bash
set -euo pipefail

VERSION="1.5.2"
if command -v dpkg >/dev/null 2>&1; then
  ARCH="$(dpkg --print-architecture)"
else
  case "$(uname -m)" in
    x86_64) ARCH="amd64" ;;
    aarch64) ARCH="arm64" ;;
    *) ARCH="$(uname -m)" ;;
  esac
fi

PKG_NAME="laptop-tune_${VERSION}_${ARCH}"
BUILD_DIR="target/deb/${PKG_NAME}"
OUT_DEB="target/deb/${PKG_NAME}.deb"

cargo build --release

rm -rf "${BUILD_DIR}" "${OUT_DEB}"
mkdir -p "${BUILD_DIR}/DEBIAN" "${BUILD_DIR}/usr/bin" "${BUILD_DIR}/usr/share/doc/laptop-tune"

install -Dm755 target/release/laptop-tune "${BUILD_DIR}/usr/bin/laptop-tune"
install -Dm644 packaging/laptop-tune.desktop "${BUILD_DIR}/usr/share/applications/laptop-tune.desktop"
install -Dm644 packaging/laptop-tune.svg "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/laptop-tune.svg"
install -Dm644 packaging/org.laptop-tune.policy "${BUILD_DIR}/usr/share/polkit-1/actions/org.laptop-tune.policy"
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

if command -v dpkg-deb >/dev/null 2>&1; then
  dpkg-deb --build "${BUILD_DIR}" "${OUT_DEB}"
else
  TMP_AR="$(mktemp -d)"
  trap 'rm -rf "${TMP_AR}"' EXIT
  printf "2.0\n" > "${TMP_AR}/debian-binary"
  tar --owner=0 --group=0 -C "${BUILD_DIR}/DEBIAN" -czf "${TMP_AR}/control.tar.gz" .
  tar --owner=0 --group=0 --exclude='./DEBIAN' -C "${BUILD_DIR}" -czf "${TMP_AR}/data.tar.gz" .
  (cd "${TMP_AR}" && ar rcs "${OLDPWD}/${OUT_DEB}" debian-binary control.tar.gz data.tar.gz)
fi

echo "Built: ${OUT_DEB}"
