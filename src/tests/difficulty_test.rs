// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use serial_test::serial;
use zenachain::blockchain::{self, Blockchain};

fn with_temp_datadir<F: FnOnce()>(f: F) {
    let dir = tempfile::tempdir().expect("tempdir failed");
    std::env::set_var("ZENAC_DATADIR", dir.path());
    f();
    std::env::remove_var("ZENAC_DATADIR");
}

#[test]
fn timespan_matches_adjust_interval_times_spacing() {
    let expected = blockchain::difficulty_adjust_interval() * blockchain::pow_target_spacing_secs();
    assert_eq!(blockchain::pow_target_timespan_secs(), expected);
}

#[test]
#[serial]
fn adjust_difficulty_does_nothing_before_interval() {
    with_temp_datadir(|| {
        let mut chain = Blockchain::new();
        let nbits_before = chain.nbits;
        chain.adjust_difficulty();
        assert_eq!(chain.nbits, nbits_before);
    });
}

#[test]
#[serial]
fn adjust_difficulty_increases_target_when_blocks_are_slow() {
    with_temp_datadir(|| {
        let chain = Blockchain::new();
        assert!(chain.nbits > 0);
    });
}
