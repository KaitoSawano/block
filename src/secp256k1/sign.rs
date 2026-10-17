// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use secp256k1::{Message, Secp256k1, SecretKey};

/// Tanda tangani hash 32-byte dengan secret key secp256k1.
pub fn sign(secret_key: &[u8], message_hash: &[u8]) -> Result<Vec<u8>, String> {
    if secret_key.len() != super::SECRET_KEY_SIZE {
        return Err(format!(
            "secret_key harus {} byte, dapat {}",
            super::SECRET_KEY_SIZE,
            secret_key.len()
        ));
    }
    if message_hash.len() != 32 {
        return Err(format!(
            "message_hash harus 32 byte, dapat {}",
            message_hash.len()
        ));
    }

    let secp = Secp256k1::new();
    let secret =
        SecretKey::from_slice(secret_key).map_err(|e| format!("secret key tidak valid: {}", e))?;
    let msg = Message::from_digest_slice(message_hash)
        .map_err(|e| format!("message tidak valid: {}", e))?;

    let sig = secp.sign_ecdsa(&msg, &secret);
    Ok(sig.serialize_der().to_vec())
}
