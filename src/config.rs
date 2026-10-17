use crate::datadir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub network: String,
    pub rpc_addr: String,
    pub rpc_user: String,
    pub rpc_password: String,
    pub listen_addr: String,
    pub bootstrap: Vec<String>,
    pub mining_address: String,
    pub created_at: u128,
    pub protocol_version: u32,
}

impl Config {
    pub fn new() -> Self {
        Config {
            network: "main".to_string(),
            rpc_addr: "127.0.0.1:8332".to_string(),
            rpc_user: "zenachain".to_string(),
            rpc_password: "rahasia".to_string(),
            listen_addr: "/ip4/0.0.0.0/tcp/8333".to_string(),
            bootstrap: Vec::new(),
            mining_address: "ZENAMINERDEFAULT".to_string(),
            created_at: crate::utils::now_millis(),
            protocol_version: crate::blockchain::protocol_version(),
        }
    }

    pub fn default_path() -> String {
        datadir::as_str(&datadir::config_path())
    }

    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        let data = match std::fs::read_to_string(&path) {
            Ok(d) => d,
            Err(_) => return Config::new(),
        };
        Config::parse(&data)
    }

    pub fn parse(data: &str) -> Self {
        let mut cfg = Config::new();

        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, value) = match line.split_once('=') {
                Some(kv) => kv,
                None => continue,
            };

            let key = key.trim();
            let value = value.trim().trim_matches('"');

            match key {
                "network" => cfg.network = value.to_string(),
                "rpcport" => cfg.rpc_addr = format!("127.0.0.1:{}", value),
                "rpcbind" => cfg.rpc_addr = value.to_string(),
                "rpcuser" => cfg.rpc_user = value.to_string(),
                "rpcpassword" => cfg.rpc_password = value.to_string(),
                "port" => cfg.listen_addr = format!("/ip4/0.0.0.0/tcp/{}", value),
                "listen" => cfg.listen_addr = value.to_string(),
                "bootstrap" => cfg.bootstrap.push(value.to_string()),
                "miningaddress" => cfg.mining_address = value.to_string(),
                _ => {}
            }
        }
        cfg
    }

    pub fn save_default(&self) -> std::io::Result<()> {
        datadir::ensure_dirs()?;
        let path = Self::default_path();

        let rpcport = self.rpc_addr.rsplit(':').next().unwrap_or("8332");
        let port = self.listen_addr.rsplit(':').next().unwrap_or("8333");

        let mut out = String::new();
        out.push_str("# ZenaChain configuration\n");
        out.push_str(&format!("network={}\n", self.network));
        out.push_str(&format!("rpcport={}\n", rpcport));
        out.push_str(&format!("rpcuser={}\n", self.rpc_user));
        out.push_str(&format!("rpcpassword={}\n", self.rpc_password));
        out.push_str(&format!("port={}\n", port));
        out.push_str(&format!("listen={}\n", self.listen_addr));
        out.push_str(&format!("miningaddress={}\n", self.mining_address));

        for b in &self.bootstrap {
            out.push_str(&format!("bootstrap={}\n", b));
        }

        std::fs::write(path, out)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
