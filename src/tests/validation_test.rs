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

/// The transaction identifier must change when the public key changes.
/// This is a regression test for signature malleability.
#[test]
fn txid_changes_when_public_key_changes() {
    let mut tx = Transaction::new(&dummy_address('a'), &dummy_address('b'), 1, 0);
    let txid1 = tx.txid.clone();
    tx.public_key = "aa".repeat(33);
    let txid2 = tx.compute_txid();
    assert_ne!(txid1, txid2);
}

/// The transaction identifier must change when the signature changes.
/// This is a regression test for signature malleability.
#[test]
fn txid_changes_when_signature_changes() {
    let mut tx = Transaction::new(&dummy_address('a'), &dummy_address('b'), 1, 0);
    let txid1 = tx.txid.clone();
    tx.signature = "bb".repeat(70);
    let txid2 = tx.compute_txid();
    assert_ne!(txid1, txid2);
}

/// The block total fees must equal the sum of the fees of all
/// transactions in the block.
#[test]
fn block_total_fees_mismatch_is_rejected() {
    let chain = Blockchain::new();
    let prev = chain.chain.last().unwrap().clone();
    let mut b = Block::new(prev.index + 1, prev.hash.clone(), 0, chain.nbits, Vec::new());
    b.total_fees = 1_000_000; // declared fees do not match the empty tx list
    b.reward = b.subsidy.saturating_add(b.total_fees);
    b.hash = b.calculate_hash();
    let err = validation::validate_block(&chain, &prev, &b);
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().code, "E047");
}

/// A non-genesis block must have a valid mined_for address.
#[test]
fn block_with_invalid_mined_for_is_rejected() {
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
    );
    b.hash = b.calculate_hash();
    let err = validation::validate_block(&chain, &prev, &b);
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().code, "E049");
}

/// A block whose coinbase_txid does not match index|mined_for|timestamp
/// must be rejected.
#[test]
fn block_with_inconsistent_coinbase_is_rejected() {
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
    );
    b.coinbase_txid = "00".repeat(32);
    b.hash = b.calculate_hash();
    let err = validation::validate_block(&chain, &prev, &b);
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().code, "E048");
}
