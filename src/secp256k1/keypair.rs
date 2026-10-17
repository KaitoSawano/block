// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use rand::rngs::OsRng;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

use super::{PUBLIC_KEY_SIZE, SECRET_KEY_SIZE};

/// Pasangan kunci secp256k1.
#[derive(Debug, Clone)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

impl KeyPair {
    pub fn generate() -> Result<Self, String> {
        let secp = Secp256k1::new();
        let (secret, public) = secp.generate_keypair(&mut OsRng);

        Ok(KeyPair {
            public_key: public.serialize().to_vec(),
            secret_key: secret.secret_bytes().to_vec(),
        })
    }

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

    pub fn from_secret(secret_bytes: &[u8]) -> Result<Self, String> {
        let secp = Secp256k1::new();
        let secret = SecretKey::from_slice(secret_bytes)
            .map_err(|e| format!("secret key tidak valid: {}", e))?;
        let public = PublicKey::from_secret_key(&secp, &secret);

        Ok(KeyPair {
            public_key: public.serialize().to_vec(),
            secret_key: secret.secret_bytes().to_vec(),
        })
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(&self.public_key)
    }

    pub fn secret_key_hex(&self) -> String {
        hex::encode(&self.secret_key)
    }
}
