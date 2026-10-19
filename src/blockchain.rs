// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Blockchain state and consensus-related operations.

use std::collections::HashMap;

use crate::block::Block;
use crate::consensus::{self, genesis, pow as cpow, rules, spec};
use crate::pow;
use crate::transaction::Transaction;
use crate::validation;
use serde::{Deserialize, Serialize};

pub mod checkpoint {
    pub struct Checkpoint {
        pub height: u64,
        pub hash: &'static str,
    }

    pub const CHECKPOINTS: &[Checkpoint] = &[Checkpoint {
        height: 0,
        hash: "0000000000000000000000000000000000000000000000000000000000000000",
    }];

    pub fn find(height: u64) -> Option<&'static str> {
        CHECKPOINTS.iter().find(|c| c.height == height).map(|c| c.hash)
    }

    pub fn is_checkpoint(height: u64) -> bool {
        CHECKPOINTS.iter().any(|c| c.height == height)
    }

    pub fn highest() -> u64 {
        CHECKPOINTS.iter().map(|c| c.height).max().unwrap_or(0)
    }

    pub fn verify(height: u64, hash: &str) -> bool {
        match find(height) {
            Some(expected) => expected == hash,
            None => true,
        }
    }
}

pub mod network {
    pub const NETWORK_MAGIC: [u8; 4] = [0x5a, 0x45, 0x4e, 0x41];
    pub const NETWORK_NAME: &str = "main";
    pub const DEFAULT_PORT: u16 = 8333;
    pub const PRUNE_AFTER_HEIGHT: u64 = 100_000;
    pub const ASSUMED_BLOCKCHAIN_SIZE: u64 = 496;
    pub const ASSUMED_CHAIN_STATE_SIZE: u64 = 6;

    pub fn network_magic_hex() -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}",
            NETWORK_MAGIC[0], NETWORK_MAGIC[1], NETWORK_MAGIC[2], NETWORK_MAGIC[3]
        )
    }
}

pub use consensus::consensus_fingerprint;
pub use consensus::genesis_coinbase_txid;
pub use genesis::genesis_message;
pub use genesis::genesis_output_script;
pub use genesis::genesis_secret;
pub use genesis::genesis_timestamp;
pub use network::DEFAULT_PORT;
pub use spec::coin_name;
pub use spec::decimals;
pub use spec::difficulty_adjust_interval;
pub use spec::hash_algo;
pub use spec::initial_reward;
pub use spec::initial_reward_zenac;
pub use spec::max_halvings;
pub use spec::max_supply;
pub use spec::max_supply_zenac;
pub use spec::miner_confirmation_window;
pub use spec::pow_target_spacing_secs;
pub use spec::pow_target_timespan_secs;
pub use spec::protocol_version;
pub use spec::satoshi;
pub use spec::signature_algo;
pub use spec::subsidy_halving_interval;
pub use spec::symbol;
pub use spec::version;

