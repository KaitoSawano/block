// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use crate::utils::sha3_256_hex;

use super::PUBLIC_KEY_SIZE;

/// Hitung alamat ZENA dari public key secp256k1.
///
/// Format: "ZENA" + 40 hex pertama dari
/// SHA3-256(compressed_pubkey || "addr")
pub fn address_from_public_key(public_key: &[u8]) -> Result<String, String> {
    if public_key.len() != PUBLIC_KEY_SIZE {
        return Err(format!(
            "public_key harus {} byte, dapat {}",
            PUBLIC_KEY_SIZE,
            public_key.len()
        ));
    }

    let payload = [public_key, b"addr"].concat();
    let hash = sha3_256_hex(&payload);
    Ok(format!("ZENA{}", &hash[..40]))
}
