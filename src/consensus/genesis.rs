// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Genesis consensus parameters for ZenaChain.

pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
pub const GENESIS_OUTPUT_SCRIPT: &str =
    "OP_DUP OP_HASH160 ZenaChainGenesisOutput OP_EQUALVERIFY OP_CHECKSIG";

/// The expected hash of the genesis block.
pub const EXPECTED_GENESIS_HASH: &str =
    "000007c50a27e3992b8b089f484532806b7536b25ed5b24fdd50621e43440f53";

/// The expected merkle root of the genesis block.
pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =
    "d2b3a95f56d04cf97a8ff85f52d6f8d88928905255b756aa87d44e9b1e1a0d9a";

/// The expected timestamp of the genesis block.
pub const EXPECTED_GENESIS_TIMESTAMP: u128 = 1792012800000;

/// The expected nonce of the genesis block.
pub const EXPECTED_GENESIS_NONCE: u64 = 1075998;

pub fn genesis_message() -> &'static str {
    GENESIS_MESSAGE
}
pub fn genesis_secret() -> &'static str {
    GENESIS_SECRET
}
pub fn genesis_output_script() -> String {
    GENESIS_OUTPUT_SCRIPT.to_string()
}
