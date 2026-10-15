use crate::block::Block;

pub fn hash_meets_target(hash: &str, difficulty: usize) -> bool {
    hash.starts_with(&"0".repeat(difficulty))
}

pub fn mine(block: &mut Block, difficulty: usize) {
    println!("[pow] mine: mencari nonce sampai hash punya prefix nol sejumlah difficulty.");
    let target = "0".repeat(difficulty);
    loop {
        block.hash = block.calculate_hash();
        if block.hash.starts_with(&target) {
            break;
        }
        block.nonce += 1;
    }
}
