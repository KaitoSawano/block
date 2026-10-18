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
pub struct WalletEntry {
    pub address: String,
    pub public_key: String,
    pub secret: String, // Jika wallet terkunci, ini berisi ciphertext base64
    pub label: String,
    pub created_at: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub entries: Vec<WalletEntry>,
    pub created_at: u128,
    pub encrypted: bool,
    pub salt: String,
    pub nonce: String,
}

impl Wallet {
    /// Create a new wallet with a freshly generated key pair.
    pub fn new() -> Self {
        let mut w = Wallet {
            entries: Vec::new(),
            created_at: now_millis(),
            encrypted: false,
            salt: String::new(),
            nonce: String::new(),
        };
        w.add_new_address("default".to_string());
        w
    }

    /// Add a new address to the wallet.
    pub fn add_new_address(&mut self, label: String) -> String {
        let kp = KeyPair::generate().expect("keypair generation failed");
        let addr = address::address_from_public_key(&kp.public_key).expect("address generation failed");
        
        let entry = WalletEntry {
            address: addr.clone(),
            public_key: kp.public_key_hex(),
            secret: kp.secret_key_hex(),
            label,
            created_at: now_millis(),
        };
        
        self.entries.push(entry);
        addr
    }

    /// Get a specific entry by address.
    pub fn get_entry(&self, addr: &str) -> Option<&WalletEntry> {
        self.entries.iter().find(|e| e.address == addr)
    }

    /// Get the most recently created address.
    pub fn default_address(&self) -> Option<&WalletEntry> {
        self.entries.last()
    }

    /// Restore a wallet from a hexadecimal private key.
    pub fn from_secret(secret_hex: &str, label: String) -> Result<WalletEntry, String> {
        let bytes = hex::decode(secret_hex).map_err(|e| e.to_string())?;
        let kp = KeyPair::from_secret(&bytes)?;
        let addr = address::address_from_public_key(&kp.public_key)?;

        Ok(WalletEntry {
            address: addr,
            public_key: kp.public_key_hex(),
            secret: kp.secret_key_hex(),
            label,
            created_at: now_millis(),
        })
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

    // === FITUR ENKRIPSI ===

    /// Encrypt all private keys in the wallet using AES-256-GCM.
    pub fn encrypt(&mut self, password: &str) -> Result<(), String> {
        if self.encrypted {
            return Err("wallet is already encrypted".into());
        }
        
        let salt = crate::utils::random_hex(16);
        let nonce_hex = crate::utils::random_hex(12);
        
        let key = crate::utils::derive_key(password, &salt);
        let nonce = crate::utils::hex_to_bytes(&nonce_hex)?;
        
        for entry in self.entries.iter_mut() {
            let ciphertext = crate::utils::aes_encrypt(&key, &nonce, &entry.secret)?;
            entry.secret = ciphertext;
        }
        
        self.encrypted = true;
        self.salt = salt;
        self.nonce = nonce_hex;
        
        Ok(())
    }

    /// Decrypt all private keys using the provided password.
    /// This does not modify the wallet on disk; it returns a temporary decrypted copy.
    pub fn decrypt(&self, password: &str) -> Result<Wallet, String> {
        if !self.encrypted {
            return Ok(self.clone());
        }
        
        let key = crate::utils::derive_key(password, &self.salt);
        let nonce = crate::utils::hex_to_bytes(&self.nonce)?;
        
        let mut decrypted = self.clone();
        
        for entry in decrypted.entries.iter_mut() {
            let plaintext = crate::utils::aes_decrypt(&key, &nonce, &entry.secret)?;
            entry.secret = plaintext;
        }
        
        decrypted.encrypted = false;
        Ok(decrypted)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}
