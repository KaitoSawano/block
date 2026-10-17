// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use zenachain::block::Block;
use zenachain::blockchain;

#[test]
fn genesis_secret_tertanam() {
    let g = Block::genesis();
    let secret_tx = g
        .transactions
        .iter()
        .find(|tx| tx.from == "GENESIS")
        .expect("secret tx missing");
    assert_eq!(secret_tx.note, "ID Today 15/Oct/2026");
}

#[test]
fn genesis_hash_mengikat_pesan() {
    let mut g1 = Block::genesis();
    let h1 = g1.calculate_hash();
    g1.transactions[1].note = "CHANGED".into();
    let h2 = g1.calculate_hash();
    assert_ne!(h1, h2);
}

#[test]
fn genesis_timestamp() {
    let g = Block::genesis();
    assert_eq!(g.timestamp, blockchain::genesis_timestamp());
}

#[test]
fn genesis_index_nol() {
    let g = Block::genesis();
    assert_eq!(g.index, 0);
}

#[test]
fn genesis_previous_hash_nol() {
    let g = Block::genesis();
    assert_eq!(g.previous_hash, "0".repeat(64));
}
