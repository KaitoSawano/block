// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Ethereum-style state database using Merkle Patricia Trie backed by LevelDB.

use eth_trie::{EthTrie, Trie};
use keccak_hash::keccak;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::account::Account;
use super::adapter::LevelDbAdapter;

pub struct StateDB {
    trie: EthTrie<LevelDbAdapter>,
    // Store the adapter Arc so we can recreate the trie during rollback.
    adapter: Arc<LevelDbAdapter>,
    path: PathBuf,
}

impl StateDB {
    pub fn open(path: &Path) -> Result<Self, String> {
        let adapter = Arc::new(LevelDbAdapter::open(path)?);
        let trie = EthTrie::new(adapter.clone());
        Ok(StateDB {
            trie,
            adapter,
            path: path.to_path_buf(),
        })
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

    /// Rollback the state to a specific root hash. This is used during a chain reorganization.
    /// Creates a new trie instance from the stored adapter and the provided root hash.
    pub fn rollback_to(&mut self, root_hash: &[u8; 32]) -> Result<(), String> {
        // Convert the byte array to the H256 type expected by eth_trie.
        let root = eth_trie::H256::from_slice(root_hash);
        
        // Create a new trie from the same adapter, rooted at the given hash.
        let new_trie = EthTrie::from(self.adapter.clone(), root)
            .map_err(|e| format!("failed to create trie from root: {}", e))?;
        
        self.trie = new_trie;
        Ok(())
    }

    /// Return the total balance, which includes both mature and immature funds.
    pub fn balance_of(&self, address: &str) -> u64 {
        match self.get_account(address) {
            Ok(Some(acc)) => acc.balance.saturating_add(acc.immature_balance),
            _ => 0,
        }
    }

    /// Return the spendable balance, which excludes immature mining rewards.
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
            .unwrap_or(Account { nonce: 0, balance: 0, immature_balance: 0, last_reward_height: 0 });

        if from_acc.nonce != nonce {
            return Err(format!(
                "invalid nonce: got {}, expected {}",
                nonce, from_acc.nonce
            ));
        }

        let total = amount
            .checked_add(fee)
            .ok_or_else(|| "amount + fee overflows u64".to_string())?;

        // Coinbase Maturity enforcement: transactions can only spend mature funds.
        if from_acc.balance < total {
            return Err(format!(
                "insufficient mature balance: required {}, available {} (immature: {})",
                total, from_acc.balance, from_acc.immature_balance
            ));
        }

        let mut to_acc = self
            .get_account(to)?
            .unwrap_or(Account { nonce: 0, balance: 0, immature_balance: 0, last_reward_height: 0 });

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

    /// Credit a mining reward to an account. The reward enters the `immature_balance`
    /// and is moved to the `balance` after the `COINBASE_MATURITY` window has elapsed.
    pub fn credit_reward(&mut self, miner: &str, reward: u64, block_height: u64) -> Result<(), String> {
        let mut acc = self
            .get_account(miner)?
            .unwrap_or(Account { nonce: 0, balance: 0, immature_balance: 0, last_reward_height: 0 });

        let maturity_window = crate::consensus::rules::COINBASE_MATURITY;

        // Step 1: Mature any previous immature balance if the maturity window has passed.
        if acc.immature_balance > 0 && block_height >= acc.last_reward_height.saturating_add(maturity_window) {
            acc.balance = acc.balance.saturating_add(acc.immature_balance);
            acc.immature_balance = 0;
        }

        // Step 2: Add the new reward to the immature balance.
        acc.immature_balance = acc
            .immature_balance
            .checked_add(reward)
            .ok_or_else(|| "miner balance overflow".to_string())?;
        
        // Step 3: Update the height at which the last reward was credited.
        acc.last_reward_height = block_height;

        self.update_account(miner, &acc)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), String> {
        // The adapter flushes on every write, so this is a no-op.
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
