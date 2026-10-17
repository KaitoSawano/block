#!/usr/bin/env bash
# Copyright (c) 2026 Aldianokto
# Copyright (c) 2026-present Zenachain Core developers
# Distributed under the MIT software license, see the accompanying
# file LICENSE or https://opensource.org/license/mit.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo not found. Install Rust from https://rustup.rs/"
    exit 1
fi

TARGET="${1:-}"

if [ -n "$TARGET" ]; then
    cargo build --release --locked --target "$TARGET" --bins
    BIN_DIR="target/$TARGET/release"
else
    cargo build --release --locked --bins
    BIN_DIR="target/release"
fi

BINARIES=(zenachain zenachaind zenachain-miner zenachain-genesis)

for bin in "${BINARIES[@]}"; do
    cp "$BIN_DIR/$bin" "./$bin"
    chmod +x "./$bin"
done

echo "Binaries ready:"
for bin in "${BINARIES[@]}"; do
    echo "  ./$bin"
done
