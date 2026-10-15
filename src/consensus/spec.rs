//! Parameter ekonomi konsensus ZenaChain.
//!
//! Semua parameter di sini masuk ke `consensus_fingerprint`.

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

pub fn coin_name() -> &'static str { COIN_NAME }
pub fn symbol() -> &'static str { SYMBOL }
pub fn version() -> &'static str { VERSION }
pub fn protocol_version() -> u32 { PROTOCOL_VERSION }
pub fn hash_algo() -> &'static str { HASH_ALGO }
pub fn signature_algo() -> &'static str { SIGNATURE_ALGO }
pub fn decimals() -> u32 { DECIMALS }
pub fn satoshi() -> u64 { SATOSHI }
pub fn max_supply_zenac() -> u64 { MAX_SUPPLY_ZENAC }
pub fn max_supply() -> u64 { MAX_SUPPLY }
pub fn initial_reward_zenac() -> u64 { INITIAL_REWARD_ZENAC }
pub fn initial_reward() -> u64 { INITIAL_REWARD }
pub fn subsidy_halving_interval() -> u64 { SUBSIDY_HALVING_INTERVAL }
pub fn max_halvings() -> u32 { MAX_HALVINGS }
pub fn pow_target_spacing_secs() -> u64 { POW_TARGET_SPACING_SECS }
pub fn pow_target_timespan_secs() -> u64 { POW_TARGET_TIMESPAN_SECS }
pub fn miner_confirmation_window() -> u64 { MINER_CONFIRMATION_WINDOW }
pub fn difficulty_adjust_interval() -> u64 { DIFFICULTY_ADJUST_INTERVAL }
