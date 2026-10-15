use dcrypt::api::Signature;
use dcrypt::sign::Dilithium3;
use rand::rngs::OsRng;

use super::{PUBLIC_KEY_SIZE, SECRET_KEY_SIZE};

/// Pasangan kunci ML-DSA-65.
#[derive(Debug, Clone)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

impl KeyPair {
    /// Generate key pair baru dengan sumber acak dari sistem operasi.
    pub fn generate() -> Result<Self, String> {
        let (pk, sk) = Dilithium3::keypair(&mut OsRng)
            .map_err(|e| format!("keypair gagal: {:?}", e))?;
        Ok(KeyPair {
            public_key: pk,
            secret_key: sk,
        })
    }

    /// Muat key pair dari bytes yang tersimpan.
    pub fn from_bytes(public_key: &[u8], secret_key: &[u8]) -> Result<Self, String> {
        if public_key.len() != PUBLIC_KEY_SIZE {
            return Err(format!(
                "public_key harus {} byte, dapat {}",
                PUBLIC_KEY_SIZE,
                public_key.len()
            ));
        }
        if secret_key.len() != SECRET_KEY_SIZE {
            return Err(format!(
                "secret_key harus {} byte, dapat {}",
                SECRET_KEY_SIZE,
                secret_key.len()
            ));
        }
        Ok(KeyPair {
            public_key: public_key.to_vec(),
            secret_key: secret_key.to_vec(),
        })
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(&self.public_key)
    }

    pub fn secret_key_hex(&self) -> String {
        hex::encode(&self.secret_key)
    }
}
