//! Parameter genesis konsensus ZenaChain.
//!
//! Nilai `EXPECTED_*` di bawah adalah sidik jari genesis yang sudah final.
//! Kalau `Block::genesis()` menghitung genesis dengan hash atau merkle
//! root yang berbeda, `assert!` di `src/block.rs` akan gagal, dan node
//! tidak bisa dijalankan.
//!
//! Untuk memakai genesis baru, isi ulang konstanta di bawah dengan
//! nilai genesis yang baru.

pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
pub const GENESIS_TIMESTAMP: u128 = 1_792_012_800_000;
pub const GENESIS_OUTPUT_SCRIPT: &str =
    "OP_DUP OP_HASH160 ZenaChainGenesisOutput OP_EQUALVERIFY OP_CHECKSIG";

/// Hash genesis yang diharapkan.
pub const EXPECTED_GENESIS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Merkle root genesis yang diharapkan.
pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Timestamp genesis yang diharapkan.
pub const EXPECTED_GENESIS_TIMESTAMP: u128 = 1_792_012_800_000;

/// Nonce genesis yang diharapkan.
pub const EXPECTED_GENESIS_NONCE: u64 = 0;

pub fn genesis_message() -> &'static str { GENESIS_MESSAGE }
pub fn genesis_secret() -> &'static str { GENESIS_SECRET }
pub fn genesis_timestamp() -> u128 { GENESIS_TIMESTAMP }
pub fn genesis_output_script() -> String { GENESIS_OUTPUT_SCRIPT.to_string() }
