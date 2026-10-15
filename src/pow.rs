use crate::block::Block;

/// Nilai nBits genesis ZenaChain. Sama dengan Bitcoin: 0x1d00ffff.
pub const GENESIS_NBITS: u32 = 0x1d00ffff;

/// powLimit ZenaChain. Sama dengan Bitcoin:
/// 00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffff
pub const POW_LIMIT: &str = "00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

/// Ubah nBits format compact menjadi target 256-bit big-endian.
/// Implementasi persis seperti Bitcoin/Litecoin `SetCompact`.
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

/// Bandingkan hash (hex 64 char) dengan target 256-bit.
/// Return true bila hash <= target.
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

/// Cek hash memenuhi target dari nbits.
pub fn hash_meets_nbits(hash_hex: &str, nbits: u32) -> bool {
    let target = nbits_to_target(nbits);
    hash_le_target(hash_hex, &target)
}

/// Mining blok. Loop sampai hash memenuhi target nbits.
pub fn mine(block: &mut Block, nbits: u32) -> bool {
    println!(
        "[pow] mine: mencari nonce sampai hash memenuhi target nbits={:#010x}.",
        nbits
    );
    let target = nbits_to_target(nbits);
    loop {
        block.hash = block.calculate_hash();
        if hash_le_target(&block.hash, &target) {
            return true;
        }
        if block.nonce == u64::MAX {
            println!("[pow] nonce habis, mining gagal untuk blok ini.");
            return false;
        }
        block.nonce = block.nonce.wrapping_add(1);
    }
}
