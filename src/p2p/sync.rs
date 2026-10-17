//! Logika sinkronisasi chain dari peer.

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::validation;

pub struct SyncState {
    pub best_known_height: u64,
    pub last_request_at_height: Option<u64>,
    pub sync_peer: Option<u64>,
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

    pub fn accept_block(chain: &mut Blockchain, block: Block) -> Result<(), String> {
        let prev = match chain.chain.last() {
            Some(p) => p.clone(),
            None => return Err("chain kosong".into()),
        };

        if block.index != prev.index + 1 {
            return Err(format!(
                "blok index {} tidak nyambung dengan tip {}",
                block.index, prev.index
            ));
        }

        if block.previous_hash != prev.hash {
            return Err(format!("blok {} previous_hash tidak cocok", block.index));
        }

        if let Err(e) = validation::validate_block(chain, &prev, &block) {
            return Err(format!("blok {} ditolak: {}", block.index, e));
        }

        chain.chain.push(block);
        chain.adjust_difficulty();
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
