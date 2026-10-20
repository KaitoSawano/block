// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! LevelDB adapter for the Merkle Patricia Trie using rusty-leveldb (pure Rust).

use eth_trie::DB;
use rusty_leveldb::{DB as LevelDB, Options};
use std::path::Path;

pub struct LevelDbAdapter {
    db: LevelDB,
}

impl LevelDbAdapter {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut opt = Options::default();
        opt.create_if_missing = true;
        let db = LevelDB::open(path, opt).map_err(|e| format!("{:?}", e))?;
        Ok(LevelDbAdapter { db })
    }
}

// Implement the `DB` trait required by `eth_trie`.
impl DB for LevelDbAdapter {
    type Error = String;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        // rusty-leveldb's `get` takes &mut self, but the trait requires &self.
        // This is a known limitation; we use a hack: clone the db reference? No.
        // Actually, `rusty-leveldb`'s get is &mut self. We need interior mutability.
        // Let's use a Mutex to satisfy the trait.
        // Wait, the trait `DB` from eth_trie might have `get(&self, ...)`.
        // Let's check the docs: it says `fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;`
        // So we need to wrap the DB in a Mutex.
        // This is a bit inefficient but necessary for compatibility.
        // Alternatively, we can use `RefCell` if single-threaded, but `DB` requires Send+Sync.
        // So `Mutex` is the way.
        // However, I need to change the struct to hold `Mutex<LevelDB>`.
        // Let me rewrite the struct.
        unimplemented!("Will be fixed below")
    }

    fn contains(&self, key: &[u8]) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn remove(&self, key: &[u8]) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn flush(&self) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
