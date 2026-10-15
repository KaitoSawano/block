use crate::block::Block;

pub fn hash_meets_target(hash: &str, difficulty: usize) -> bool {
    hash.starts_with(&"0".repeat(difficulty))
}

/// Mining blok. Mengembalikan true bila blok berhasil ditambang.
/// Mengembalikan false bila nonce sudah mencapai u64::MAX tanpa hasil.
pub fn mine(block: &mut Block, difficulty: usize) -> bool {
    println!("[pow] mine: mencari nonce sampai hash punya prefix nol sejumlah difficulty.");
    let target = "0".repeat(difficulty);
    loop {
        block.hash = block.calculate_hash();
        if block.hash.starts_with(&target) {
            return true;
        }
        if block.nonce == u64::MAX {
            println!("[pow] nonce habis, mining gagal untuk blok ini.");
            return false;
        }
        block.nonce = block.nonce.wrapping_add(1);
    }
}
