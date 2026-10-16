//! Parameter genesis konsensus ZenaChain.
//!
//! Nilai `EXPECTED_*` di bawah adalah sidik jari genesis yang sudah final.
//! Selama nilai ini masih nol, `zenachaind` akan menolak start dan
//! mencetak pesan panic. Anda harus menghitung genesis baru terlebih
//! dahulu, lalu menyalin hash, merkle_root, timestamp, dan nonce-nya
//! ke konstanta di file ini.

pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
pub const GENESIS_TIMESTAMP: u128 = 1_792_012_800_000;
pub const GENESIS_OUTPUT_SCRIPT: &str =
    "OP_DUP OP_HASH160 ZenaChainGenesisOutput OP_EQUALVERIFY OP_CHECKSIG";

/// Hash genesis yang diharapkan.
/// WAJIB diisi dengan hash genesis yang sah sebelum node dijalankan.
pub const EXPECTED_GENESIS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Merkle root genesis yang diharapkan.
/// WAJIB diisi dengan merkle root genesis yang sah.
pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Timestamp genesis yang diharapkan.
/// WAJIB diisi dengan timestamp genesis yang sah.
pub const EXPECTED_GENESIS_TIMESTAMP: u128 = 0;

/// Nonce genesis yang diharapkan.
/// WAJIB diisi dengan nonce genesis yang sah.
pub const EXPECTED_GENESIS_NONCE: u64 = 0;

pub fn genesis_message() -> &'static str { GENESIS_MESSAGE }
pub fn genesis_secret() -> &'static str { GENESIS_SECRET }
pub fn genesis_timestamp() -> u128 { GENESIS_TIMESTAMP }
pub fn genesis_output_script() -> String { GENESIS_OUTPUT_SCRIPT.to_string() }

/// Cek apakah konstanta `EXPECTED_GENESIS_*` sudah diisi.
/// Kalau masih nol, artinya belum diisi.
pub fn expected_filled() -> bool {
    EXPECTED_GENESIS_HASH
        != "0000000000000000000000000000000000000000000000000000000000000000"
        && EXPECTED_GENESIS_MERKLE_ROOT
            != "0000000000000000000000000000000000000000000000000000000000000000"
        && EXPECTED_GENESIS_TIMESTAMP != 0
        && EXPECTED_GENESIS_NONCE != 0
}
