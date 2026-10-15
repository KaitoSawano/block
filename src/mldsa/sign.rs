use dcrypt::api::Signature;
use dcrypt::sign::Dilithium3;

use super::SIGNATURE_SIZE;

/// Tanda tangani pesan dengan secret key ML-DSA-65.
pub fn sign(secret_key: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
    if secret_key.len() != super::SECRET_KEY_SIZE {
        return Err(format!(
            "secret_key harus {} byte, dapat {}",
            super::SECRET_KEY_SIZE,
            secret_key.len()
        ));
    }

    let signature = Dilithium3::sign(message, secret_key)
        .map_err(|e| format!("sign gagal: {:?}", e))?;

    if signature.len() != SIGNATURE_SIZE {
        return Err(format!(
            "signature size tidak sesuai: {}",
            signature.len()
        ));
    }

    Ok(signature)
}
