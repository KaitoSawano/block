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
        Config {
            mining_address,
            created_at: crate::utils::now_millis(),
            protocol_version: crate::blockchain::protocol_version(),
            network: "main".to_string(),
        }
    }

    pub fn save_default(&self) -> std::io::Result<()> {
        datadir::ensure_dirs()?;
        let path = datadir::as_str(&datadir::config_path());
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    pub fn load_default() -> std::io::Result<Self> {
        let path = datadir::as_str(&datadir::config_path());
        let data = std::fs::read_to_string(path)?;
        let c: Config = serde_json::from_str(&data).map_err(std::io::Error::other)?;
        Ok(c)
    }

    pub fn load_or_default() -> Self {
        match Config::load_default() {
            Ok(c) => c,
            Err(_) => Config::new("ZENAMINERDEFAULT".to_string()),
        }
    }
}
