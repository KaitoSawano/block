use serde::{Deserialize, Serialize};

/// Pesan blok untuk broadcast P2P.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZenaBlock {
    pub hash: String,
    pub height: u64,
    pub raw: Vec<u8>,
}

impl ZenaBlock {
    pub fn new(hash: String, height: u64, raw: Vec<u8>) -> Self {
        ZenaBlock { hash, height, raw }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }
}

/// Pesan transaksi untuk broadcast P2P.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZenaTransaction {
    pub txid: String,
    pub raw: Vec<u8>,
}

impl ZenaTransaction {
    pub fn new(txid: String, raw: Vec<u8>) -> Self {
        ZenaTransaction { txid, raw }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }
}
