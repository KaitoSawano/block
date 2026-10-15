use crate::block::Block;
use crate::pow;
use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};

mod spec {
    pub const COIN_NAME: &str = "ZenaChain";
    pub const SYMBOL: &str = "ZENAC";
    pub const VERSION: &str = "0.1.0";
    pub const PROTOCOL_VERSION: u32 = 1;
    pub const HASH_ALGO: &str = "SHA3-256";
    pub const SIGNATURE_ALGO: &str = "secp256k1-ECDSA";
    pub const DECIMALS: u32 = 8;

    pub const SATOSHI: u64 = 100_000_000;

    // Supply maksimum: 28.000.000.000 ZENAC
    pub const MAX_SUPPLY_ZENAC: u64 = 28_000_000_000;
    pub const MAX_SUPPLY: u64 = MAX_SUPPLY_ZENAC * SATOSHI;

    // Reward awal: 50 ZENAC
    pub const INITIAL_REWARD_ZENAC: u64 = 50;
    pub const INITIAL_REWARD: u64 = INITIAL_REWARD_ZENAC * SATOSHI;

    // Halving: setiap 280.000.000 blok.
    // Total supply = 2 × 280.000.000 × 50 = 28.000.000.000 ZENAC.
    pub const SUBSIDY_HALVING_INTERVAL: u64 = 280_000_000;
    pub const MAX_HALVINGS: u32 = 64;

    // Target spacing: 15 detik.
    pub const POW_TARGET_SPACING_SECS: u64 = 15;

    // Target timespan: 3 hari.
    pub const POW_TARGET_TIMESPAN_SECS: u64 = 259_200;

    // Miner confirmation window: 259200 / 15 * 4 = 69.120 blok.
    pub const MINER_CONFIRMATION_WINDOW: u64 = 69_120;

    // Interval penyesuaian nBits: sama dengan confirmation window.
    pub const DIFFICULTY_ADJUST_INTERVAL: u64 = 69_120;

    pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
    pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
    pub const GENESIS_TIMESTAMP: u128 = 1_792_012_800_000;

    pub const MAX_TX_PER_BLOCK: usize = 10_000;
}

pub mod checkpoint {
    pub struct Checkpoint {
        pub height: u64,
        pub hash: &'static str,
    }

    pub const CHECKPOINTS: &[Checkpoint] = &[
        Checkpoint {
            height: 0,
            hash: "0000000000000000000000000000000000000000000000000000000000000000",
        },
    ];

