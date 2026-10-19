// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Chain synchronization logic for the ZenaChain peer-to-peer network.

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

        if block.index != prev.index + 1 {
            return Err(format!(
                "block index {} does not follow tip {}",
                block.index, prev.index
            ));
        }

        if block.previous_hash != prev.hash {
            return Err(format!("block {} previous_hash mismatch", block.index));
        }

        if let Err(e) = validation::validate_block(chain, &prev, &block) {
            return Err(format!("block {} rejected: {}", block.index, e));
        }

        // Apply every transaction in the block to the account state.
        for tx in &block.transactions {
            chain.apply_transaction(tx);
        }

        // Credit the miner with the block reward (subsidy + fees).
        let miner_balance = chain
            .balances
            .entry(block.mined_for.clone())
            .or_insert(0);
        *miner_balance = miner_balance.saturating_add(block.reward);

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
