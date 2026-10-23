// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

use serial_test::serial;
use zenachain::block::Block;
use zenachain::blockchain::Blockchain;
use zenachain::consensus::rules;
use zenachain::transaction::Transaction;
use zenachain::validation;

fn dummy_address(seed: char) -> String {
    format!("ZENA{}", seed.to_string().repeat(40))
}

fn with_temp_datadir<F: FnOnce()>(f: F) {
    let dir = tempfile::tempdir().expect("tempdir failed");
    std::env::set_var("ZENAC_DATADIR", dir.path());
    f();
    std::env::remove_var("ZENAC_DATADIR");
}

#[test]
#[serial]
#[cfg(not(feature = "skip-pow"))]
fn valid_chain_passes() {
    with_temp_datadir(|| {
        let chain = Blockchain::new();
        assert!(validation::validate_chain(&chain).is_ok());
    });
}

#[test]
#[serial]
fn block_with_wrong_previous_hash_is_rejected() {
    with_temp_datadir(|| {
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
    });
}

#[test]
#[serial]
fn block_with_wrong_index_is_rejected() {
    with_temp_datadir(|| {
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
    });
}

#[test]
#[serial]
fn transaction_with_zero_amount_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new();
        let from = dummy_address('a');
        let to = dummy_address('b');
        let tx = Transaction::new(&from, &to, 0, 0, 0);
        let err = validation::validate_transaction(&chain, &tx);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().code, "E005");
    });
}

#[test]
#[serial]
fn transaction_with_self_send_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new();
        let addr = dummy_address('a');
        let tx = Transaction::new(&addr, &addr, 1, 0, 0);
        let err = validation::validate_transaction(&chain, &tx);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().code, "E004");
    });
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

#[test]
fn txid_changes_when_public_key_changes() {
    let mut tx = Transaction::new(&dummy_address('a'), &dummy_address('b'), 1, 0, 0);
    let txid1 = tx.txid.clone();
    tx.public_key = "aa".repeat(33);
    let txid2 = tx.compute_txid();
    assert_ne!(txid1, txid2);
}

#[test]
#[serial]
fn block_total_fees_mismatch_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new();
        let prev = chain.chain.last().unwrap().clone();
        let mut b = Block::new(
            prev.index + 1,
            prev.hash.clone(),
            Blockchain::calculate_reward(prev.index + 1),
            chain.nbits,
            Vec::new(),
        );
        b.total_fees = 1_000_000;
        b.reward = b.subsidy.saturating_add(b.total_fees);
        b.hash = b.calculate_hash();
        let err = validation::validate_block(&chain, &prev, &b);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().code, "E047");
    });
}

#[test]
#[serial]
fn block_with_invalid_mined_for_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new();
        let prev = chain.chain.last().unwrap().clone();
        let mut b = Block::new_for(
            prev.index + 1,
            prev.hash.clone(),
            Blockchain::calculate_reward(prev.index + 1),
            0,
            chain.nbits,
            Vec::new(),
            "not-an-address".to_string(),
            String::new(),
            prev.chainwork,
        );
        b.hash = b.calculate_hash();
        let err = validation::validate_block(&chain, &prev, &b);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().code, "E049");
    });
}

#[test]
#[serial]
fn block_with_inconsistent_coinbase_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new();
        let prev = chain.chain.last().unwrap().clone();
        let addr = format!("ZENA{}", "a".repeat(40));
        let mut b = Block::new_for(
            prev.index + 1,
            prev.hash.clone(),
            Blockchain::calculate_reward(prev.index + 1),
            0,
            chain.nbits,
            Vec::new(),
            addr.clone(),
            String::new(),
            prev.chainwork,
        );
        b.coinbase_txid = "00".repeat(32);
        b.hash = b.calculate_hash();
        let err = validation::validate_block(&chain, &prev, &b);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().code, "E048");
    });
}
