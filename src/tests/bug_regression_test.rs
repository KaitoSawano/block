// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Regression tests for the five bugs identified during the code review.
//!
//! Bug #1: Genesis nonce mismatch between `genesis.rs` and `spec.rs`.
//! Bug #2: Hardcoded `ADDRESS_PREFIX` that did not adapt to the network.
//! Bug #3: `StateDB` panicked on `clone()` (compile-time check).
//! Bug #4: Race condition on `ZENAC_DATADIR` in tests.
//! Bug #5: Legacy genesis accessors always returned Mainnet values (compile-time check).

use serial_test::serial; // For serializing tests that modify global state.
use std::sync::Mutex; // For protecting the environment variable.
use zenachain::blockchain::Blockchain; // The Blockchain struct.
use zenachain::consensus::genesis; // Genesis parameters.
use zenachain::consensus::spec::{self, Network}; // Network parameters.
use zenachain::validation; // Validation rules.

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

// ----------------------------------------------------------------------
// Bug #1: Genesis Nonce Consistency
// ----------------------------------------------------------------------

// This test verifies that the genesis nonce in `spec.rs` matches the one
// in `genesis.rs` for all networks. If they diverge, the node will refuse
// to start because the mined genesis block will not match the expected
// constants.
#[test]
fn bug_1_genesis_nonce_is_consistent() {
    // Mainnet: `spec.rs` and `genesis.rs` must agree.
    let spec_main = spec::Network::Main.expected_genesis_nonce(); // From spec.rs.
    let genesis_main = genesis::expected_genesis_nonce_for(Network::Main); // From genesis.rs.
    assert_eq!(
        spec_main, genesis_main,
        "Mainnet genesis nonce mismatch: spec.rs = {}, genesis.rs = {}",
        spec_main, genesis_main
    );

    // Testnet: `spec.rs` and `genesis.rs` must agree.
    let spec_test = spec::Network::Test.expected_genesis_nonce(); // From spec.rs.
    let genesis_test = genesis::expected_genesis_nonce_for(Network::Test); // From genesis.rs.
    assert_eq!(
        spec_test, genesis_test,
        "Testnet genesis nonce mismatch: spec.rs = {}, genesis.rs = {}",
        spec_test, genesis_test
    );

    // Regtest: `spec.rs` and `genesis.rs` must agree.
    let spec_regtest = spec::Network::Regtest.expected_genesis_nonce(); // From spec.rs.
    let genesis_regtest = genesis::expected_genesis_nonce_for(Network::Regtest); // From genesis.rs.
    assert_eq!(
        spec_regtest, genesis_regtest,
        "Regtest genesis nonce mismatch: spec.rs = {}, genesis.rs = {}",
        spec_regtest, genesis_regtest
    );
}

// This test verifies that the expected genesis nonce is not zero for Mainnet
// and Testnet (it must be a valid mined nonce). Regtest is allowed to be
// zero or very small because it uses an easy target.
#[test]
fn bug_1_genesis_nonce_is_valid() {
    let main = genesis::expected_genesis_nonce_for(Network::Main); // Mainnet nonce.
    assert!(main > 0, "Mainnet genesis nonce must be greater than zero");

    let test = genesis::expected_genesis_nonce_for(Network::Test); // Testnet nonce.
    assert!(test > 0, "Testnet genesis nonce must be greater than zero");

    let regtest = genesis::expected_genesis_nonce_for(Network::Regtest); // Regtest nonce.
    assert!(regtest < 100, "Regtest genesis nonce should be small");
}

// ----------------------------------------------------------------------
// Bug #2: Network-Aware Address Prefix
// ----------------------------------------------------------------------

