// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Address derivation for ZenaChain.

use zenachain_crypto::sha3_256_hex; // For SHA3-256 hashing.

use crate::checksum; // For checksum computation.
use crate::consensus::rules; // For address length constants.
use crate::consensus::spec; // For network-specific prefixes.

use super::PUBLIC_KEY_SIZE; // For public key size validation.

/// Compute the address for a secp256k1 public key, using the address
/// prefix of the currently active network.
///
/// Format: `<prefix>` + 40 hex chars of SHA3-256(pubkey || "addr")
///         + 8 hex chars of checksum.
/// The checksum is the first 4 bytes of SHA3-256(SHA3-256(prefix + hex)).
pub fn address_from_public_key(public_key: &[u8]) -> Result<String, String> {
    if public_key.len() != PUBLIC_KEY_SIZE {
        // Ensure the public key is the correct size.
        return Err(format!(
            "public key must be {} bytes, got {}",
            PUBLIC_KEY_SIZE,
            public_key.len()
        ));
    }

    let payload = [public_key, b"addr"].concat(); // Concatenate public key and "addr".
    let hash = sha3_256_hex(&payload); // Compute SHA3-256.
    let prefix = spec::current_network().address_prefix(); // Get the network prefix.
    let body = format!("{}{}", prefix, &hash[..rules::ADDRESS_HEX_LEN]); // Prefix + 40 hex chars.

    // Compute the checksum for the body.
    let checksum = checksum::compute_checksum(&body);

    Ok(format!("{}{}", body, checksum)) // Return body + checksum.
}
