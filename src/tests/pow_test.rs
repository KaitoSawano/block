use zenachain::block::Block;
use zenachain::blockchain;
use zenachain::pow::{hash_meets_target, mine};
use zenachain::transaction::Transaction;

#[test]
fn target_prefix() {
    assert!(hash_meets_target("0000abc", 4));
    assert!(!hash_meets_target("000abc", 4));
}

#[test]
fn mining_hash_valid() {
    let mut b = Block::new(
        1,
        "0".repeat(64),
        blockchain::initial_reward(),
        vec![Transaction::new("a", "b", 1, 0)],
    );
    mine(&mut b, 3);
    assert!(b.hash.starts_with("000"));
}

#[test]
fn mining_nonce_naik() {
    let mut b = Block::new(
        1,
        "0".repeat(64),
        blockchain::initial_reward(),
        vec![Transaction::new("a", "b", 1, 0)],
    );
    let n0 = b.nonce;
    mine(&mut b, 2);
    assert!(b.nonce >= n0);
}
