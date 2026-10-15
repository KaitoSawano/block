use serde::{Deserialize, Serialize};

pub const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mining_address: String,
    pub created_at: u128,
}

impl Config {
    pub fn new(mining_address: String) -> Self {
        println!("[config] new: membuat konfigurasi node dengan alamat mining.");
        Config {
            mining_address,
            created_at: crate::utils::now_millis(),
        }
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        println!("[config] save: menulis konfigurasi ke berkas.");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    pub fn load(path: &str) -> std::io::Result<Self> {
        println!("[config] load: membaca konfigurasi dari berkas.");
        let data = std::fs::read_to_string(path)?;
        let c: Config = serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(c)
    }

    pub fn load_or_default(path: &str) -> Self {
        match Config::load(path) {
            Ok(c) => c,
            Err(_) => {
                println!("[config] load_or_default: config tidak ditemukan, memakai alamat default.");
                Config::new("ZENAMINERDEFAULT".to_string())
            }
        }
    }
}
