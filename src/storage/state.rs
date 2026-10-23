// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Ethereum-style state database using Merkle Patricia Trie backed by LevelDB.

use alloy_primitives::B256;
use eth_trie::{EthTrie, Trie};
use keccak_hash::keccak;
use std::path::Path;
use std::sync::Arc;

use super::account::Account;
use super::adapter::LevelDbAdapter;

pub struct StateDB {
    trie: EthTrie<LevelDbAdapter>,
    adapter: Arc<LevelDbAdapter>,
}

impl StateDB {
    pub fn open(path: &Path) -> Result<Self, String> {
        let adapter = Arc::new(LevelDbAdapter::open(path)?);
        let trie = EthTrie::new(adapter.clone());
        Ok(StateDB { trie, adapter })
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
        arr.copy_from_slice(root.as_slice());
        Ok(arr)
    }

    pub fn rollback_to(&mut self, root_hash: &[u8; 32]) -> Result<(), String> {
        let root = B256::from_slice(root_hash);
        let new_trie = EthTrie::from(self.adapter.clone(), root)
            .map_err(|e| format!("failed to create trie from root: {}", e))?;
        self.trie = new_trie;
        Ok(())
    }

    /// Return the total balance (mature + immature).
    pub fn balance_of(&self, address: &str) -> u64 {
        match self.get_account(address) {
            Ok(Some(acc)) => {
                let immature: u64 = acc.immature_rewards.iter().map(|(_, a)| a).sum();
                acc.balance.saturating_add(immature)
            }
            _ => 0,
        }
    }

    /// Return the spendable (mature) balance.
    pub fn spendable_balance_of(&self, address: &str) -> u64 {
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
            .unwrap_or(Account { nonce: 0, balance: 0, immature_rewards: Vec::new() });

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
                "insufficient mature balance: required {}, available {}",
                total, from_acc.balance
            ));
        }

        let mut to_acc = self
            .get_account(to)?
            .unwrap_or(Account { nonce: 0, balance: 0, immature_rewards: Vec::new() });

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

    /// Credit a mining reward to an account. The reward is added to the list of
    /// immature rewards. On each call, any reward that has reached `COINBASE_MATURITY`
    /// (100 blocks) is moved to the mature balance.
    pub fn credit_reward(&mut self, miner: &str, reward: u64, block_height: u64) -> Result<(), String> {
        let mut acc = self
            .get_account(miner)?
            .unwrap_or(Account { nonce: 0, balance: 0, immature_rewards: Vec::new() });

        let maturity = crate::consensus::rules::COINBASE_MATURITY;

        // Mature any rewards that have reached the maturity window.
        let mut still_immature = Vec::new();
        for (height, amount) in acc.immature_rewards.drain(..) {
            if block_height >= height.saturating_add(maturity) {
                acc.balance = acc.balance.saturating_add(amount);
            } else {
                still_immature.push((height, amount));
            }
        }
        acc.immature_rewards = still_immature;

        // Add the new reward.
        acc.immature_rewards.push((block_height, reward));

        self.update_account(miner, &acc)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), String> {
        Ok(())
    }
}

impl std::fmt::Debug for StateDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateDB").finish()
    }
}

impl Clone for StateDB {
    fn clone(&self) -> Self {
        panic!("StateDB cannot be cloned");
    }
}

impl Default for StateDB {
    fn default() -> Self {
        panic!("StateDB must be explicitly opened with a path");
    }
}
