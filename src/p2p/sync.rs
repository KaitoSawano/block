// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Chain synchronization logic for the ZenaChain peer-to-peer network.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::validation;

pub const MAX_BLOCKS_IN_FLIGHT: usize = 16;
pub const BLOCK_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub struct SyncState {
    pub best_known_height: u64,
    pub last_request_at_height: Option<u64>,
    pub sync_peer: Option<u64>,
    pub in_flight: HashMap<String, Instant>,
}

impl SyncState {
    pub fn new() -> Self {
        SyncState {
            best_known_height: 0,
            last_request_at_height: None,
            sync_peer: None,
            in_flight: HashMap::new(),
        }
    }

    pub fn is_synced(&self, chain: &Blockchain) -> bool {
        self.best_known_height <= chain.height()
    }

    pub fn request_block(&mut self, hash: &str) -> bool {
        self.expire_requests();
        if self.in_flight.len() >= MAX_BLOCKS_IN_FLIGHT {
            return false;
        }
        self.in_flight.insert(hash.to_string(), Instant::now());
        true
    }

    pub fn received_block(&mut self, hash: &str) {
        self.in_flight.remove(hash);
    }

    pub fn expire_requests(&mut self) {
        let now = Instant::now();
        self.in_flight
            .retain(|_, &mut requested_at| now.duration_since(requested_at) < BLOCK_REQUEST_TIMEOUT);
    }

    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    /// Handle a potential chain reorganization based on total work.
    /// If the new block's chain has more total work than our current tip,
    /// we roll back to the common ancestor and apply the new block.
    pub fn handle_reorg(chain: &mut Blockchain, new_block: Block) -> Result<(), String> {
        println!("[reorg] potential reorg detected at height {}", new_block.index);

        // 1. Compute the total work of the new block's chain.
        // The new block's chainwork already includes the work of all its ancestors.
        let new_chain_work = new_block.chainwork;

        // 2. Compare with our current chain's total work.
        let our_chain_work = chain.total_chainwork();
        if new_chain_work <= our_chain_work {
            println!(
                "[reorg] rejecting reorg: new chain work {} <= our chain work {}",
                new_chain_work, our_chain_work
            );
            return Err("new chain has less or equal work".into());
        }

        // 3. Find the common ancestor.
        let ancestor_hash = new_block.previous_hash.clone();
        let ancestor_index;

        loop {
            if let Some(our_block) = chain.chain.iter().find(|b| b.hash == ancestor_hash) {
                ancestor_index = our_block.index;
                println!(
                    "[reorg] common ancestor found at height {} ({})",
                    ancestor_index, ancestor_hash
                );
                break;
            }
            return Err(format!("common ancestor {} not found in chain", ancestor_hash));
        }

        // 4. Collect blocks to rollback (from current tip down to ancestor).
        let rollback_blocks;
        if ancestor_index < chain.height() {
            rollback_blocks = chain.chain
                .split_off((ancestor_index + 1) as usize)
                .into_iter()
                .rev()
                .collect::<Vec<_>>();
        } else {
            rollback_blocks = Vec::new();
        }

        // 5. Rollback state to the ancestor's state_root.
        let ancestor_block = &chain.chain[ancestor_index as usize];
        if ancestor_block.state_root.is_empty() {
            return Err("ancestor block has no state root".into());
        }

        let root_bytes = hex::decode(&ancestor_block.state_root)
            .map_err(|e| format!("invalid state root hex: {}", e))?;
        let mut root_arr = [0u8; 32];
        root_arr.copy_from_slice(&root_bytes);

        if let Err(e) = chain.state.rollback_to(&root_arr) {
            return Err(format!("state rollback failed: {}", e));
        }

        println!(
            "[reorg] rolled back {} blocks to height {}",
            rollback_blocks.len(),
            ancestor_index
        );

        // 6. Apply the new block.
        if let Err(e) = Self::accept_block(chain, new_block) {
            return Err(format!("failed to apply new block after reorg: {}", e));
        }

        Ok(())
    }

    /// Validate a block received from a peer and append it to the chain.
    pub fn accept_block(chain: &mut Blockchain, block: Block) -> Result<(), String> {
        if chain.chain.iter().any(|b| b.hash == block.hash) {
            return Err("block already exists".into());
        }

        let prev = match chain.chain.last() {
            Some(p) => p.clone(),
            None => return Err("chain is empty".into()),
        };

        // If the block does not connect to the tip, it might be a reorg.
        if block.previous_hash != prev.hash {
            if chain.chain.iter().any(|b| b.hash == block.previous_hash) {
                return Self::handle_reorg(chain, block);
            } else {
                return Err(format!("block {} previous_hash mismatch (orphan)", block.index));
            }
        }

        if let Err(e) = validation::validate_block(chain, &prev, &block) {
            return Err(format!("block {} rejected: {}", block.index, e));
        }

        for tx in &block.transactions {
            if let Err(e) = chain.apply_transaction(tx) {
                return Err(format!(
                    "failed to apply tx {} in block {}: {}",
                    tx.txid, block.index, e
                ));
            }
        }

        if let Err(e) = chain.credit_reward(&block.mined_for, block.reward, block.index) {
            return Err(format!(
                "failed to credit reward for block {}: {}",
                block.index, e
            ));
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
