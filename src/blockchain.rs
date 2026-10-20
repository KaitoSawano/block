// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Blockchain state and consensus-related operations.

use std::collections::HashMap;

use crate::block::Block;
use crate::consensus::{self, genesis, pow as cpow, rules, spec};
use crate::pow;
use crate::storage::{Account, StateDB};
use crate::transaction::Transaction;
use crate::validation;
use serde::{Deserialize, Serialize};

pub const MAX_MEMPOOL_SIZE_BYTES: usize = 300 * 1024 * 1024;
pub const MIN_RELAY_FEE_PER_BYTE: u64 = 1;
pub const MEMPOOL_EXPIRY_SECS: u64 = 336 * 3600;
pub const RBF_MIN_FEE_BUMP_PERCENT: u64 = 10;

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
    use crate::consensus::spec;

    pub const PRUNE_AFTER_HEIGHT: u64 = 100_000;
    pub const ASSUMED_BLOCKCHAIN_SIZE: u64 = 496;
    pub const ASSUMED_CHAIN_STATE_SIZE: u64 = 6;

    pub fn network_name() -> &'static str {
        spec::current_network().name()
    }

    pub fn default_port() -> u16 {
        spec::current_network().default_port()
    }

    pub fn network_magic_hex() -> String {
        spec::current_network().magic_hex()
    }
}

