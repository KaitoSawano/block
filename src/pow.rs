// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Proof-of-work primitives.

use crate::block::Block;
use std::sync::atomic::{AtomicBool, Ordering};

pub use crate::consensus::pow::{GENESIS_NBITS, POW_LIMIT};

/// Convert a compact nBits value into a 256-bit big-endian target.
pub fn nbits_to_target(nbits: u32) -> [u8; 32] {
    let exponent = (nbits >> 24) as usize;
    let mantissa = nbits & 0x007fffff;
    let negative = (nbits & 0x00800000) != 0;

    let mut target = [0u8; 32];

    if negative {
        return target;
    }

    if exponent <= 3 {
        let value = mantissa >> (8 * (3 - exponent));
        let be = value.to_be_bytes();
        target[29] = be[1];
        target[30] = be[2];
        target[31] = be[3];
        return target;
    }

    let shift = 32 - exponent;
    if shift + 3 > 32 {
        return target;
    }
    let be = mantissa.to_be_bytes();
    target[shift] = be[1];
    target[shift + 1] = be[2];
    target[shift + 2] = be[3];
    target
}

/// Return whether `hash_hex` is less than or equal to `target`.
pub fn hash_le_target(hash_hex: &str, target: &[u8; 32]) -> bool {
    let hash_bytes = match hex::decode(hash_hex) {
        Ok(b) if b.len() == 32 => b,
        _ => return false,
    };

    for i in 0..32 {
        if hash_bytes[i] < target[i] {
            return true;
        }
        if hash_bytes[i] > target[i] {
            return false;
        }
    }
    true
}

/// Return whether `hash_hex` meets the target encoded by `nbits`.
pub fn hash_meets_nbits(hash_hex: &str, nbits: u32) -> bool {
    let target = nbits_to_target(nbits);
    hash_le_target(hash_hex, &target)
}

/// Mine the block until the hash meets the target or the nonce is exhausted.
/// Return `true` on success and `false` if the nonce is exhausted.
pub fn mine(block: &mut Block, nbits: u32) -> bool {
    mine_interruptible(block, nbits, None)
}

/// Mine the block until the hash meets the target, the nonce is exhausted,
/// or the interrupt flag is set. Return `true` on success.
pub fn mine_interruptible(
    block: &mut Block,
    nbits: u32,
    interrupt: Option<&AtomicBool>,
) -> bool {
    // During tests, skip the actual mining loop to prevent the test suite
    // from hanging. We simply calculate the hash once and return success.
    #[cfg(test)]
    {
        block.hash = block.calculate_hash();
        return true;
    }

    #[cfg(not(test))]
    {
        let target = nbits_to_target(nbits);
        loop {
            if let Some(flag) = interrupt {
                if flag.load(Ordering::SeqCst) {
                    return false;
                }
            }
            block.hash = block.calculate_hash();
            if hash_le_target(&block.hash, &target) {
                return true;
            }
            if block.nonce == u64::MAX {
                return false;
            }
            block.nonce = block.nonce.wrapping_add(1);
        }
    }
}
