use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use std::io::{Error, ErrorKind};

pub const CHAIN_FILE: &str = "chain.json";
pub const WALLET_FILE: &str = "wallet.json";

pub fn save_chain(chain: &Blockchain, path: &str) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(chain)
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
    std::fs::write(path, json)
}

pub fn load_chain(path: &str) -> std::io::Result<Blockchain> {
    let data = std::fs::read_to_string(path)?;
    let chain: Blockchain = serde_json::from_str(&data)
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
    Ok(chain)
}

pub fn save_wallet(wallet: &Wallet, path: &str) -> std::io::Result<()> {
    wallet.save(path)
}

pub fn load_wallet(path: &str) -> std::io::Result<Wallet> {
    Wallet::load(path)
}
