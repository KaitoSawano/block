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
    "00000efb658a36715e611a97ce22343e5e1ccd3243157680eb998316648a439f";

/// The expected merkle root of the genesis block.
/// TODO: Update this value after running `cargo run --release --bin zenachain-genesis`.
pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =
    "a0d24005775f3cd42a6f7d4a03f6d9d650ef814a2d3bc73833a57d1749c3cd6c";

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
