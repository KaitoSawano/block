//! Parameter genesis konsensus ZenaChain.
//!
//! Nilai `EXPECTED_*` di bawah adalah sidik jari genesis yang sudah final.
//! Kalau `Block::genesis()` menghitung genesis dengan hash atau merkle
//! root yang berbeda, `zenachaind` akan mencetak nilai genesis yang baru,
//! lalu keluar. Anda salin ke file ini, build ulang, jalankan lagi.

pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
pub const GENESIS_TIMESTAMP: u128 = 1_792_012_800_000;
pub const GENESIS_OUTPUT_SCRIPT: &str =
    "OP_DUP OP_HASH160 ZenaChainGenesisOutput OP_EQUALVERIFY OP_CHECKSIG";

pub const EXPECTED_GENESIS_HASH: &str =
    "00000c99854f0f3ffca1b12ad07a91c9171447136c3b50dbb012830f116ba303";

pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =
    "a0d24005775f3cd42a6f7d4a03f6d9d650ef814a2d3bc73833a57d1749c3cd6c";

pub const EXPECTED_GENESIS_TIMESTAMP: u128 = 1_792_012_800_000;

pub const EXPECTED_GENESIS_NONCE: u64 = 621179;

pub fn genesis_message() -> &'static str {
    GENESIS_MESSAGE
}
pub fn genesis_secret() -> &'static str {
    GENESIS_SECRET
}
pub fn genesis_timestamp() -> u128 {
    GENESIS_TIMESTAMP
}
pub fn genesis_output_script() -> String {
    GENESIS_OUTPUT_SCRIPT.to_string()
}
