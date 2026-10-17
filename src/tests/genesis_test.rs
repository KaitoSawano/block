// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use zenachain::block::Block;
use zenachain::blockchain;
use zenachain::consensus;

#[test]
fn genesis_secret_is_embedded() {
    let g = Block::genesis();
    let secret_tx = g
        .transactions
        .iter()
        .find(|tx| tx.from == "GENESIS")
        .expect("secret transaction is missing");
    assert_eq!(secret_tx.note, blockchain::genesis_secret());
}

#[test]
fn genesis_hash_binds_message() {
    let mut g1 = Block::genesis();
    let h1 = g1.calculate_hash();
    g1.transactions[1].note = "CHANGED".into();
    let h2 = g1.calculate_hash();
    assert_ne!(h1, h2);
}

#[test]
fn genesis_timestamp_matches_consensus() {
    let g = Block::genesis();
    assert_eq!(g.timestamp, blockchain::genesis_timestamp());
}

#[test]
fn genesis_index_is_zero() {
    let g = Block::genesis();
    assert_eq!(g.index, 0);
}

#[test]
fn genesis_previous_hash_is_zero() {
    let g = Block::genesis();
    assert_eq!(g.previous_hash, consensus::rules::ZERO_HASH);
}

#[test]
fn genesis_hash_matches_expected() {
    let g = Block::genesis();
    assert_eq!(g.hash, consensus::genesis::EXPECTED_GENESIS_HASH);
}

#[test]
fn genesis_merkle_root_matches_expected() {
    let g = Block::genesis();
    assert_eq!(
        g.merkle_root,
        consensus::genesis::EXPECTED_GENESIS_MERKLE_ROOT
    );
}

#[test]
fn genesis_nonce_matches_expected() {
    let g = Block::genesis();
    assert_eq!(g.nonce, consensus::genesis::EXPECTED_GENESIS_NONCE);
}
