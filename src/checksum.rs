// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Address checksum computation and verification for ZenaChain.
//!
//! The checksum is the first 4 bytes (8 hex characters) of
//! SHA3-256(SHA3-256(prefix + hex_body)). This allows the wallet and
//! validation layer to detect typos in addresses before funds are sent.

use zenachain_crypto::sha3_256_hex; // For SHA3-256 hashing.

use crate::consensus::rules; // For address length constants.
use crate::consensus::spec; // For network-specific prefixes.

/// Compute the checksum for an address body (prefix + 40 hex chars).
/// Returns an 8-character lowercase hex string (4 bytes).
pub fn compute_checksum(body: &str) -> String {
    let first_hash = sha3_256_hex(body.as_bytes()); // First SHA3-256 of the body.
    let first_bytes = hex::decode(&first_hash).expect("sha3 hex is always valid"); // Decode to bytes.
    let second_hash = sha3_256_hex(&first_bytes); // Second SHA3-256.
    second_hash[..rules::ADDRESS_CHECKSUM_LEN].to_string() // Return the first 8 hex chars.
}

/// Verify the checksum of a full address (prefix + 40 hex + 8 checksum).
/// Returns `true` if the checksum is valid, `false` otherwise.
pub fn verify_checksum(address: &str) -> bool {
    let prefix = spec::current_network().address_prefix(); // Get the network prefix.
    if address.len() != rules::ADDRESS_TOTAL_LEN {
        // Ensure the address has the correct total length.
        return false;
    }
    if !address.starts_with(prefix) {
        // Ensure the address starts with the correct prefix.
        return false;
    }

    // Extract the body (prefix + 40 hex) and the expected checksum.
    let body = &address[..prefix.len() + rules::ADDRESS_HEX_LEN];
    let expected_checksum = &address[prefix.len() + rules::ADDRESS_HEX_LEN..];

    // Compute the checksum for the body.
    let computed_checksum = compute_checksum(body);

    // Compare.
    expected_checksum == computed_checksum
}