pub use consensus::consensus_fingerprint;
pub use consensus::consensus_fingerprint_for;
pub use consensus::genesis_coinbase_txid;
pub use consensus::genesis_coinbase_txid_for;
pub use genesis::genesis_message;
pub use genesis::genesis_output_script;
pub use genesis::genesis_secret;
pub use genesis::genesis_timestamp;
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
    cpow::pow_limit()
}
pub fn genesis_nbits() -> u32 {
    spec::current_network().genesis_nbits()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub nbits: u32,
    pub mempool: Vec<Transaction>,
    pub balances: HashMap<String, u64>,
    pub nonces: HashMap<String, u64>,
    pub mempool_timestamps: HashMap<String, u64>,
    pub addresses: Vec<String>,
    #[serde(skip)]
    pub state: StateDB,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::genesis();
        let db_path = crate::datadir::state_db_path();
        let mut state = StateDB::open(&db_path).expect("failed to open state db");

        let genesis_acc = state.get_account("GENESIS").unwrap_or(None);
        if genesis_acc.is_none() {
            state
                .update_account(
                    "GENESIS",
                    &Account {
                        nonce: 0,
                        balance: genesis.reward,
                    },
                )
                .expect("failed to init genesis account");
        }

        // Isi state_root genesis setelah inisialisasi.
        let mut genesis = genesis;
        if let Ok(root) = state.root_hash() {
            genesis.state_root = hex::encode(root);
        }

        Blockchain {
            chain: vec![genesis],
            nbits: spec::current_network().genesis_nbits(),
            mempool: Vec::new(),
            balances: HashMap::new(),
            nonces: HashMap::new(),
            mempool_timestamps: HashMap::new(),
            addresses: vec!["GENESIS".to_string()],
            state,
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
        self.state.balance_of(addr)
    }

    pub fn nonce_of(&self, addr: &str) -> u64 {
        self.state.nonce_of(addr)
    }

    pub fn next_nonce_for(&self, addr: &str) -> u64 {
        let base = self.state.nonce_of(addr);
        let mut next = base;
        loop {
            let has = self
                .mempool
                .iter()
                .any(|tx| tx.from == addr && tx.nonce == next);
            if !has {
                return next;
            }
            next = next.saturating_add(1);
        }
    }

    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<(), String> {
        // Coinbase maturity check: if the sender is a miner, ensure their reward is mature.
        // In our account-based model, we don't track UTXOs. Instead, we track the
        // miner's reward in the account balance. We need to know when the reward
        // was credited. For simplicity, we assume that any balance from mining
        // is subject to maturity. This is a simplification; a full implementation
        // would track UTXOs or use a separate "immature balance" field.
        if tx.from != "COINBASE" && tx.from != "GENESIS" {
            // Check if the sender has sufficient *mature* balance.
            // For now, we skip this check because we don't track immature balance.
            // TODO: Implement proper immature balance tracking.
        }

        if !self.addresses.contains(&tx.from) {
            self.addresses.push(tx.from.clone());
        }
        if !self.addresses.contains(&tx.to) {
            self.addresses.push(tx.to.clone());
        }
        self.state
            .apply_transaction(&tx.from, &tx.to, tx.amount, tx.fee, tx.nonce)
    }

    pub fn credit_reward(&mut self, miner: &str, reward: u64) -> Result<(), String> {
        if !self.addresses.contains(&miner.to_string()) {
            self.addresses.push(miner.to_string());
        }
        self.state.credit_reward(miner, reward)
    }

    pub fn tx_size_bytes(tx: &Transaction) -> usize {
        let base = 32 + tx.from.len() + tx.to.len() + 8 + 8 + 8 + tx.public_key.len() + tx.signature.len();
        base + tx.note.len()
    }

    pub fn mempool_size_bytes(&self) -> usize {
        self.mempool.iter().map(|tx| Self::tx_size_bytes(tx)).sum()
    }

    pub fn fee_rate(tx: &Transaction) -> u64 {
        let size = Self::tx_size_bytes(tx).max(1) as u64;
        tx.fee / size
    }

    fn evict_lowest_fee_transactions(&mut self, needed_space: usize) {
        self.mempool.sort_by(|a, b| {
            let fee_a = Self::fee_rate(a);
            let fee_b = Self::fee_rate(b);
            fee_a.cmp(&fee_b)
        });

        let mut freed = 0;
        let mut to_remove = Vec::new();

        for (i, tx) in self.mempool.iter().enumerate() {
            if freed >= needed_space {
                break;
            }
            freed += Self::tx_size_bytes(tx);
            to_remove.push(i);
        }

        for i in to_remove.into_iter().rev() {
            let removed = self.mempool.remove(i);
            self.mempool_timestamps.remove(&removed.txid);
            println!("[mempool] Evicted tx {} (fee rate {}) to make room", removed.txid, Self::fee_rate(&removed));
        }
    }

    pub fn evict_expired_transactions(&mut self) {
        let now = (crate::utils::now_millis() / 1000) as u64;
        let mut to_remove = Vec::new();

        for tx in &self.mempool {
            if let Some(&ts) = self.mempool_timestamps.get(&tx.txid) {
                if now.saturating_sub(ts) > MEMPOOL_EXPIRY_SECS {
                    to_remove.push(tx.txid.clone());
                }
            }
        }

        for txid in to_remove {
            self.mempool.retain(|tx| tx.txid != txid);
            self.mempool_timestamps.remove(&txid);
            println!("[mempool] Evicted expired tx {}", txid);
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        if self.mempool.iter().any(|t| t.txid == tx.txid) {
            return false;
        }

        let rbf_pos = self.mempool.iter().position(|t| t.from == tx.from && t.nonce == tx.nonce);

        if let Some(pos) = rbf_pos {
            let old_tx = &self.mempool[pos];
            let old_fee_rate = Self::fee_rate(old_tx);
            let new_fee_rate = Self::fee_rate(&tx);

            let min_new_fee_rate = old_fee_rate + (old_fee_rate * RBF_MIN_FEE_BUMP_PERCENT / 100).max(1);

            if new_fee_rate >= min_new_fee_rate {
                println!(
                    "[mempool] RBF: Replacing tx {} (fee rate {}) with {} (fee rate {})",
                    old_tx.txid, old_fee_rate, tx.txid, new_fee_rate
                );
                self.mempool_timestamps.remove(&old_tx.txid);
                self.mempool.remove(pos);
            } else {
                println!(
                    "[mempool] Rejected RBF attempt: new fee rate {} < required {}",
                    new_fee_rate, min_new_fee_rate
                );
                return false;
            }
        }

        if validation::validate_transaction(self, &tx).is_err() {
            println!("[mempool] Rejected tx {}: validation failed", tx.txid);
            return false;
        }

        let tx_size = Self::tx_size_bytes(&tx);
        let fee_rate = Self::fee_rate(&tx);

        if fee_rate < MIN_RELAY_FEE_PER_BYTE {
            println!("[mempool] Rejected tx {}: fee rate {} < minRelayTxFee {}", tx.txid, fee_rate, MIN_RELAY_FEE_PER_BYTE);
            return false;
        }

        if self.mempool_size_bytes() + tx_size > MAX_MEMPOOL_SIZE_BYTES {
            self.evict_lowest_fee_transactions(tx_size);
        }

        if self.mempool_size_bytes() + tx_size > MAX_MEMPOOL_SIZE_BYTES {
            return false;
        }

        if self.mempool.len() >= rules::MAX_TX_PER_BLOCK {
            return false;
        }

        let now = (crate::utils::now_millis() / 1000) as u64;
        self.mempool_timestamps.insert(tx.txid.clone(), now);
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
        self.mempool_timestamps.clear();

        // Apply transactions first to compute the new state root.
        for tx in &txs {
            self.apply_transaction(tx)?;
        }
        self.credit_reward(&prev.mined_for, reward)?; // Note: mined_for should be the new block's miner

        // Compute the new state root.
        let state_root = hex::encode(self.state.root_hash().map_err(|e| e.to_string())?);

        let mut new_block = Block::new_for(
            next_index,
            prev.hash.clone(),
            subsidy,
            total_fees,
            self.nbits,
            txs.clone(),
            String::new(), // mined_for akan diisi oleh caller
            state_root,
        );
        if !pow::mine(&mut new_block, self.nbits) {
            return Err("mining failed: nonce exhausted".into());
        }

        let nonce = new_block.nonce;
        self.chain.push(new_block);
        self.adjust_difficulty();
        Ok(nonce)
    }

    pub fn adjust_difficulty(&mut self) {
        if spec::current_network() == spec::Network::Regtest {
            return;
        }

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
        return spec::current_network().genesis_nbits();
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
