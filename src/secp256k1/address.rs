// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Address derivation for ZenaChain using Base58Check.

use zenachain_crypto::sha3_256_hex; // For SHA3-256 hashing.

use crate::base58check; // For Base58Check encoding/decoding.
use crate::consensus::spec; // For network-specific prefixes.

use super::PUBLIC_KEY_SIZE; // For public key size validation.

/// Compute the address for a secp256k1 public key, using the address
/// prefix of the currently active network.
///
/// Format: Base58Check(prefix_byte + hash20).
/// This is similar to Bitcoin's Legacy (P2PKH) addresses, but uses SHA3-256
/// instead of SHA-256 for hashing.
pub fn address_from_public_key(public_key: &[u8]) -> Result<String, String> {
    if public_key.len() != PUBLIC_KEY_SIZE {
        // Ensure the public key is the correct size.
        return Err(format!(
            "public key must be {} bytes, got {}",
            PUBLIC_KEY_SIZE,
            public_key.len()
        ));
    }

    // Step 1: Compute the hash of the public key.
    let payload = [public_key, b"addr"].concat(); // Concatenate public key and "addr".
    let hash_hex = sha3_256_hex(&payload); // Compute SHA3-256.
    let hash_bytes = hex::decode(&hash_hex).expect("sha3 hex is always valid"); // Decode to bytes.

    // Step 2: Take the first 20 bytes (like RIPEMD-160 in Bitcoin).
    let hash20 = &hash_bytes[..20];

    // Step 3: Prepend the network prefix byte.
    let prefix = spec::current_network().address_prefix_byte(); // Get the prefix byte.
    let mut body = vec![prefix]; // Start with the prefix byte.
    body.extend_from_slice(hash20); // Append the 20-byte hash.

    // Step 4: Encode with Base58Check.
    Ok(base58check::encode(&body))
}

/// Verify that an address is valid (Base58Check with correct checksum and prefix).
pub fn verify_address(address: &str) -> bool {
    let payload = match base58check::decode(address) {
        Some(p) => p,
        None => return false, // Invalid Base58 or checksum.
    };

    // A valid payload must be exactly 21 bytes (1 prefix + 20 hash).
    if payload.len() != 21 {
        return false;
    }

    // Verify the prefix byte matches the active network.
    let expected_prefix = spec::current_network().address_prefix_byte();
    payload[0] == expected_prefix
}
