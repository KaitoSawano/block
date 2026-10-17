// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use zenachain::blockchain::{self, Blockchain};

#[test]
fn reward_era_0() {
    let r = Blockchain::calculate_reward(0);
    assert_eq!(r, blockchain::initial_reward());
}

#[test]
fn reward_era_1() {
    let r = Blockchain::calculate_reward(blockchain::halving_interval_blocks());
    assert_eq!(r, blockchain::initial_reward() / 2);
}

#[test]
fn reward_era_2() {
    let r = Blockchain::calculate_reward(blockchain::halving_interval_blocks() * 2);
    assert_eq!(r, blockchain::initial_reward() / 4);
}

#[test]
fn reward_batas_halving() {
    let r = Blockchain::calculate_reward(blockchain::halving_interval_blocks() * 100);
    assert_eq!(r, 0);
}
