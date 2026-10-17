// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Proof-of-work consensus parameters for ZenaChain.

/// Genesis nBits. This value is the same as Litecoin: 0x1e0ffff0.
pub const GENESIS_NBITS: u32 = 0x1e0ffff0;

/// The proof-of-work limit. This value is the same as Litecoin:
/// 00000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
pub const POW_LIMIT: &str = "00000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

pub fn genesis_nbits() -> u32 {
    GENESIS_NBITS
}
pub fn pow_limit() -> &'static str {
    POW_LIMIT
}
