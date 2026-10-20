// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Ethereum-style state database using Merkle Patricia Trie backed by LevelDB.

use eth_trie::{EthTrie, Trie, TrieDB};
use keccak_hash::keccak;
use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::{Options, ReadOptions, WriteOptions};
use std::path::Path;
use std::sync::Arc;

use crate::account::Account;

/// Adapter to make LevelDB implement the `TrieDB` trait required by `eth-trie`.
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
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&root);
        Ok(arr)
    }

    pub fn balance_of(&self, address: &str) -> u64 {
        match self.get_account(address) {
            Ok(Some(acc)) => acc.balance,
            _ => 0,
        }
    }

    pub fn nonce_of(&self, address: &str) -> u64 {
        match self.get_account(address) {
            Ok(Some(acc)) => acc.nonce,
            _ => 0,
        }
    }

    pub fn apply_transaction(
        &mut self,
        from: &str,
        to: &str,
        amount: u64,
        fee: u64,
        nonce: u64,
    ) -> Result<(), String> {
        let mut from_acc = self
            .get_account(from)?
            .unwrap_or(Account { nonce: 0, balance: 0 });

        if from_acc.nonce != nonce {
            return Err(format!(
                "invalid nonce: got {}, expected {}",
                nonce, from_acc.nonce
            ));
        }
        let total = amount
            .checked_add(fee)
            .ok_or_else(|| "amount + fee overflows u64".to_string())?;
        if from_acc.balance < total {
            return Err(format!(
                "insufficient balance: required {}, available {}",
                total, from_acc.balance
            ));
        }

        let mut to_acc = self
            .get_account(to)?
            .unwrap_or(Account { nonce: 0, balance: 0 });

        from_acc.balance -= total;
        from_acc.nonce += 1;
        to_acc.balance = to_acc
            .balance
            .checked_add(amount)
            .ok_or_else(|| "recipient balance overflow".to_string())?;

        self.update_account(from, &from_acc)?;
        self.update_account(to, &to_acc)?;

        Ok(())
    }

    pub fn credit_reward(&mut self, miner: &str, reward: u64) -> Result<(), String> {
        let mut acc = self
            .get_account(miner)?
            .unwrap_or(Account { nonce: 0, balance: 0 });
        acc.balance = acc
            .balance
            .checked_add(reward)
            .ok_or_else(|| "miner balance overflow".to_string())?;
        self.update_account(miner, &acc)?;
        Ok(())
    }
}
