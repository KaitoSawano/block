// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Address derivation for ZenaChain.

use zenachain_crypto::sha3_256_hex;

use super::PUBLIC_KEY_SIZE;

/// Compute the ZENA address for a secp256k1 public key.
///
/// Format: "ZENA" followed by the first 40 hexadecimal characters of
/// SHA3-256(compressed_pubkey || "addr").
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
    Ok(format!("ZENA{}", &hash[..40]))
}
