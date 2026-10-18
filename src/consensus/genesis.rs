// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Genesis consensus parameters for ZenaChain.

pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
pub const GENESIS_OUTPUT_SCRIPT: &str =
    "OP_DUP OP_HASH160 ZenaChainGenesisOutput OP_EQUALVERIFY OP_CHECKSIG";

pub const GENESIS_TIMESTAMP: u128 = 1792303198445;

pub const EXPECTED_GENESIS_HASH: &str =
    "000005d589f2e9a9fa09571fb2a978e5bf998f2f8104c570ff66212c7083620a";

pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =
    "a0d24005775f3cd42a6f7d4a03f6d9d650ef814a2d3bc73833a57d1749c3cd6c";

pub const EXPECTED_GENESIS_TIMESTAMP: u128 = 1792303198445;

pub const EXPECTED_GENESIS_NONCE: u64 = 72180;

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
