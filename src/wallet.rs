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
    pub fn new() -> Self {
        println!("[wallet] new: membuat wallet ZENAC dengan secp256k1.");
        let kp = KeyPair::generate().expect("generate keypair");
        let addr = address::address_from_public_key(&kp.public_key).expect("address");

        Wallet {
            address: addr,
            public_key: kp.public_key_hex(),
            secret: kp.secret_key_hex(),
            created_at: now_millis(),
        }
    }

    pub fn from_secret(secret_hex: &str) -> Result<Self, String> {
        println!("[wallet] from_secret: membangun wallet dari private key secp256k1.");
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

    pub fn keypair(&self) -> Result<KeyPair, String> {
        let sk = hex::decode(&self.secret).map_err(|e| e.to_string())?;
        KeyPair::from_secret(&sk)
    }

    pub fn save_default(&self) -> std::io::Result<()> {
        datadir::ensure_dirs()?;
        println!("[wallet] save_default: menyimpan wallet ke .chain/wallet.dat.");
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
