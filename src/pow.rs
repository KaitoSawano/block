use crate::block::Block;

/// Cek apakah hash memenuhi target difficulty (prefix nol).
pub fn hash_meets_target(hash: &str, difficulty: usize) -> bool {
    hash.starts_with(&"0".repeat(difficulty))
}

/// Mining loop: cari nonce sampai hash punya prefix "0" * difficulty.
pub fn mine(block: &mut Block, difficulty: usize) {
    let target = "0".repeat(difficulty);
    loop {
        block.hash = block.calculate_hash();
        if block.hash.starts_with(&target) {
            break;
        }
        block.nonce += 1;
    }
}
