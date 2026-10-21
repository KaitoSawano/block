// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use serial_test::serial;
use zenachain::blockchain::Blockchain;
use zenachain::storage;

fn with_temp_datadir<F: FnOnce()>(f: F) {
    let dir = tempfile::tempdir().expect("tempdir failed");
    std::env::set_var("ZENAC_DATADIR", dir.path());
    f();
    std::env::remove_var("ZENAC_DATADIR");
}

#[test]
#[serial]
fn save_and_load_chain() {
    with_temp_datadir(|| {
        // Step 1: Create a chain and save it.
        // We use a scope to ensure the chain (and its LevelDB lock) is
        // released before we attempt to load it again.
        {
            let chain = Blockchain::new();
            storage::save_chain(&chain).expect("save failed");
            // `chain` is dropped here, releasing the LevelDB lock.
        }

        // Step 2: Load the chain from disk.
        // At this point, no other instance holds the lock.
        let loaded = storage::load_chain().expect("load failed");
        assert_eq!(loaded.height(), 0);
        assert_eq!(loaded.chain.len(), 1);
    });
}

#[test]
#[serial]
fn load_chain_returns_new_when_empty() {
    with_temp_datadir(|| {
        let loaded = storage::load_chain().expect("load failed");
        assert_eq!(loaded.chain.len(), 1);
        assert_eq!(loaded.height(), 0);
    });
}
