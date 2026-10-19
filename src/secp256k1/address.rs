// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Address derivation for ZenaChain.

use zenachain_crypto::sha3_256_hex;

use crate::consensus::spec;

use super::PUBLIC_KEY_SIZE;

/// Compute the address for a secp256k1 public key, using the address
/// prefix of the currently active network.
///
/// Format: `<prefix>` followed by the first 40 hexadecimal characters
/// of SHA3-256(compressed_pubkey || "addr").
pub fn address_from_public_key(public_key: &[u8]) -> Result<String, String> {
    if public_key.len() != PUBLIC_KEY_SIZE {
        return Err(format!(
            "public key must be {} bytes, got {}",
            PUBLIC_KEY_SIZE,
            public_key.len()
        ));
    }

    let payload = [public_key, b"addr"].concat();
    let hash = sha3_256_hex(&payload);
    let prefix = spec::current_network().address_prefix();
    Ok(format!("{}{}", prefix, &hash[..40]))
}
