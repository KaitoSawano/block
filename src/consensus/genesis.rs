//! Parameter genesis konsensus ZenaChain.

pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
pub const GENESIS_TIMESTAMP: u128 = 1_792_012_800_000;
pub const GENESIS_OUTPUT_SCRIPT: &str =
    "OP_DUP OP_HASH160 ZenaChainGenesisOutput OP_EQUALVERIFY OP_CHECKSIG";

pub fn genesis_message() -> &'static str { GENESIS_MESSAGE }
pub fn genesis_secret() -> &'static str { GENESIS_SECRET }
pub fn genesis_timestamp() -> u128 { GENESIS_TIMESTAMP }
pub fn genesis_output_script() -> String { GENESIS_OUTPUT_SCRIPT.to_string() }