pub fn max_tx_per_block() -> usize {
    rules::MAX_TX_PER_BLOCK
}
pub fn pow_limit() -> &'static str {
    cpow::POW_LIMIT
}
pub fn genesis_nbits() -> u32 {
    cpow::GENESIS_NBITS
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub nbits: u32,
    pub mempool: Vec<Transaction>,
    pub balances: HashMap<String, u64>,
    pub nonces: HashMap<String, u64>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::genesis();
        let mut balances = HashMap::new();
        let mut nonces = HashMap::new();

        // Initialize the genesis coinbase balance.
        // The genesis block credits its reward to the "GENESIS" address,
        // which is not spendable but keeps the supply accounting
        // consistent.
        balances.insert("GENESIS".to_string(), genesis.reward);
        nonces.insert("GENESIS".to_string(), 0);

        Blockchain {
            chain: vec![genesis],
            nbits: cpow::GENESIS_NBITS,
            mempool: Vec::new(),
            balances,
            nonces,
        }
    }

    pub fn height(&self) -> u64 {
        self.chain.last().unwrap().index
    }

    pub fn circulating_supply(&self) -> u64 {
        self.chain
            .iter()
            .fold(0u64, |acc, b| acc.saturating_add(b.reward))
    }

    pub fn calculate_reward(height: u64) -> u64 {
        let halvings = (height / spec::SUBSIDY_HALVING_INTERVAL) as u32;
        if halvings >= spec::MAX_HALVINGS {
            return 0;
        }
        spec::INITIAL_REWARD >> halvings
    }

    pub fn current_era(&self) -> u64 {
        self.height() / spec::SUBSIDY_HALVING_INTERVAL
    }

    pub fn blocks_until_halving(&self) -> u64 {
        let next = (self.current_era() + 1) * spec::SUBSIDY_HALVING_INTERVAL;
        next.saturating_sub(self.height())
    }

    pub fn balance_of(&self, addr: &str) -> u64 {
        self.balances.get(addr).copied().unwrap_or(0)
    }

    pub fn nonce_of(&self, addr: &str) -> u64 {
        self.nonces.get(addr).copied().unwrap_or(0)
    }

    pub fn apply_transaction(&mut self, tx: &Transaction) {
        let from_balance = self.balances.entry(tx.from.clone()).or_insert(0);
        *from_balance = from_balance.saturating_sub(tx.amount.saturating_add(tx.fee));

        let to_balance = self.balances.entry(tx.to.clone()).or_insert(0);
        *to_balance = to_balance.saturating_add(tx.amount);

        let nonce = self.nonces.entry(tx.from.clone()).or_insert(0);
        *nonce = nonce.saturating_add(1);
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        if validation::validate_transaction(self, &tx).is_err() {
            return false;
        }
        if self.mempool.len() >= rules::MAX_TX_PER_BLOCK {
            return false;
        }
        self.mempool.push(tx);
        true
    }

    pub fn mine_pending(&mut self) -> Result<u64, String> {
        let prev = self.chain.last().unwrap().clone();
        let next_index = prev.index + 1;
        let subsidy = Self::calculate_reward(next_index);
        let total_fees: u64 = self.mempool.iter().map(|tx| tx.fee).sum();
        let reward = subsidy.saturating_add(total_fees);

        if self.circulating_supply().saturating_add(reward) > spec::MAX_SUPPLY {
            return Err("max supply reached".into());
        }

        let txs = std::mem::take(&mut self.mempool);
        let mut new_block = Block::new_for(
            next_index,
            prev.hash.clone(),
            subsidy,
            total_fees,
            self.nbits,
            txs.clone(),
            String::new(),
        );
        if !pow::mine(&mut new_block, self.nbits) {
            return Err("mining failed: nonce exhausted".into());
        }

        for tx in &txs {
            self.apply_transaction(tx);
        }

        let miner_balance = self.balances.entry(new_block.mined_for.clone()).or_insert(0);
        *miner_balance = miner_balance.saturating_add(reward);

        let nonce = new_block.nonce;
        self.chain.push(new_block);
        self.adjust_difficulty();
        Ok(nonce)
    }

    pub fn adjust_difficulty(&mut self) {
        let height = self.height();
        if height == 0 || !height.is_multiple_of(spec::DIFFICULTY_ADJUST_INTERVAL) {
            return;
        }

        let interval = spec::DIFFICULTY_ADJUST_INTERVAL as usize;
        let last = &self.chain[height as usize];
        let first = &self.chain[height as usize - interval];

        let actual_time_ms = last.timestamp.saturating_sub(first.timestamp) as u64;
        let actual_time_secs = actual_time_ms / 1000;
        let expected_time_secs = spec::POW_TARGET_TIMESPAN_SECS;

        if actual_time_secs == 0 {
            return;
        }

        let current_target = pow::nbits_to_target(self.nbits);
        let scale = (actual_time_secs as u128) * 256 / (expected_time_secs as u128);
        let new_target = scale_target(&current_target, scale);
        let new_nbits = target_to_nbits(&new_target);

        if new_nbits != self.nbits {
            self.nbits = new_nbits;
        }
    }

    pub fn is_valid(&self) -> bool {
        validation::validate_chain(self).is_ok()
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

fn scale_target(target: &[u8; 32], scale_256: u128) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut carry: u128 = 0;

    for i in (0..32).rev() {
        let v = (target[i] as u128) * scale_256 + carry;
        result[i] = (v & 0xff) as u8;
        carry = v >> 8;
    }
    result
}

fn target_to_nbits(target: &[u8; 32]) -> u32 {
    let mut i = 0;
    while i < 32 && target[i] == 0 {
        i += 1;
    }
    if i == 32 {
        return cpow::GENESIS_NBITS;
    }

    let size = 32 - i;
    let mut mantissa: u32 = (target[i] as u32) << 16;
    if i + 1 < 32 {
        mantissa |= (target[i + 1] as u32) << 8;
    }
    if i + 2 < 32 {
        mantissa |= target[i + 2] as u32;
    }

    if mantissa & 0x00800000 == 0 {
        mantissa <<= 8;
        return (((size - 1) as u32) << 24) | (mantissa & 0x00ffffff);
    }

    ((size as u32) << 24) | (mantissa & 0x00ffffff)
}
