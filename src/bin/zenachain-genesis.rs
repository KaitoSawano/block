//! Binary sekali pakai untuk menghitung genesis ZenaChain.
//!
//! Jalankan `./zenachain-genesis` setiap kali parameter konsensus
//! diubah. Cetak nilai hash, merkle_root, timestamp, dan nonce
//! genesis yang baru. Salin keempat nilai ke
//! `src/consensus/genesis.rs`.

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
