// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use zenachain::block::Block;

fn main() {
    let b = Block::genesis();

    println!();
    println!("============================================================");
    println!("  GENESIS ZENACHAIN (salin ke src/consensus/genesis.rs)");
    println!("============================================================");
    println!("  pub const EXPECTED_GENESIS_HASH: &str =");
    println!("      \"{}\";", b.hash);
    println!("  pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =");
    println!("      \"{}\";", b.merkle_root);
    println!(
        "  pub const EXPECTED_GENESIS_TIMESTAMP: u128 = {};",
        b.timestamp
    );
    println!("  pub const EXPECTED_GENESIS_NONCE: u64 = {};", b.nonce);
    println!("============================================================");
    println!();
}
