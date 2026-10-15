use zenachain::block::Block;
use zenachain::blockchain::*;
use zenachain::pow::hash_meets_target;
use zenachain::transaction::Transaction;

#[test]
fn test_pow_target_prefix() {
    assert!(hash_meets_target("0000abc", 4));
    assert!(!hash_meets_target("000abc", 4));
}

#[test]
fn test_mining_menghasilkan_hash_valid() {
    let mut b = Block::new(1, "0".repeat(64), INITIAL_REWARD, vec![
        Transaction::new("a", "b", 1, 0),
    ]);
    zenachain::pow::mine(&mut b, 3);
    assert!(b.hash.starts_with("000"));
}
