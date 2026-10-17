use crate::block::Block;
use std::sync::atomic::{AtomicBool, Ordering};

pub use crate::consensus::pow::{GENESIS_NBITS, POW_LIMIT};

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

pub fn hash_meets_nbits(hash_hex: &str, nbits: u32) -> bool {
    let target = nbits_to_target(nbits);
    hash_le_target(hash_hex, &target)
}

/// Mine blok sampai hash memenuhi target atau `interrupt` di-set.
/// Return `true` jika berhasil, `false` jika diinterupsi atau nonce habis.
pub fn mine(block: &mut Block, nbits: u32) -> bool {
    mine_interruptible(block, nbits, None)
}

/// Versi `mine` yang bisa diinterupsi lewat `AtomicBool`.
pub fn mine_interruptible(block: &mut Block, nbits: u32, interrupt: Option<&AtomicBool>) -> bool {
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