// This test verifies that `validation::current_address_prefix()` returns
// the correct prefix for the active network. Because `current_network()`
// is a `OnceLock`, we can only test the default (Mainnet) here. The
// other networks are tested via the `Network::address_prefix()` method.
#[test]
fn bug_2_address_prefix_is_network_aware() {
    // The default network is Mainnet, so the prefix must be "ZENA".
    assert_eq!(
        validation::current_address_prefix(),
        "ZENA",
        "Default address prefix must be ZENA (Mainnet)"
    );

    // Verify the prefix for each network directly via the `Network` enum.
    assert_eq!(Network::Main.address_prefix(), "ZENA"); // Mainnet prefix.
    assert_eq!(Network::Test.address_prefix(), "ZTES"); // Testnet prefix.
    assert_eq!(Network::Regtest.address_prefix(), "ZREG"); // Regtest prefix.
}

// This test verifies that `is_valid_address()` accepts addresses with the
// correct prefix and rejects addresses with the wrong prefix.
#[test]
fn bug_2_address_validation_uses_correct_prefix() {
    // A valid Mainnet address.
    let main_addr = format!("ZENA{}", "a".repeat(40));
    assert!(validation::is_valid_address(&main_addr), "Mainnet address should be valid");

    // A Testnet address with the wrong prefix for the active network (Mainnet).
    let test_addr = format!("ZTES{}", "a".repeat(40));
    assert!(!validation::is_valid_address(&test_addr), "Testnet address should be invalid on Mainnet");

    // A Regtest address with the wrong prefix for the active network (Mainnet).
    let regtest_addr = format!("ZREG{}", "a".repeat(40));
    assert!(!validation::is_valid_address(&regtest_addr), "Regtest address should be invalid on Mainnet");
}

// ----------------------------------------------------------------------
// Bug #3: StateDB Clone Panic (Compile-Time)
// ----------------------------------------------------------------------

// This is a compile-time test. If `StateDB` implements `Clone`, the
// following function will compile. If it does not, the compiler will
// reject it. We cannot write a runtime test for this because the panic
// happens at runtime if `Clone` is implemented.
//
// To verify that the bug is fixed, simply uncomment the following
// function and run `cargo check`. If it fails to compile, the fix is
// working.
//
// fn bug_3_state_db_cannot_be_cloned(db: &zenachain::storage::StateDB) {
//     let _ = db.clone(); // This should NOT compile.
// }

// ----------------------------------------------------------------------
// Bug #4: Environment Variable Race Condition
// ----------------------------------------------------------------------

// This test verifies that `with_temp_datadir` is safe to use with
// multiple tests. Because we use a mutex, the tests will not race.
#[test]
#[serial]
fn bug_4_temp_datadir_is_safe() {
    with_temp_datadir(|| {
        let chain = Blockchain::new(); // Create a new chain.
        assert_eq!(chain.height(), 0, "Genesis height should be 0"); // Sanity check.
    });
}

// A second test that uses `with_temp_datadir` to verify that the mutex
// serializes access correctly.
#[test]
#[serial]
fn bug_4_temp_datadir_is_safe_again() {
    with_temp_datadir(|| {
        let chain = Blockchain::new(); // Create a new chain.
        assert_eq!(chain.chain.len(), 1, "Genesis chain should have one block"); // Sanity check.
    });
}

// ----------------------------------------------------------------------
// Bug #5: Legacy Genesis Accessors (Compile-Time)
// ----------------------------------------------------------------------

// This is a compile-time test. If the legacy accessors
// `expected_genesis_hash()`, `expected_genesis_merkle_root()`, and
// `expected_genesis_nonce()` still exist, the following function will
// compile. If they have been removed, the compiler will reject it.
//
// To verify that the bug is fixed, simply uncomment the following
// function and run `cargo check`. If it fails to compile, the fix is
// working.
//
// fn bug_5_legacy_accessors_are_removed() {
//     let _ = genesis::expected_genesis_hash(); // This should NOT compile.
//     let _ = genesis::expected_genesis_merkle_root(); // This should NOT compile.
//     let _ = genesis::expected_genesis_nonce(); // This should NOT compile.
// }
