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
    pub const DECIMALS: u32 = 8;

    pub const SATOSHI: u64 = 100_000_000;

    pub const MAX_SUPPLY_ZENAC: u64 = 284_000_000;
    pub const MAX_SUPPLY: u64 = MAX_SUPPLY_ZENAC * SATOSHI;

    pub const INITIAL_REWARD_ZENAC: u64 = 50;
    pub const INITIAL_REWARD: u64 = INITIAL_REWARD_ZENAC * SATOSHI;

    pub const HALVING_INTERVAL_BLOCKS: u64 = 2_628_000;
    pub const MAX_HALVINGS: u32 = 64;

    pub const TARGET_BLOCK_TIME_SECS: u64 = 60;
    pub const DIFFICULTY_ADJUST_INTERVAL: u64 = 2016;
    pub const INITIAL_DIFFICULTY: usize = 3;
    pub const MIN_DIFFICULTY: usize = 1;
    pub const MAX_DIFFICULTY: usize = 12;

    pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
    pub const GENESIS_SECRET: &str = "ID Today 15/Oct/2026";
    pub const GENESIS_TIMESTAMP: u128 = 1_792_012_800_000;
}

pub(crate) use spec::{
    DIFFICULTY_ADJUST_INTERVAL, GENESIS_MESSAGE, GENESIS_SECRET, GENESIS_TIMESTAMP,
    HALVING_INTERVAL_BLOCKS, HASH_ALGO, INITIAL_DIFFICULTY, INITIAL_REWARD,
    INITIAL_REWARD_ZENAC, MAX_DIFFICULTY, MAX_HALVINGS, MAX_SUPPLY, MAX_SUPPLY_ZENAC,
    MIN_DIFFICULTY, SATOSHI, TARGET_BLOCK_TIME_SECS,
};

pub fn coin_name() -> &'static str {
    println!("[blockchain] coin_name: nama resmi coin.");
    spec::COIN_NAME
}

pub fn symbol() -> &'static str {
    println!("[blockchain] symbol: simbol ticker coin.");
    spec::SYMBOL
}

pub fn version() -> &'static str {
    println!("[blockchain] version: versi perangkat lunak.");
    spec::VERSION
}

pub fn protocol_version() -> u32 {
    println!("[blockchain] protocol_version: versi aturan konsensus.");
    spec::PROTOCOL_VERSION
}

pub fn hash_algo() -> &'static str {
    println!("[blockchain] hash_algo: algoritma hash yang dipakai untuk PoW dan identitas blok.");
    spec::HASH_ALGO
}

pub fn decimals() -> u32 {
    println!("[blockchain] decimals: jumlah angka di belakang koma untuk tampilan coin.");
    spec::DECIMALS
}

pub fn satoshi() -> u64 {
    println!("[blockchain] satoshi: unit terkecil, 1 ZENAC = nilai ini.");
    spec::SATOSHI
}

pub fn max_supply_zenac() -> u64 {
    println!("[blockchain] max_supply_zenac: batas maksimum supply dalam satuan ZENAC.");
    spec::MAX_SUPPLY_ZENAC
}

pub fn max_supply() -> u64 {
    println!("[blockchain] max_supply: batas maksimum supply dalam satuan terkecil.");
    spec::MAX_SUPPLY
}

pub fn initial_reward_zenac() -> u64 {
    println!("[blockchain] initial_reward_zenac: hadiah blok awal dalam ZENAC.");
    spec::INITIAL_REWARD_ZENAC
}

pub fn initial_reward() -> u64 {
    println!("[blockchain] initial_reward: hadiah blok awal dalam satuan terkecil.");
    spec::INITIAL_REWARD
}

pub fn halving_interval_blocks() -> u64 {
    println!("[blockchain] halving_interval_blocks: jarak blok antar halving.");
    spec::HALVING_INTERVAL_BLOCKS
}

pub fn max_halvings() -> u32 {
    println!("[blockchain] max_halvings: jumlah maksimum peristiwa halving.");
    spec::MAX_HALVINGS
}

pub fn target_block_time_secs() -> u64 {
    println!("[blockchain] target_block_time_secs: target waktu antar blok.");
    spec::TARGET_BLOCK_TIME_SECS
}

pub fn difficulty_adjust_interval() -> u64 {
    println!("[blockchain] difficulty_adjust_interval: jarak blok penyesuaian difficulty.");
    spec::DIFFICULTY_ADJUST_INTERVAL
}

pub fn initial_difficulty() -> usize {
    println!("[blockchain] initial_difficulty: difficulty awal saat genesis.");
    spec::INITIAL_DIFFICULTY
}

