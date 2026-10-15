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

    pub const MAX_SUPPLY_ZENAC: u64 = 28_000_000_000;
    pub const MAX_SUPPLY: u64 = MAX_SUPPLY_ZENAC * SATOSHI;

    pub const INITIAL_REWARD_ZENAC: u64 = 50;
    pub const INITIAL_REWARD: u64 = INITIAL_REWARD_ZENAC * SATOSHI;

    pub const SUBSIDY_HALVING_INTERVAL: u64 = 280_000_000;
    pub const MAX_HALVINGS: u32 = 64;

    pub const POW_TARGET_SPACING_SECS: u64 = 15;
    pub const POW_TARGET_TIMESPAN_SECS: u64 = 259_200;
    pub const MINER_CONFIRMATION_WINDOW: u64 = 69_120;
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
pub
