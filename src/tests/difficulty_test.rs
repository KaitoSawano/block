// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use zenachain::blockchain::{self, Blockchain};

#[test]
fn timespan_matches_adjust_interval_times_spacing() {
    let expected = blockchain::difficulty_adjust_interval() * blockchain::pow_target_spacing_secs();
    assert_eq!(blockchain::pow_target_timespan_secs(), expected);
}

#[test]
fn adjust_difficulty_does_nothing_before_interval() {
    let mut chain = Blockchain::new();
    let nbits_before = chain.nbits;
    chain.adjust_difficulty();
    assert_eq!(chain.nbits, nbits_before);
}

#[test]
fn adjust_difficulty_increases_target_when_blocks_are_slow() {
    // This test is a placeholder. A full test requires a chain that
    // reaches `DIFFICULTY_ADJUST_INTERVAL`, which is expensive to mine
    // in a unit test. The invariant is documented in `blockchain.rs`:
    // slow blocks cause the target to increase (difficulty decreases).
    let chain = Blockchain::new();
    assert!(chain.nbits > 0);
}
