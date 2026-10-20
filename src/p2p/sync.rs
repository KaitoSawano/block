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

/// Maximum number of blocks that can be requested from a peer at once.
/// This limits memory usage during synchronization (moving download window).
pub const MAX_BLOCKS_IN_FLIGHT: usize = 16;

/// How long to wait for a requested block before considering it timed out.
pub const BLOCK_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub struct SyncState {
    pub best_known_height: u64,
    pub last_request_at_height: Option<u64>,
    pub sync_peer: Option<u64>,
    /// Tracks requested blocks that have not yet been received.
    /// Key: block hash, Value: time when the request was made.
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

    /// Register a block request. Returns false if the window is full.
    pub fn request_block(&mut self, hash: &str) -> bool {
        // Remove expired requests first.
        self.expire_requests();

        if self.in_flight.len() >= MAX_BLOCKS_IN_FLIGHT {
            return false;
        }
        self.in_flight.insert(hash.to_string(), Instant::now());
        true
    }

    /// Mark a block as received and remove it from the in-flight window.
    pub fn received_block(&mut self, hash: &str) {
        self.in_flight.remove(hash);
    }

    /// Remove requests that have exceeded the timeout.
    pub fn expire_requests(&mut self) {
        let now = Instant::now();
        self.in_flight
            .retain(|_, &mut requested_at| now.duration_since(requested_at) < BLOCK_REQUEST_TIMEOUT);
    }

    /// Return the number of blocks currently in flight.
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    /// Handle a potential chain reorganization.
    /// This is a simplified version that finds the common ancestor by walking
    /// back from the new block's previous_hash.
    pub fn handle_reorg(chain: &mut Blockchain, new_block: Block) -> Result<(), String> {
        println!("[reorg] potential reorg detected at height {}", new_block.index);
        
        // 1. Find the common ancestor.
        // Walk back from the new block's parent until we find a block that
        // exists in our current chain.
        let mut ancestor_hash = new_block.previous_hash.clone();
        let mut ancestor_index = new_block.index - 1;
        
        let mut rollback_blocks = Vec::new();
        
        // Find the fork point.
        loop {
            if let Some(our_block) = chain.chain.iter().find(|b| b.hash == ancestor_hash) {
                ancestor_index = our_block.index;
                println!("[reorg] common ancestor found at height {} ({})", ancestor_index, ancestor_hash);
                break;
            }
            // If we don't have the ancestor, we need to request it.
            // This requires a more complex sync state, but for now, we'll
            // assume the ancestor is in our chain. If not, we reject the reorg.
            return Err(format!("common ancestor {} not found in chain", ancestor_hash));
        }
        
        // 2. Collect blocks to rollback (from current tip down to ancestor).
        if ancestor_index < chain.height() {
            rollback_blocks = chain.chain
                .split_off((ancestor_index + 1) as usize)
                .into_iter()
                .rev()
                .collect();
        }
        
        // 3. Rollback state to the ancestor's state_root.
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
        
        println!("[reorg] rolled back {} blocks to height {}", rollback_blocks.len(), ancestor_index);
        
        // 4. Apply the new block.
        // Note: This assumes the new block is the only one we need to apply.
        // In a real implementation, we would need to apply a full new chain.
        if let Err(e) = Self::accept_block(chain, new_block) {
            return Err(format!("failed to apply new block after reorg: {}", e));
        }
        
        Ok(())
    }

    /// Validate a block received from a peer and append it to the chain.
    ///
    /// This function performs full validation, applies every transaction
    /// to the account state, credits the miner with the block reward,
    /// and appends the block to the chain. The difficulty is adjusted
    /// after the block is appended.
    ///
    /// If the block is already in the chain, the function returns an
    /// error with the message "block already exists". The caller is
    /// expected to treat this as a non-fatal condition and mark the
    /// block as known.
    pub fn accept_block(chain: &mut Blockchain, block: Block) -> Result<(), String> {
        // If the block is already in our chain, treat it as already
        // processed. This mirrors Bitcoin Core, where a block that is
        // already known is accepted without further validation.
        if chain.chain.iter().any(|b| b.hash == block.hash) {
            return Err("block already exists".into());
        }

        let prev = match chain.chain.last() {
            Some(p) => p.clone(),
            None => return Err("chain is empty".into()),
        };

        // Check if the block extends the current tip.
        if block.previous_hash != prev.hash {
            // The block does not connect to our tip. It might be a reorg.
            // Check if the block is a direct child of any block in our chain.
            if chain.chain.iter().any(|b| b.hash == block.previous_hash) {
                // This is a potential reorg.
                return Self::handle_reorg(chain, block);
            } else {
                return Err(format!("block {} previous_hash mismatch (orphan)", block.index));
            }
        }

        if let Err(e) = validation::validate_block(chain, &prev, &block) {
            return Err(format!("block {} rejected: {}", block.index, e));
        }

        // Apply every transaction in the block to the account state.
        // If any transaction fails to apply, abort the block and return
        // an error. This keeps the chain state consistent with the
        // block that is about to be appended.
        for tx in &block.transactions {
            if let Err(e) = chain.apply_transaction(tx) {
                return Err(format!(
                    "failed to apply tx {} in block {}: {}",
                    tx.txid, block.index, e
                ));
            }
        }

        // Credit the miner with the block reward (subsidy + fees).
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
