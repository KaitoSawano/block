// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! LevelDB adapter for the Merkle Patricia Trie.

use eth_trie::TrieDB;
use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::{Options, ReadOptions, WriteOptions};
use std::path::Path;

pub struct LevelDbAdapter {
    db: Database<Vec<u8>>,
}

impl LevelDbAdapter {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut options = Options::new();
        options.create_if_missing = true;
        let db = Database::open(path, options)
            .map_err(|e| format!("failed to open leveldb: {:?}", e))?;
        Ok(LevelDbAdapter { db })
    }
}

impl TrieDB for LevelDbAdapter {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let read_opts = ReadOptions::new();
        self.db.get(read_opts, key).ok().flatten()
    }

    fn insert(&self, key: &[u8], value: Vec<u8>) {
        let write_opts = WriteOptions::new();
        let _ = self.db.put(write_opts, key, &value);
    }

    fn remove(&self, key: &[u8]) {
        let write_opts = WriteOptions::new();
        let _ = self.db.delete(write_opts, key);
    }
}
