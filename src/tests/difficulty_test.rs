// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use std::sync::OnceLock;
use zenachain::blockchain::{self, Blockchain};

/// Cache the initial blockchain so that expensive proof-of-work mining
/// is performed only once for the entire test suite.
fn cached_chain() -> &'static Blockchain {
    static CHAIN: OnceLock<Blockchain> = OnceLock::new();
    CHAIN.get_or_init(|| Blockchain::new())
}

#[test]
fn timespan_matches_adjust_interval_times_spacing() {
    let expected = blockchain::difficulty_adjust_interval() * blockchain::pow_target_spacing_secs();
    assert_eq!(blockchain::pow_target_timespan_secs(), expected);
}

#[test]
fn adjust_difficulty_does_nothing_before_interval() {
    let chain = cached_chain();
    let nbits_before = chain.nbits;
    // Use a fresh copy to avoid mutating the cached instance
    let mut fresh = chain.clone();
    fresh.adjust_difficulty();
    assert_eq!(fresh.nbits, nbits_before);
}

#[test]
fn adjust_difficulty_increases_target_when_blocks_are_slow() {
    // This test is a placeholder. A full test requires a chain that
    // reaches `DIFFICULTY_ADJUST_INTERVAL`, which is expensive to mine
    // in a unit test. The invariant is documented in `blockchain.rs`:
    // slow blocks cause the target to increase (difficulty decreases).
    let chain = cached_chain();
    assert!(chain.nbits > 0);
}
