//! Parameter genesis konsensus ZenaChain.
//!
//! Nilai `EXPECTED_*` di bawah adalah hasil genesis yang sudah final.
//! Kalau ada parameter konsensus yang diubah, genesis hash akan
//! berbeda dari `EXPECTED_GENESIS_HASH`, dan node menolak start.
//!
//! Untuk memakai genesis baru, jalankan node dengan `.chain` kosong,
//! lalu salin nilai `hash`, `merkle_root`, `timestamp`, dan `nonce`
//! dari `getgenesis` ke konstanta di file ini.

pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
pub const GENESIS_TIMESTAMP: u128 = 1_792_012_800_000;
pub const GENESIS_OUTPUT_SCRIPT: &str =
    "OP_DUP OP_HASH160 ZenaChainGenesisOutput OP_EQUALVERIFY OP_CHECKSIG";

/// Hash genesis yang diharapkan. Kalau genesis yang dihitung berbeda,
/// node menolak start.
pub const EXPECTED_GENESIS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Merkle root genesis yang diharapkan.
pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Timestamp genesis yang diharapkan.
pub const EXPECTED_GENESIS_TIMESTAMP: u128 = GENESIS_TIMESTAMP;

/// Nonce genesis yang diharapkan.
pub const EXPECTED_GENESIS_NONCE: u64 = 0;

pub fn genesis_message() -> &'static str { GENESIS_MESSAGE }
pub fn genesis_secret() -> &'static str { GENESIS_SECRET }
pub fn genesis_timestamp() -> u128 { GENESIS_TIMESTAMP }
pub fn genesis_output_script() -> String { GENESIS_OUTPUT_SCRIPT.to_string() }