pub fn min_difficulty() -> usize {
    println!("[blockchain] min_difficulty: difficulty terendah yang diizinkan.");
    spec::MIN_DIFFICULTY
}

pub fn max_difficulty() -> usize {
    println!("[blockchain] max_difficulty: difficulty tertinggi yang diizinkan.");
    spec::MAX_DIFFICULTY
}

pub fn genesis_message() -> &'static str {
    println!("[blockchain] genesis_message: pesan publik pada genesis block.");
    spec::GENESIS_MESSAGE
}

pub fn genesis_secret() -> &'static str {
    println!("[blockchain] genesis_secret: pesan rahasia yang tertanam di genesis block.");
    spec::GENESIS_SECRET
}

pub fn genesis_timestamp() -> u128 {
    println!("[blockchain] genesis_timestamp: waktu genesis dalam milidetik Unix.");
    spec::GENESIS_TIMESTAMP
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub mempool: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        println!("[blockchain] new: membuat instance blockchain dengan genesis block.");
        Blockchain {
            chain: vec![Block::genesis()],
            difficulty: spec::INITIAL_DIFFICULTY,
            mempool: Vec::new(),
        }
    }

    pub fn height(&self) -> u64 {
        self.chain.last().unwrap().index
    }

    pub fn circulating_supply(&self) -> u64 {
        self.chain.iter().map(|b| b.reward).sum()
    }

    pub fn calculate_reward(height: u64) -> u64 {
        let halvings = (height / spec::HALVING_INTERVAL_BLOCKS) as u32;
        if halvings >= spec::MAX_HALVINGS {
            return 0;
        }
        spec::INITIAL_REWARD >> halvings
    }

    pub fn current_era(&self) -> u64 {
        self.height() / spec::HALVING_INTERVAL_BLOCKS
    }

    pub fn blocks_until_halving(&self) -> u64 {
        let next = (self.current_era() + 1) * spec::HALVING_INTERVAL_BLOCKS;
        next.saturating_sub(self.height())
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        println!("[blockchain] add_transaction: menambahkan transaksi ke mempool bila valid.");
        if tx.is_valid() {
            self.mempool.push(tx);
            true
        } else {
            false
        }
    }

    pub fn mine_pending(&mut self) -> Result<u64, String> {
        println!("[blockchain] mine_pending: menambang blok baru dari transaksi mempool.");
        let prev = self.chain.last().unwrap().clone();
        let next_index = prev.index + 1;
        let reward = Self::calculate_reward(next_index);

        if self.circulating_supply() + reward > spec::MAX_SUPPLY {
            return Err("max supply reached".into());
        }

        let txs = std::mem::take(&mut self.mempool);
        let mut new_block = Block::new(next_index, prev.hash.clone(), reward, txs);
        pow::mine(&mut new_block, self.difficulty);
        let nonce = new_block.nonce;
        self.chain.push(new_block);
        Ok(nonce)
    }

    pub fn adjust_difficulty(&mut self) {
        let height = self.height();
        if height == 0 || height % spec::DIFFICULTY_ADJUST_INTERVAL != 0 {
            return;
        }

        println!("[blockchain] adjust_difficulty: menyesuaikan difficulty berdasarkan waktu aktual blok.");
        let interval = spec::DIFFICULTY_ADJUST_INTERVAL as usize;
        let last = &self.chain[height as usize];
        let first = &self.chain[height as usize - interval];

        let actual_time_ms = last.timestamp.saturating_sub(first.timestamp) as u64;
        let actual_time_secs = actual_time_ms / 1000;
        let expected_time_secs = spec::TARGET_BLOCK_TIME_SECS * spec::DIFFICULTY_ADJUST_INTERVAL;

        if actual_time_secs < expected_time_secs / 2 {
            if self.difficulty < spec::MAX_DIFFICULTY {
                self.difficulty += 1;
            }
        } else if actual_time_secs > expected_time_secs * 2 {
            if self.difficulty > spec::MIN_DIFFICULTY {
                self.difficulty -= 1;
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        println!("[blockchain] is_valid: memverifikasi hash, prev_hash, reward, dan PoW tiap blok.");
        for i in 1..self.chain.len() {
            let cur = &self.chain[i];
            let prev = &self.chain[i - 1];

            if cur.hash != cur.calculate_hash() {
                return false;
            }
            if cur.previous_hash != prev.hash {
                return false;
            }
            if cur.reward != Self::calculate_reward(cur.index) {
                return false;
            }
            if !pow::hash_meets_target(&cur.hash, self.difficulty) {
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