    pub fn find(height: u64) -> Option<&'static str> {
        CHECKPOINTS
            .iter()
            .find(|c| c.height == height)
            .map(|c| c.hash)
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

pub fn coin_name() -> &'static str { spec::COIN_NAME }
pub fn symbol() -> &'static str { spec::SYMBOL }
pub fn version() -> &'static str { spec::VERSION }
pub fn protocol_version() -> u32 { spec::PROTOCOL_VERSION }
pub fn hash_algo() -> &'static str { spec::HASH_ALGO }
pub fn signature_algo() -> &'static str { spec::SIGNATURE_ALGO }
pub fn decimals() -> u32 { spec::DECIMALS }
pub fn satoshi() -> u64 { spec::SATOSHI }
pub fn max_supply_zenac() -> u64 { spec::MAX_SUPPLY_ZENAC }
pub fn max_supply() -> u64 { spec::MAX_SUPPLY }
pub fn initial_reward_zenac() -> u64 { spec::INITIAL_REWARD_ZENAC }
pub fn initial_reward() -> u64 { spec::INITIAL_REWARD }
pub fn subsidy_halving_interval() -> u64 { spec::SUBSIDY_HALVING_INTERVAL }
pub fn max_halvings() -> u32 { spec::MAX_HALVINGS }
pub fn pow_target_spacing_secs() -> u64 { spec::POW_TARGET_SPACING_SECS }
pub fn pow_target_timespan_secs() -> u64 { spec::POW_TARGET_TIMESPAN_SECS }
pub fn miner_confirmation_window() -> u64 { spec::MINER_CONFIRMATION_WINDOW }
pub fn difficulty_adjust_interval() -> u64 { spec::DIFFICULTY_ADJUST_INTERVAL }
pub fn genesis_message() -> &'static str { spec::GENESIS_MESSAGE }
pub fn genesis_secret() -> &'static str { spec::GENESIS_SECRET }
pub fn genesis_timestamp() -> u128 { spec::GENESIS_TIMESTAMP }
pub fn max_tx_per_block() -> usize { spec::MAX_TX_PER_BLOCK }
pub fn pow_limit() -> &'static str { pow::POW_LIMIT }
pub fn genesis_nbits() -> u32 { pow::GENESIS_NBITS }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub nbits: u32,
    pub mempool: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            chain: vec![Block::genesis()],
            nbits: pow::GENESIS_NBITS,
            mempool: Vec::new(),
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

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        if !tx.is_valid() {
            return false;
        }
        if self.mempool.len() >= spec::MAX_TX_PER_BLOCK {
            return false;
        }
        self.mempool.push(tx);
        true
    }

    pub fn mine_pending(&mut self) -> Result<u64, String> {
        let prev = self.chain.last().unwrap().clone();
        let next_index = prev.index + 1;
        let reward = Self::calculate_reward(next_index);

        if self.circulating_supply().saturating_add(reward) > spec::MAX_SUPPLY {
            return Err("max supply reached".into());
        }

        let txs = std::mem::take(&mut self.mempool);
        let mut new_block =
            Block::new(next_index, prev.hash.clone(), reward, self.nbits, txs);
        if !pow::mine(&mut new_block, self.nbits) {
            return Err("mining gagal: nonce habis".into());
        }
        let nonce = new_block.nonce;
        self.chain.push(new_block);
        Ok(nonce)
    }

    pub fn adjust_difficulty(&mut self) {
        let height = self.height();
        if height == 0 || height % spec::DIFFICULTY_ADJUST_INTERVAL != 0 {
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
        let scale = (expected_time_secs as u128) * 256 / (actual_time_secs as u128);
        let new_target = scale_target(&current_target, scale);
        let new_nbits = target_to_nbits(&new_target);

        if new_nbits != self.nbits {
            println!(
                "[blockchain] adjust_difficulty: nbits {:#010x} -> {:#010x} pada height {}",
                self.nbits, new_nbits, height
            );
            self.nbits = new_nbits;
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.chain.is_empty() {
            return false;
        }

        let genesis = &self.chain[0];
        if genesis.index != 0 {
            return false;
        }
        if genesis.previous_hash != "0".repeat(64) {
            return false;
        }
        if !pow::hash_meets_nbits(&genesis.hash, genesis.nbits) {
            return false;
        }

        for i in 1..self.chain.len() {
            let cur = &self.chain[i];
            let prev = &self.chain[i - 1];

            if cur.index != prev.index + 1 {
                return false;
            }
            if cur.hash != cur.calculate_hash() {
                return false;
            }
            if cur.previous_hash != prev.hash {
                return false;
            }
            if cur.reward != Self::calculate_reward(cur.index) {
                return false;
            }
            if !pow::hash_meets_nbits(&cur.hash, cur.nbits) {
                return false;
            }
            if cur.transactions.len() > spec::MAX_TX_PER_BLOCK {
                return false;
            }
            if cur.timestamp < prev.timestamp {
                return false;
            }
            if !checkpoint::verify(cur.index, &cur.hash) {
                return false;
            }
            for tx in &cur.transactions {
                if !tx.verify_signature() {
                    return false;
                }
            }
        }

        true
    }

    pub fn check_against_checkpoints(chain: &[Block]) -> bool {
        for block in chain {
            if !checkpoint::verify(block.index, &block.hash) {
                return false;
            }
        }
        true
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
        return pow::GENESIS_NBITS;
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
