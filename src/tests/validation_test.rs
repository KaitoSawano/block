// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use zenachain::block::Block;
use zenachain::blockchain::Blockchain;
use zenachain::consensus::rules;
use zenachain::transaction::Transaction;
use zenachain::validation;

fn dummy_address(seed: char) -> String {
    format!("ZENA{}", seed.to_string().repeat(40))
}

#[test]
fn valid_chain_passes() {
    if std::env::var("ZENAC_SKIP_POW").is_ok() {
        return;
    }
    let chain = Blockchain::new();
    assert!(validation::validate_chain(&chain).is_ok());
}

#[test]
fn block_with_wrong_previous_hash_is_rejected() {
    let chain = Blockchain::new();
    let prev = chain.chain.last().unwrap().clone();
    let mut b = Block::new(
        prev.index + 1,
        "0".repeat(64),
        50_000_000_000,
        chain.nbits,
        Vec::new(),
    );
    b.hash = b.calculate_hash();
    let err = validation::validate_block(&chain, &prev, &b);
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().code, "E035");
}

#[test]
fn block_with_wrong_index_is_rejected() {
    let chain = Blockchain::new();
    let prev = chain.chain.last().unwrap().clone();
    let mut b = Block::new(
        prev.index + 2,
        prev.hash.clone(),
        50_000_000_000,
        chain.nbits,
        Vec::new(),
    );
    b.hash = b.calculate_hash();
    let err = validation::validate_block(&chain, &prev, &b);
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().code, "E036");
}

#[test]
fn transaction_with_zero_amount_is_rejected() {
    let chain = Blockchain::new();
    let from = dummy_address('a');
    let to = dummy_address('b');
    let tx = Transaction::new(&from, &to, 0, 0);
    let err = validation::validate_transaction(&chain, &tx);
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().code, "E005");
}

#[test]
fn transaction_with_self_send_is_rejected() {
    let chain = Blockchain::new();
    let addr = dummy_address('a');
    let tx = Transaction::new(&addr, &addr, 1, 0);
    let err = validation::validate_transaction(&chain, &tx);
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().code, "E004");
}

#[test]
fn address_format_validation() {
    assert!(validation::is_valid_address(&format!("ZENA{}", "a".repeat(40))));
    assert!(!validation::is_valid_address("ZENA123"));
    assert!(!validation::is_valid_address(&format!("XXXX{}", "a".repeat(40))));
}

#[test]
fn zero_hash_constant_is_correct() {
    assert_eq!(rules::ZERO_HASH.len(), 64);
    assert!(rules::ZERO_HASH.chars().all(|c| c == '0'));
}
