use dcrypt::api::Signature;
use dcrypt::sign::Dilithium3;

use super::PUBLIC_KEY_SIZE;

/// Verifikasi signature ML-DSA-65.
pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
    if public_key.len() != PUBLIC_KEY_SIZE {
        return false;
    }
    if signature.len() != super::SIGNATURE_SIZE {
        return false;
    }

    Dilithium3::verify(message, signature, public_key).is_ok()
}
