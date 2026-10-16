//! Logika sinkronisasi chain dari peer (IBD).
//!
//! Tahap 1: minta header dari peer, validasi header secara ringan.
//! Tahap 2 (belum): minta full block, validasi penuh, tambahkan ke chain.

use crate::blockchain::Blockchain;
use crate::p2p::codec::{BlockHeader, SyncMessage};
use crate::p2p::peers;
use crate::validation;

pub struct SyncState {
    pub best_known_height: u64,
    pub last_request_at_height: Option<u64>,
    pub sync_peer: Option<libp2p::PeerId>,
}

impl SyncState {
    pub fn new() -> Self {
        SyncState {
            best_known_height: 0,
            last_request_at_height: None,
            sync_peer: None,
        }
    }

    pub fn is_synced(&self, chain: &Blockchain) -> bool {
        self.best_known_height <= chain.height()
    }

    /// Bikin pesan GetHeaders dari tinggi sekarang sampai max.
    pub fn make_get_headers(&self, chain: &Blockchain) -> SyncMessage {
        SyncMessage::GetHeaders {
            from_height: chain.height() + 1,
            max: peers::HEADERS_PER_REQUEST,
        }
    }

    /// Proses pesan Headers dari peer.
    pub fn apply_headers(
        &mut self,
        chain: &Blockchain,
        headers: &[BlockHeader],
    ) -> Result<(), String> {
        if headers.is_empty() {
            return Ok(());
        }

        let tip = chain.chain.last().unwrap();
        let first = &headers[0];

        if first.index != tip.index + 1 {
            return Err(format!(
                "header pertama index {} tidak nyambung dengan tip {}",
                first.index, tip.index
            ));
        }

        if first.previous_hash != tip.hash {
            return Err(format!(
                "header pertama previous_hash {} tidak cocok dengan tip hash {}",
                first.previous_hash, tip.hash
            ));
        }

        for i in 1..headers.len() {
            let prev = &headers[i - 1];
            let cur = &headers[i];

            if cur.index != prev.index + 1 {
                return Err(format!(
                    "header {} tidak berurutan setelah {}",
                    cur.index, prev.index
                ));
            }

            if cur.previous_hash != prev.hash {
                return Err(format!(
                    "header {} previous_hash tidak cocok dengan header sebelumnya",
                    cur.index
                ));
            }
        }

        for h in headers {
            let expected = Blockchain::calculate_reward(h.index);
            if h.reward != expected {
                return Err(format!("header {} reward tidak sesuai era", h.index));
            }
        }

        for h in headers {
            if !crate::pow::hash_meets_nbits(&h.hash, h.nbits) {
                return Err(format!(
                    "header {} hash tidak memenuhi target nbits",
                    h.index
                ));
            }
        }

        let last = headers.last().unwrap();
        self.best_known_height = last.index;
        self.last_request_at_height = Some(last.index);

        Ok(())
    }

    pub fn finalize(&self, chain: &Blockchain) -> bool {
        validation::validate_chain(chain).is_ok()
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}
