// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

use serial_test::serial; // For serializing tests that modify global state.
use std::sync::Mutex; // For protecting the environment variable.
use zenachain::block::Block; // The Block struct.
use zenachain::blockchain::Blockchain; // The Blockchain struct.
use zenachain::consensus::rules; // Consensus rule constants.
use zenachain::transaction::Transaction; // The Transaction struct.
use zenachain::validation; // The validation module.

// A global mutex to protect the ZENAC_DATADIR environment variable.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

// A helper function that sets up a temporary datadir, runs the closure, and
// then restores the environment. It uses a mutex to prevent race conditions.
fn with_temp_datadir<F: FnOnce()>(f: F) {
    let _guard = ENV_MUTEX.lock().unwrap(); // Acquire the mutex.
    let dir = tempfile::tempdir().expect("tempdir failed"); // Create a temp directory.
    std::env::set_var("ZENAC_DATADIR", dir.path()); // Set the environment variable.
    f(); // Run the closure.
    std::env::remove_var("ZENAC_DATADIR"); // Remove the environment variable.
}

// Creates a dummy address with a repeated seed character.
fn dummy_address(seed: char) -> String {
    format!("ZENA{}", seed.to_string().repeat(40)) // Prefix "ZENA" + 40 repeated chars.
}

#[test]
#[serial]
#[cfg(not(feature = "skip-pow"))] // Skip this test if the `skip-pow` feature is enabled.
fn valid_chain_passes() {
    with_temp_datadir(|| {
        let chain = Blockchain::new(); // Create a new chain.
        assert!(validation::validate_chain(&chain).is_ok()); // The chain should be valid.
    });
}

#[test]
#[serial]
fn block_with_wrong_previous_hash_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new(); // Create a new chain.
        let prev = chain.chain.last().unwrap().clone(); // Get the previous block.
        let mut b = Block::new(
            prev.index + 1, // The next index.
            "0".repeat(64), // A wrong previous hash.
            50_000_000_000, // The reward.
            chain.nbits, // The nbits.
            Vec::new(), // No transactions.
        );
        b.hash = b.calculate_hash(); // Recalculate the hash.
        let err = validation::validate_block(&chain, &prev, &b); // Validate the block.
        assert!(err.is_err()); // It should be rejected.
        assert_eq!(err.unwrap_err().code, "E035"); // The error code should be E035.
    });
}

#[test]
#[serial]
fn block_with_wrong_index_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new(); // Create a new chain.
        let prev = chain.chain.last().unwrap().clone(); // Get the previous block.
        let mut b = Block::new(
            prev.index + 2, // A wrong index (skipping one).
            prev.hash.clone(), // The correct previous hash.
            50_000_000_000, // The reward.
            chain.nbits, // The nbits.
            Vec::new(), // No transactions.
        );
        b.hash = b.calculate_hash(); // Recalculate the hash.
        let err = validation::validate_block(&chain, &prev, &b); // Validate the block.
        assert!(err.is_err()); // It should be rejected.
        assert_eq!(err.unwrap_err().code, "E036"); // The error code should be E036.
    });
}

#[test]
#[serial]
fn transaction_with_zero_amount_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new(); // Create a new chain.
        let from = dummy_address('a'); // A dummy sender address.
        let to = dummy_address('b'); // A dummy recipient address.
        let tx = Transaction::new(&from, &to, 0, 0, 0); // A transaction with zero amount.
        let err = validation::validate_transaction(&chain, &tx); // Validate the transaction.
        assert!(err.is_err()); // It should be rejected.
        assert_eq!(err.unwrap_err().code, "E005"); // The error code should be E005.
    });
}

#[test]
#[serial]
fn transaction_with_self_send_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new(); // Create a new chain.
        let addr = dummy_address('a'); // A dummy address.
        let tx = Transaction::new(&addr, &addr, 1, 0, 0); // A self-send transaction.
        let err = validation::validate_transaction(&chain, &tx); // Validate the transaction.
        assert!(err.is_err()); // It should be rejected.
        assert_eq!(err.unwrap_err().code, "E004"); // The error code should be E004.
    });
}

