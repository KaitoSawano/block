use serde::{Deserialize, Serialize};

/// Pesan-pesan protokol sinkronisasi ZenaChain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Minta header mulai dari `from_height`.
    GetHeaders {
        from_height: u64,
        max: u64,
    },
    /// Balasan header.
    Headers {
        headers: Vec<BlockHeader>,
    },
    /// Minta blok berdasarkan hash.
    GetData {
        hashes: Vec<String>,
    },
    /// Balasan blok.
    Block {
        height: u64,
        hash: String,
        raw: Vec<u8>,
    },
    /// Ping sinkronisasi.
    Ping,
    /// Pong sinkronisasi.
    Pong,
}

/// Header blok. Cukup untuk verifikasi ringan tanpa full block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub nversion: u32,
    pub index: u64,
    pub timestamp: u128,
    pub previous_hash: String,
    pub merkle_root: String,
    pub nonce: u64,
    pub nbits: u32,
    pub hash: String,
    pub reward: u64,
}

impl SyncMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }
}
