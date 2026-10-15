use crate::block::Block;

/// Nilai nBits untuk difficulty 1 (target genesis ZenaChain).
/// Target = 00000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
pub const NBITS_ONE: u32 = 0x1e0ffff0;

/// Nilai nBits minimum yang diizinkan (difficulty tertinggi).
/// Setara dengan target 0000000000000000000000000000000000000000000000000000000000ffffff
pub const NBITS_MIN: u32 = 0x0300ffff;

/// Nilai nBits maksimum yang diizinkan (difficulty terendah).
pub const NBITS_MAX: u32 = 0x1e0ffff0;

/// Ubah nBits (format compact Bitcoin) menjadi target 256-bit big-endian.
pub fn nbits_to_target(nbits: u32) -> [u8; 32] {
    let exponent = (nbits >> 24) as usize;
    let mantissa = nbits & 0x00ffffff;

    let mut target = [0u8; 32];
    if exponent <= 3 {
        let value = mantissa >> (8 * (3 - exponent));
        let be = value.to_be_bytes();
        target[29] = be[1];
        target[30] = be[2];
        target[31] = be[3];
        return target;
    }

    let shift = 32 - exponent;
    let be = mantissa.to_be_bytes();
    if shift >= 3 {
        target[shift - 3] = be[1];
        target[shift - 2] = be[2];
        target[shift - 1] = be[3];
    }
    target
}

/// Bandingkan hash (hex string 64 char) dengan target 256-bit.
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

/// Cek apakah hash memenuhi nbits.
pub fn hash_meets_nbits(hash_hex: &str, nbits: u32) -> bool {
    let target = nbits_to_target(nbits);
    hash_le_target(hash_hex, &target)
}

/// Mining blok dengan target nbits.
/// Mengembalikan true bila blok berhasil ditambang.
pub fn mine(block: &mut Block, nbits: u32) -> bool {
    println!("[pow] mine: mencari nonce sampai hash memenuhi target nbits={:#010x}.", nbits);
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
