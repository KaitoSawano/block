use ml_dsa::{MlDsa65, Signature, Verifier, VerifyingKey};

use super::PUBLIC_KEY_SIZE;

/// Verifikasi signature ML-DSA-65.
pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
    if public_key.len() != PUBLIC_KEY_SIZE {
        return false;
    }
    if signature.len() != super::SIGNATURE_SIZE {
        return false;
    }

    let vk = match VerifyingKey::<MlDsa65>::decode(public_key) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let sig = match Signature::<MlDsa65>::try_from(signature) {
        Ok(s) => s,
        Err(_) => return false,
    };

    vk.verify(message, &sig).is_ok()
}
