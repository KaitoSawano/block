//! Konstanta aturan validasi konsensus ZenaChain.

pub const COINBASE_MATURITY: u64 = 100;
pub const ADDRESS_PREFIX: &str = "ZENA";
pub const ADDRESS_HEX_LEN: usize = 40;
pub const MAX_TX_PER_BLOCK: usize = 10_000;
pub const MAX_INPUTS_PER_TX: usize = 1_000;
pub const MAX_OUTPUTS_PER_TX: usize = 1_000;
pub const MAX_NOTE_LEN: usize = 256;
pub const ZERO_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
pub const BLOCK_VERSION: u32 = 1;

pub fn max_tx_per_block() -> usize { MAX_TX_PER_BLOCK }
