use crate::datadir;
use crate::utils::{now_millis, sha3_256_hex};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub public_key: String,
    pub secret: String,
    pub created_at: u128,
}

impl Wallet {
    pub fn new() -> Self {
        println!("[wallet] new: membuat wallet ZENAC dengan secret acak 32 byte.");
        let mut secret_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut secret_bytes);
        let secret = hex::encode(secret_bytes);

        let public_key = sha3_256_hex(format!("{}pub", secret).as_bytes());
        let addr_hash = sha3_256_hex(format!("{}addr", public_key).as_bytes());
        let address = format!("ZENA{}", &addr_hash[..40]);

        Wallet {
            address,
            public_key,
            secret,
            created_at: now_millis(),
        }
    }

    pub fn save_default(&self) -> std::io::Result<()> {
        println!("[wallet] save_default: menyimpan wallet ke .chain/wallet.dat.");
        datadir::ensure_dirs()?;
        self.save(&datadir::as_str(&datadir::wallet_path()))
    }

    pub fn load_default() -> std::io::Result<Self> {
        println!("[wallet] load_default: memuat wallet dari .chain/wallet.dat.");
        Self::load(&datadir::as_str(&datadir::wallet_path()))
    }

    pub fn exists_default() -> bool {
        datadir::wallet_path().exists()
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    pub fn load(path: &str) -> std::io::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let w: Wallet = serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(w)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}
