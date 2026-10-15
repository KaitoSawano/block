#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"
cargo build --release
cp target/release/zenachain ./zenachain
cp target/release/zenachaind ./zenachaind
chmod +x ./zenachain
chmod +x ./zenachaind
echo "binary siap: ./zenachain dan ./zenachaind"
