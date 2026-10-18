// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Genesis consensus parameters for ZenaChain.

pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
pub const GENESIS_OUTPUT_SCRIPT: &str =
    "OP_DUP OP_HASH160 ZenaChainGenesisOutput OP_EQUALVERIFY OP_CHECKSIG";

/// The timestamp of the genesis block. This is the exact time the block was mined.
/// It is locked forever and must not change.
pub const GENESIS_TIMESTAMP: u128 = 1792303198445;

/// The expected hash of the genesis block.
/// TODO: Update this value after running `cargo run --release --bin zenachain-genesis`.
pub const EXPECTED_GENESIS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// The expected merkle root of the genesis block.
/// TODO: Update this value after running `cargo run --release --bin zenachain-genesis`.
pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// The expected timestamp of the genesis block (must match GENESIS_TIMESTAMP).
pub const EXPECTED_GENESIS_TIMESTAMP: u128 = 1792303198445;

/// The expected nonce of the genesis block.
/// TODO: Update this value after running `cargo run --release --bin zenachain-genesis`.
pub const EXPECTED_GENESIS_NONCE: u64 = 0;

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
