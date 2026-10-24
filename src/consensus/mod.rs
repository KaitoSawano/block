// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Consensus module for ZenaChain.

pub mod genesis; // Genesis block parameters.
pub mod pow; // Proof-of-work parameters.
pub mod rules; // Consensus validation rules.
pub mod spec; // Economic consensus parameters.

use crate::consensus::spec::Network; // Network selector.
use zenachain_crypto::sha3_256_hex; // For hashing.

/// Compute the consensus fingerprint for the given network.
pub fn consensus_fingerprint_for(network: Network) -> String {
    let payload = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        network.name(),
        spec::HASH_ALGO,
        spec::SIGNATURE_ALGO,
        spec::DECIMALS,
        spec::SATOSHI,
        spec::MAX_SUPPLY_ZENAC,
        spec::MAX_SUPPLY,
        spec::INITIAL_REWARD_ZENAC,
        spec::INITIAL_REWARD,
        spec::SUBSIDY_HALVING_INTERVAL,
        spec::MAX_HALVINGS,
        spec::POW_TARGET_SPACING_SECS,
        spec::POW_TARGET_TIMESPAN_SECS,
        spec::MINER_CONFIRMATION_WINDOW,
        spec::DIFFICULTY_ADJUST_INTERVAL,
        genesis::GENESIS_MESSAGE,
        genesis::GENESIS_SECRET,
        network.genesis_timestamp(),
        genesis::GENESIS_OUTPUT_SCRIPT,
        network.genesis_nbits(),
        pow::pow_limit(),
        rules::COINBASE_MATURITY,
        network.address_prefix_byte(), // <-- Changed from address_prefix().
        rules::MAX_TX_PER_BLOCK,
        rules::MAX_NOTE_LEN,
        rules::BLOCK_VERSION,
        rules::ZERO_HASH,
        genesis::expected_genesis_hash_for(network),
    );
    sha3_256_hex(payload.as_bytes())
}

/// Compute the consensus fingerprint for the currently active network.
pub fn consensus_fingerprint() -> String {
    consensus_fingerprint_for(spec::current_network())
}

/// Compute the parameter fingerprint for the given network.
pub fn parameter_fingerprint_for(network: Network) -> String {
    let payload = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        network.name(),
        spec::HASH_ALGO,
        spec::SIGNATURE_ALGO,
        spec::DECIMALS,
        spec::SATOSHI,
        spec::MAX_SUPPLY_ZENAC,
        spec::MAX_SUPPLY,
        spec::INITIAL_REWARD_ZENAC,
        spec::INITIAL_REWARD,
        spec::SUBSIDY_HALVING_INTERVAL,
        spec::MAX_HALVINGS,
        spec::POW_TARGET_SPACING_SECS,
        spec::POW_TARGET_TIMESPAN_SECS,
        spec::MINER_CONFIRMATION_WINDOW,
        spec::DIFFICULTY_ADJUST_INTERVAL,
        genesis::GENESIS_MESSAGE,
        genesis::GENESIS_SECRET,
        network.genesis_timestamp(),
        genesis::GENESIS_OUTPUT_SCRIPT,
        network.genesis_nbits(),
        pow::pow_limit(),
        rules::COINBASE_MATURITY,
        network.address_prefix_byte(), // <-- Changed from address_prefix().
        rules::MAX_TX_PER_BLOCK,
        rules::MAX_NOTE_LEN,
        rules::BLOCK_VERSION,
        rules::ZERO_HASH,
        genesis::expected_genesis_merkle_root_for(network),
    );
    sha3_256_hex(payload.as_bytes())
}

/// Compute the parameter fingerprint for the currently active network.
pub fn parameter_fingerprint() -> String {
    parameter_fingerprint_for(spec::current_network())
}

/// Compute the genesis coinbase transaction identifier for the given network.
pub fn genesis_coinbase_txid_for(network: Network) -> String {
    let fp = parameter_fingerprint_for(network);
    let payload = format!(
        "genesis|{}|{}|{}",
        fp,
        network.genesis_timestamp(),
        network.genesis_nbits()
    );
    sha3_256_hex(payload.as_bytes())
}

/// Compute the genesis coinbase transaction identifier for the currently active network.
pub fn genesis_coinbase_txid() -> String {
    genesis_coinbase_txid_for(spec::current_network())
}

// Re-export consensus functions for convenience.
pub use genesis::genesis_message;
pub use genesis::genesis_output_script;
pub use genesis::genesis_secret;
pub use genesis::genesis_timestamp;
pub use pow::genesis_nbits;
pub use pow::pow_limit;
pub use rules::max_tx_per_block;
pub use spec::coin_name;
pub use spec::decimals;
pub use spec::difficulty_adjust_interval;
pub use spec::hash_algo;
pub use spec::initial_reward;
pub use spec::initial_reward_zenac;
pub use spec::max_halvings;
pub use spec::max_supply;
pub use spec::max_supply_zenac;
pub use spec::miner_confirmation_window;
pub use spec::pow_target_spacing_secs;
pub use spec::pow_target_timespan_secs;
pub use spec::protocol_version;
pub use spec::satoshi;
pub use spec::signature_algo;
pub use spec::subsidy_halving_interval;
pub use spec::symbol;
pub use spec::version;