#[test]
fn address_format_validation() {
    assert!(validation::is_valid_address(&format!("ZENA{}", "a".repeat(40)))); // A valid address.
    assert!(!validation::is_valid_address("ZENA123")); // Too short.
    assert!(!validation::is_valid_address(&format!("XXXX{}", "a".repeat(40)))); // Wrong prefix.
}

#[test]
fn zero_hash_constant_is_correct() {
    assert_eq!(rules::ZERO_HASH.len(), 64); // The zero hash should be 64 characters.
    assert!(rules::ZERO_HASH.chars().all(|c| c == '0')); // All characters should be '0'.
}

#[test]
fn txid_changes_when_public_key_changes() {
    let mut tx = Transaction::new(&dummy_address('a'), &dummy_address('b'), 1, 0, 0); // Create a transaction.
    let txid1 = tx.txid.clone(); // Record the original txid.
    tx.public_key = "aa".repeat(33); // Change the public key.
    let txid2 = tx.compute_txid(); // Recompute the txid.
    assert_ne!(txid1, txid2); // The txid should have changed.
}

#[test]
#[serial]
fn block_total_fees_mismatch_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new(); // Create a new chain.
        let prev = chain.chain.last().unwrap().clone(); // Get the previous block.
        let mut b = Block::new(
            prev.index + 1, // The next index.
            prev.hash.clone(), // The correct previous hash.
            Blockchain::calculate_reward(prev.index + 1), // The correct reward.
            chain.nbits, // The nbits.
            Vec::new(), // No transactions.
        );
        b.total_fees = 1_000_000; // Set a wrong total_fees.
        b.reward = b.subsidy.saturating_add(b.total_fees); // Update the reward.
        b.hash = b.calculate_hash(); // Recalculate the hash.
        let err = validation::validate_block(&chain, &prev, &b); // Validate the block.
        assert!(err.is_err()); // It should be rejected.
        assert_eq!(err.unwrap_err().code, "E047"); // The error code should be E047.
    });
}

#[test]
#[serial]
fn block_with_invalid_mined_for_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new(); // Create a new chain.
        let prev = chain.chain.last().unwrap().clone(); // Get the previous block.
        let mut b = Block::new_for(
            prev.index + 1, // The next index.
            prev.hash.clone(), // The correct previous hash.
            Blockchain::calculate_reward(prev.index + 1), // The correct reward.
            0, // No fees.
            chain.nbits, // The nbits.
            Vec::new(), // No transactions.
            "not-an-address".to_string(), // An invalid mined_for address.
            String::new(), // No state root.
            prev.chainwork, // The previous chainwork.
        );
        b.hash = b.calculate_hash(); // Recalculate the hash.
        let err = validation::validate_block(&chain, &prev, &b); // Validate the block.
        assert!(err.is_err()); // It should be rejected.
        assert_eq!(err.unwrap_err().code, "E049"); // The error code should be E049.
    });
}

#[test]
#[serial]
fn block_with_inconsistent_coinbase_is_rejected() {
    with_temp_datadir(|| {
        let chain = Blockchain::new(); // Create a new chain.
        let prev = chain.chain.last().unwrap().clone(); // Get the previous block.
        let addr = format!("ZENA{}", "a".repeat(40)); // A valid address.
        let mut b = Block::new_for(
            prev.index + 1, // The next index.
            prev.hash.clone(), // The correct previous hash.
            Blockchain::calculate_reward(prev.index + 1), // The correct reward.
            0, // No fees.
            chain.nbits, // The nbits.
            Vec::new(), // No transactions.
            addr.clone(), // The mined_for address.
            String::new(), // No state root.
            prev.chainwork, // The previous chainwork.
        );
        b.coinbase_txid = "00".repeat(32); // Set a wrong coinbase_txid.
        b.hash = b.calculate_hash(); // Recalculate the hash.
        let err = validation::validate_block(&chain, &prev, &b); // Validate the block.
        assert!(err.is_err()); // It should be rejected.
        assert_eq!(err.unwrap_err().code, "E048"); // The error code should be E048.
    });
}
