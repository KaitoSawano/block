// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Wallet representation and persistence.

use crate::datadir;
use crate::secp256k1::{address, keypair::KeyPair};
use crate::utils::now_millis;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub public_key: String,
    pub secret: String,
    pub created_at: u128,
}

impl Wallet {
    /// Create a new wallet with a freshly generated key pair.
    pub fn new() -> Self {
        let kp = KeyPair::generate().expect("keypair generation failed");
        let addr = address::address_from_public_key(&kp.public_key).expect("address generation failed");

        Wallet {
            address: addr,
            public_key: kp.public_key_hex(),
            secret: kp.secret_key_hex(),
            created_at: now_millis(),
        }
    }

    /// Restore a wallet from a hexadecimal private key.
    pub fn from_secret(secret_hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(secret_hex).map_err(|e| e.to_string())?;
        let kp = KeyPair::from_secret(&bytes)?;
        let addr = address::address_from_public_key(&kp.public_key)?;

        Ok(Wallet {
            address: addr,
            public_key: kp.public_key_hex(),
            secret: kp.secret_key_hex(),
            created_at: now_millis(),
        })
    }

    /// Return the key pair associated with this wallet.
    pub fn keypair(&self) -> Result<KeyPair, String> {
        let sk = hex::decode(&self.secret).map_err(|e| e.to_string())?;
        KeyPair::from_secret(&sk)
    }

    /// Save the wallet to the default location.
    pub fn save_default(&self) -> std::io::Result<()> {
        datadir::ensure_dirs()?;
        self.save(&datadir::as_str(&datadir::wallet_path()))
    }

    /// Load the wallet from the default location.
    pub fn load_default() -> std::io::Result<Self> {
        Self::load(&datadir::as_str(&datadir::wallet_path()))
    }

    /// Return whether a wallet exists at the default location.
    pub fn exists_default() -> bool {
        datadir::wallet_path().exists()
    }

    /// Save the wallet to the given path.
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Load a wallet from the given path.
    pub fn load(path: &str) -> std::io::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let w: Wallet = serde_json::from_str(&data).map_err(std::io::Error::other)?;
        Ok(w)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}
