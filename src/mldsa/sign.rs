use ml_dsa::{MlDsa65, Signer, SigningKey};

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

    let sk = SigningKey::<MlDsa65>::decode(secret_key)
        .map_err(|e| format!("secret key tidak valid: {:?}", e))?;

    let signature = sk.sign(message);
    let sig_bytes = signature.encode().to_vec();

    if sig_bytes.len() != SIGNATURE_SIZE {
        return Err(format!(
            "signature size tidak sesuai: {}",
            sig_bytes.len()
        ));
    }

    Ok(sig_bytes)
}
