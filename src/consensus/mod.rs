//! Modul konsensus ZenaChain.
//!
//! Semua parameter yang mengikat aturan validasi chain berada di sini.
//! Kalau ada parameter di sini yang diubah, genesis hash berubah,
//! dan seluruh chain lama menjadi tidak valid.
//!
//! Parameter jaringan (peer bootstrap, DNS seed, port P2P) TIDAK
//! di sini. Itu ada di `src/p2p/peers.rs`.

pub mod genesis;
pub mod pow;
pub mod rules;
pub mod spec;

use crate::utils::sha3_256_hex;

pub fn consensus_fingerprint() -> String {
    let payload = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        spec::HASH_ALGO,
        spec::SIGNATURE_ALGO,
        spec::DECIMALS,
        spec::SATOSHI,
        spec::MAX_SUPPLY_ZENAC,
        spec::MAX_SUPPLY,
        spec::INITIAL_REWARD_ZENAC,
        spec::INITIAL_REWARD,
        spec::SUBSIDY_HALVING_INTERVAL,
        spec::MAX_HALVINGS,
        spec::POW_TARGET_SPACING_SECS,
        spec::POW_TARGET_TIMESPAN_SECS,
        spec::MINER_CONFIRMATION_WINDOW,
        spec::DIFFICULTY_ADJUST_INTERVAL,
        genesis::GENESIS_MESSAGE,
        genesis::GENESIS_SECRET,
        genesis::GENESIS_TIMESTAMP,
        genesis::GENESIS_OUTPUT_SCRIPT,
        pow::GENESIS_NBITS,
        pow::POW_LIMIT,
        rules::COINBASE_MATURITY,
        rules::ADDRESS_PREFIX,
        rules::ADDRESS_HEX_LEN,
        rules::MAX_TX_PER_BLOCK,
        rules::MAX_INPUTS_PER_TX,
        rules::MAX_NOTE_LEN,
        rules::BLOCK_VERSION,
        rules::ZERO_HASH,
    );
    sha3_256_hex(payload.as_bytes())
}

pub fn genesis_coinbase_txid() -> String {
    let fp = consensus_fingerprint();
    let payload = format!(
        "genesis|{}|{}|{}",
        fp,
        genesis::GENESIS_TIMESTAMP,
        pow::GENESIS_NBITS
    );
    sha3_256_hex(payload.as_bytes())
}

pub use genesis::genesis_message;
pub use genesis::genesis_output_script;
pub use genesis::genesis_secret;
pub use genesis::genesis_timestamp;
pub use pow::genesis_nbits;
pub use pow::pow_limit;
pub use rules::max_tx_per_block;
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
