// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

use serial_test::serial; // For serializing tests that modify global state.
use std::sync::Mutex; // For protecting the environment variable.
use zenachain::blockchain::Blockchain; // The Blockchain struct.
use zenachain::storage; // The storage module.

// A global mutex to protect the ZENAC_DATADIR environment variable.
// This ensures that only one test can modify the environment at a time,
// even if the test runner runs tests in parallel.
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

#[test]
#[serial] // Ensure this test runs serially with other tests.
fn save_and_load_chain() {
    with_temp_datadir(|| {
        // Step 1: Create a chain and save it.
        // We use a scope to ensure the chain (and its LevelDB lock) is
        // released before we attempt to load it again.
        {
            let chain = Blockchain::new(); // Create a new chain.
            storage::save_chain(&chain).expect("save failed"); // Save it to disk.
            // `chain` is dropped here, releasing the LevelDB lock.
        }

        // Step 2: Load the chain from disk.
        // At this point, no other instance holds the lock.
        let loaded = storage::load_chain().expect("load failed"); // Load it from disk.
        assert_eq!(loaded.height(), 0); // The height should be 0 (genesis only).
        assert_eq!(loaded.chain.len(), 1); // There should be exactly one block.
    });
}

#[test]
#[serial]
fn load_chain_returns_new_when_empty() {
    with_temp_datadir(|| {
        let loaded = storage::load_chain().expect("load failed"); // Load from an empty dir.
        assert_eq!(loaded.chain.len(), 1); // The chain should contain only the genesis block.
        assert_eq!(loaded.height(), 0); // The height should be 0.
    });
}
