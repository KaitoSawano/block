// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Genesis consensus parameters for ZenaChain.
//!
//! Each network has its own genesis block. The mainnet genesis is
//! locked forever; the testnet genesis may be regenerated when the
//! consensus rules change.

use crate::consensus::spec::{self, Network};

// ----------------------------------------------------------------------
// Shared genesis messages (same on all networks)
// ----------------------------------------------------------------------

pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
pub const GENESIS_OUTPUT_SCRIPT: &str =
    "OP_DUP OP_HASH160 ZenaChainGenesisOutput OP_EQUALVERIFY OP_CHECKSIG";

// ----------------------------------------------------------------------
// Mainnet genesis
// ----------------------------------------------------------------------

pub const GENESIS_TIMESTAMP: u128 = 1792303198445;

pub const EXPECTED_GENESIS_HASH: &str =
    "00000efb658a36715e611a97ce22343e5e1ccd3243157680eb998316648a439f";

pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =
    "a0d24005775f3cd42a6f7d4a03f6d9d650ef814a2d3bc73833a57d1749c3cd6c";

pub const EXPECTED_GENESIS_TIMESTAMP: u128 = 1792303198445;

pub const EXPECTED_GENESIS_NONCE: u64 = 420282;

// ----------------------------------------------------------------------
// Testnet genesis
// ----------------------------------------------------------------------
//
// The testnet genesis is mined at build time by the `zenachain-genesis`
// binary with the `-testnet` flag. The resulting hash, merkle root,
// timestamp, and nonce must be pasted into this module.

pub const TESTNET_GENESIS_TIMESTAMP: u128 = 1792303198446;

pub const TESTNET_EXPECTED_GENESIS_HASH: &str = "";

pub const TESTNET_EXPECTED_GENESIS_MERKLE_ROOT: &str = "";

pub const TESTNET_EXPECTED_GENESIS_TIMESTAMP: u128 = 1792303198446;

pub const TESTNET_EXPECTED_GENESIS_NONCE: u64 = 0;

// ----------------------------------------------------------------------
// Network-aware accessors
// ----------------------------------------------------------------------

pub fn genesis_message() -> &'static str {
    GENESIS_MESSAGE
}

pub fn genesis_secret() -> &'static str {
    GENESIS_SECRET
}

pub fn genesis_timestamp() -> u128 {
    spec::current_network().genesis_timestamp()
}

pub fn genesis_output_script() -> String {
    GENESIS_OUTPUT_SCRIPT.to_string()
}

/// Return the expected genesis hash for the given network.
pub fn expected_genesis_hash_for(network: Network) -> &'static str {
    match network {
        Network::Main => EXPECTED_GENESIS_HASH,
        Network::Test => TESTNET_EXPECTED_GENESIS_HASH,
    }
}

/// Return the expected genesis merkle root for the given network.
pub fn expected_genesis_merkle_root_for(network: Network) -> &'static str {
    match network {
        Network::Main => EXPECTED_GENESIS_MERKLE_ROOT,
        Network::Test => TESTNET_EXPECTED_GENESIS_MERKLE_ROOT,
    }
}

/// Return the expected genesis timestamp for the given network.
pub fn expected_genesis_timestamp_for(network: Network) -> u128 {
    match network {
        Network::Main => EXPECTED_GENESIS_TIMESTAMP,
        Network::Test => TESTNET_EXPECTED_GENESIS_TIMESTAMP,
    }
}

/// Return the expected genesis nonce for the given network.
pub fn expected_genesis_nonce_for(network: Network) -> u64 {
    match network {
        Network::Main => EXPECTED_GENESIS_NONCE,
        Network::Test => TESTNET_EXPECTED_GENESIS_NONCE,
    }
}

// Legacy accessors (return mainnet values, kept for compatibility).

pub fn expected_genesis_hash() -> &'static str {
    EXPECTED_GENESIS_HASH
}

pub fn expected_genesis_merkle_root() -> &'static str {
    EXPECTED_GENESIS_MERKLE_ROOT
}

pub fn expected_genesis_nonce() -> u64 {
    EXPECTED_GENESIS_NONCE
}
