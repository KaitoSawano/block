use crate::block::Block;
use crate::pow;
use crate::transaction::Transaction;

// ============================================================
//  PARAMETER ZENACHAIN (ZENAC)
// ============================================================
pub const COIN_NAME: &str                 = "ZenaChain";
pub const SYMBOL: &str                    = "ZENAC";
pub const DECIMALS: u32                   = 8;
pub const SATOSHI: u64                    = 100_000_000;

pub const HASH_ALGO: &str                 = "SHA3-256";

pub const MAX_SUPPLY_ZENAC: u64           = 284_000_000;
pub const MAX_SUPPLY: u64                 = MAX_SUPPLY_ZENAC * SATOSHI;

pub const INITIAL_REWARD_ZENAC: u64       = 50;
pub const INITIAL_REWARD: u64             = INITIAL_REWARD_ZENAC * SATOSHI;

pub const HALVING_INTERVAL_BLOCKS: u64    = 2_628_000;
pub const MAX_HALVINGS: u32               = 64;

pub const TARGET_BLOCK_TIME_SECS: u64     = 60;
pub const DIFFICULTY_ADJUST_INTERVAL: u64 = 2016;
pub const INITIAL_DIFFICULTY: usize       = 3;

// ---- Genesis ----
pub const GENESIS_MESSAGE: &str = "ZenaChain Genesis - 2026";
pub const GENESIS_SECRET: &str  = "ID Today 15/Oct/2026";
pub const GENESIS_TIMESTAMP: u128 = 1_792_012_800_000; // 15 Oct 2026 00:00 UTC
// ============================================================

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub mempool: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            chain: vec![Block::genesis()],
            difficulty: INITIAL_DIFFICULTY,
            mempool: Vec::new(),
        }
    }

    pub fn height(&self) -> u64 {
        self.chain.last().unwrap().index
    }

    pub fn circulating_supply(&self) -> u64 {
        self.chain.iter().map(|b| b.reward).sum()
    }

    /// Reward = INITIAL_REWARD >> halvings.
    pub fn calculate_reward(height: u64) -> u64 {
        let halvings = (height / HALVING_INTERVAL_BLOCKS) as u32;
        if halvings >= MAX_HALVINGS {
            return 0;
        }
        INITIAL_REWARD >> halvings
    }

    pub fn current_era(&self) -> u64 {
        self.height() / HALVING_INTERVAL_BLOCKS
    }

    pub fn blocks_until_halving(&self) -> u64 {
        let next = (self.current_era() + 1) * HALVING_INTERVAL_BLOCKS;
        next.saturating_sub(self.height())
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        if tx.is_valid() {
            self.mempool.push(tx);
            true
        } else {
            false
        }
    }

    pub fn mine_pending(&mut self) -> Result<(), String> {
        let prev = self.chain.last().unwrap().clone();
        let next_index = prev.index + 1;
        let reward = Self::calculate_reward(next_index);

        if self.circulating_supply() + reward > MAX_SUPPLY {
            return Err("Max supply ZENAC tercapai".into());
        }

        let txs = std::mem::take(&mut self.mempool);
        let mut new_block = Block::new(next_index, prev.hash.clone(), reward, txs);
        pow::mine(&mut new_block, self.difficulty);
        self.chain.push(new_block);
        Ok(())
    }

    pub fn is_valid(&self) -> bool {
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

    pub fn info(&self) -> String {
        format!(
            "{} ({})\n\
             Hash algo         : {}\n\
             Height            : {}\n\
             Era halving       : {}\n\
             Blok ke halving   : {}\n\
             Reward sekarang   : {} ZENAC\n\
             Supply            : {} / {} ZENAC\n\
             Difficulty        : {}",
            COIN_NAME, SYMBOL,
            HASH_ALGO,
            self.height(),
            self.current_era(),
            self.blocks_until_halving(),
            Self::calculate_reward(self.height() + 1) / SATOSHI,
            self.circulating_supply() / SATOSHI,
            MAX_SUPPLY_ZENAC,
            self.difficulty,
        )
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}
