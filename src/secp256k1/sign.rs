// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! ECDSA signing for ZenaChain.

use secp256k1::{Message, Secp256k1, SecretKey};

/// Sign a 32-byte message hash with a secp256k1 secret key.
/// The function returns a DER-encoded signature.
pub fn sign(secret_key: &[u8], message_hash: &[u8]) -> Result<Vec<u8>, String> {
    if secret_key.len() != super::SECRET_KEY_SIZE {
        return Err(format!(
            "secret key must be {} bytes, got {}",
            super::SECRET_KEY_SIZE,
            secret_key.len()
        ));
    }
    if message_hash.len() != 32 {
        return Err(format!(
            "message hash must be 32 bytes, got {}",
            message_hash.len()
        ));
    }

    let secp = Secp256k1::new();
    let secret =
        SecretKey::from_slice(secret_key).map_err(|e| format!("invalid secret key: {}", e))?;
    let msg =
        Message::from_digest_slice(message_hash).map_err(|e| format!("invalid message: {}", e))?;

    let sig = secp.sign_ecdsa(&msg, &secret);
    Ok(sig.serialize_der().to_vec())
}
