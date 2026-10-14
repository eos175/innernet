#!/usr/bin/env bash
set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"
cd "$ROOT_DIR"

echo "=== 1. Building innernet & innernetd release binaries ==="
cargo build --release --bin innernet --bin innernetd

echo "=== 2. Packaging innernet RPM (CLI + innernetd + systemd) ==="
cd "$ROOT_DIR/client"
cargo rpm build

echo "=== 3. Packaging innernet-desktop RPM ==="
cd "$ROOT_DIR/desktop"
pnpm install
pnpm tauri build --bundles rpm

echo "=== 4. Collecting RPM packages ==="
mkdir -p "$ROOT_DIR/target/rpm"
cp -r "$ROOT_DIR"/target/release/rpmbuild/RPMS/*/*.rpm "$ROOT_DIR/target/rpm/" 2>/dev/null || true
cp -r "$ROOT_DIR"/desktop/src-tauri/target/release/bundle/rpm/*.rpm "$ROOT_DIR/target/rpm/" 2>/dev/null || true

echo
echo "=== Successfully built RPMs in target/rpm/ ==="
ls -lh "$ROOT_DIR/target/rpm/"*.rpm
