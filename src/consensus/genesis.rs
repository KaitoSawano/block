// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Genesis consensus parameters for ZenaChain.
//!
//! The `EXPECTED_*` values below are the final genesis fingerprint. If
//! `Block::genesis()` computes a different hash or merkle root, the node
//! prints the new genesis values and exits. Copy the new values into this
//! file, rebuild, and run the node again.

pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
pub const GENESIS_TIMESTAMP: u128 = 1_792_012_800_000;
pub const GENESIS_OUTPUT_SCRIPT: &str =
    "OP_DUP OP_HASH160 ZenaChainGenesisOutput OP_EQUALVERIFY OP_CHECKSIG";

pub const EXPECTED_GENESIS_HASH: &str =
    "0000022074f04a57393f2cb3c0d7683e653da69b177f1bc3016401f151e36e62";

pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =
    "a0d24005775f3cd42a6f7d4a03f6d9d650ef814a2d3bc73833a57d1749c3cd6c";

pub const EXPECTED_GENESIS_TIMESTAMP: u128 = 1_792_012_800_000;

pub const EXPECTED_GENESIS_NONCE: u64 = 918211;

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
