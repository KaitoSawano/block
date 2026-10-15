//! Parameter PoW konsensus ZenaChain.

/// nBits genesis. Sama dengan Litecoin: 0x1e0ffff0.
pub const GENESIS_NBITS: u32 = 0x1e0ffff0;

/// powLimit. Sama dengan Litecoin:
/// 00000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
pub const POW_LIMIT: &str =
    "00000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

pub fn genesis_nbits() -> u32 { GENESIS_NBITS }
pub fn pow_limit() -> &'static str { POW_LIMIT }
