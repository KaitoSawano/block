use crate::datadir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mining_address: String,
    pub created_at: u128,
    pub protocol_version: u32,
    pub network: String,
}

impl Config {
    pub fn new(mining_address: String) -> Self {
        println!("[config] new: membuat konfigurasi dengan alamat mining.");
        Config {
            mining_address,
            created_at: crate::utils::now_millis(),
            protocol_version: crate::blockchain::protocol_version(),
            network: "main".to_string(),
        }
    }

    pub fn save_default(&self) -> std::io::Result<()> {
        datadir::ensure_dirs()?;
        println!("[config] save_default: menyimpan konfigurasi ke .chain/zenachain.conf.");
        let path = datadir::as_str(&datadir::config_path());
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    pub fn load_default() -> std::io::Result<Self> {
        println!("[config] load_default: memuat konfigurasi dari .chain/zenachain.conf.");
        let path = datadir::as_str(&datadir::config_path());
        let data = std::fs::read_to_string(path)?;
        let c: Config = serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(c)
    }

    pub fn load_or_default() -> Self {
        match Config::load_default() {
            Ok(c) => c,
            Err(_) => {
                println!("[config] load_or_default: config tidak ada, memakai alamat default.");
                Config::new("ZENAMINERDEFAULT".to_string())
            }
        }
    }
}
