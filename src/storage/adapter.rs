// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! LevelDB adapter for the Merkle Patricia Trie using rusty-leveldb (pure Rust).

use eth_trie::DB;
use rusty_leveldb::{DB as LevelDB, Options};
use std::path::Path;
use std::sync::Mutex;

pub struct LevelDbAdapter {
    db: Mutex<LevelDB>,
}

impl LevelDbAdapter {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut opt = Options::default();
        opt.create_if_missing = true;
        let db = LevelDB::open(path, opt).map_err(|e| format!("{:?}", e))?;
        Ok(LevelDbAdapter { db: Mutex::new(db) })
    }
}

impl DB for LevelDbAdapter {
    type Error = std::io::Error;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        let mut db = self.db.lock().unwrap();
        Ok(db.get(key))
    }

    fn insert(&self, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error> {
        let mut db = self.db.lock().unwrap();
        db.put(key, &value)
            .map_err(|e| std::io::Error::other(format!("{:?}", e)))
    }

    fn remove(&self, key: &[u8]) -> Result<(), Self::Error> {
        let mut db = self.db.lock().unwrap();
        db.delete(key)
            .map_err(|e| std::io::Error::other(format!("{:?}", e)))
    }

    fn flush(&self) -> Result<(), Self::Error> {
        let mut db = self.db.lock().unwrap();
        db.flush()
            .map_err(|e| std::io::Error::other(format!("{:?}", e)))
    }
}
