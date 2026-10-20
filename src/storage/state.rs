// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Ethereum-style state database using Merkle Patricia Trie backed by LevelDB.

use eth_trie::{EthTrie, Trie};
use keccak_hash::keccak;
use std::path::Path;
use std::sync::Arc;

use super::account::Account;
use super::adapter::LevelDbAdapter;

pub struct StateDB {
    trie: EthTrie<LevelDbAdapter>,
}

impl StateDB {
    pub fn open(path: &Path) -> Result<Self, String> {
        let adapter = LevelDbAdapter::open(path)?;
        let trie = EthTrie::new(Arc::new(adapter));
        Ok(StateDB { trie })
    }

    fn address_key(address: &str) -> [u8; 32] {
        keccak(address.as_bytes()).0
    }

    pub fn update_account(&mut self, address: &str, account: &Account) -> Result<(), String> {
        let key = Self::address_key(address);
        let value = rlp::encode(account).to_vec();
        self.trie.insert(&key, &value).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_account(&self, address: &str) -> Result<Option<Account>, String> {
        let key = Self::address_key(address);
        match self.trie.get(&key).map_err(|e| e.to_string())? {
            Some(value) => {
                let account: Account =
                    rlp::decode(&value).map_err(|e| format!("RLP decode error: {}", e))?;
                Ok(Some(account))
            }
            None => Ok(None),
        }
    }

    pub fn root_hash(&mut self) -> Result<[u8; 32], String> {
        let root = self.trie.root_hash().map_err(|e| e.to_string())?;
        // In eth_trie 0.6, root_hash returns a FixedBytes<32>.
        // We convert it to [u8; 32] using as_slice().
        let mut arr = [0u8; 32];
        arr.copy_from_slice(root.as_slice());
        Ok(arr)
    }
