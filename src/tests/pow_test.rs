// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use zenachain::block::Block;
use zenachain::blockchain;
use zenachain::pow::{hash_meets_nbits, mine, nbits_to_target};
use zenachain::transaction::Transaction;

#[test]
fn nbits_to_target_genesis() {
    let target = nbits_to_target(blockchain::genesis_nbits());
    // The genesis target has four leading zero bytes.
    assert_eq!(&target[..2], &[0u8, 0u8]);
    assert!(target[2] <= 0x0f);
}

#[test]
fn hash_meets_nbits_rejects_easy_hash() {
    // A hash that starts with 0xff does not meet the genesis target.
    let hash = "ff".repeat(32);
    assert!(!hash_meets_nbits(&hash, blockchain::genesis_nbits()));
}

#[test]
fn mining_produces_valid_hash() {
    let nbits = blockchain::genesis_nbits();
    let mut b = Block::new(
        1,
        "0".repeat(64),
        blockchain::initial_reward(),
        nbits,
        vec![Transaction::new("ZENA_alice", "ZENA_bob", 1, 0)],
    );
    let ok = mine(&mut b, nbits);
    assert!(ok);
    assert!(hash_meets_nbits(&b.hash, nbits));
}

#[test]
fn mining_increments_nonce() {
    let nbits = blockchain::genesis_nbits();
    let mut b = Block::new(
        1,
        "0".repeat(64),
        blockchain::initial_reward(),
        nbits,
        vec![Transaction::new("ZENA_alice", "ZENA_bob", 1, 0)],
    );
    let n0 = b.nonce;
    let ok = mine(&mut b, nbits);
    assert!(ok);
    assert!(b.nonce >= n0);
}
