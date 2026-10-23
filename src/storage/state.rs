// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Ethereum-style state database using Merkle Patricia Trie backed by LevelDB.

use alloy_primitives::B256; // 256-bit hash type from alloy.
use eth_trie::{EthTrie, Trie}; // Ethereum-style Merkle Patricia Trie.
use keccak_hash::keccak; // Keccak hashing for address keys.
use std::path::Path; // For filesystem paths.
use std::sync::Arc; // For shared ownership of the adapter.

use super::account::Account; // The account structure.
use super::adapter::LevelDbAdapter; // The LevelDB adapter for the trie.

/// The state database, backed by a Merkle Patricia Trie and LevelDB.
pub struct StateDB {
    trie: EthTrie<LevelDbAdapter>, // The underlying trie.
    adapter: Arc<LevelDbAdapter>,  // Shared adapter for creating new trie roots.
}

impl StateDB {
    // Opens the state database at the given path.
    pub fn open(path: &Path) -> Result<Self, String> {
        let adapter = Arc::new(LevelDbAdapter::open(path)?); // Open LevelDB.
        let trie = EthTrie::new(adapter.clone()); // Create a new trie.
        Ok(StateDB { trie, adapter }) // Return the StateDB instance.
    }

    // Computes the 32-byte key for an address using Keccak-256.
    fn address_key(address: &str) -> [u8; 32] {
        keccak(address.as_bytes()).0 // Hash the address bytes.
    }

    // Updates or inserts an account in the trie.
    pub fn update_account(&mut self, address: &str, account: &Account) -> Result<(), String> {
        let key = Self::address_key(address); // Compute the address key.
        let value = rlp::encode(account).to_vec(); // RLP-encode the account.
        self.trie.insert(&key, &value).map_err(|e| e.to_string())?; // Insert into the trie.
        Ok(()) // Success.
    }

    // Retrieves an account from the trie, if it exists.
    pub fn get_account(&self, address: &str) -> Result<Option<Account>, String> {
        let key = Self::address_key(address); // Compute the address key.
        match self.trie.get(&key).map_err(|e| e.to_string())? {
            // If the key exists in the trie...
            Some(value) => {
                let account: Account =
                    rlp::decode(&value).map_err(|e| format!("RLP decode error: {}", e))?; // Decode the RLP.
                Ok(Some(account)) // Return the account.
            }
            None => Ok(None), // The account does not exist.
        }
    }

    // Returns the current root hash of the trie.
    pub fn root_hash(&mut self) -> Result<[u8; 32], String> {
        let root = self.trie.root_hash().map_err(|e| e.to_string())?; // Get the root hash.
        let mut arr = [0u8; 32]; // Prepare a 32-byte array.
        arr.copy_from_slice(root.as_slice()); // Copy the root hash into the array.
        Ok(arr) // Return the array.
    }

    // Rolls back the trie to a specific root hash.
    pub fn rollback_to(&mut self, root_hash: &[u8; 32]) -> Result<(), String> {
        let root = B256::from_slice(root_hash); // Convert the bytes to a B256.
        let new_trie = EthTrie::from(self.adapter.clone(), root)
            .map_err(|e| format!("failed to create trie from root: {}", e))?; // Create a new trie from the root.
        self.trie = new_trie; // Replace the current trie.
        Ok(()) // Success.
    }

    // Returns the total balance (mature + immature) for an address.
    pub fn balance_of(&self, address: &str) -> u64 {
        match self.get_account(address) {
            // If the account exists...
            Ok(Some(acc)) => {
                let immature: u64 = acc.immature_rewards.iter().map(|(_, a)| a).sum(); // Sum the immature rewards.
                acc.balance.saturating_add(immature) // Return the total balance.
            }
            _ => 0, // The account does not exist; balance is zero.
        }
    }

    // Returns the spendable (mature) balance for an address.
    pub fn spendable_balance_of(&self, address: &str) -> u64 {
        match self.get_account(address) {
            Ok(Some(acc)) => acc.balance, // Return the mature balance.
            _ => 0, // The account does not exist; balance is zero.
        }
    }

    // Returns the nonce for an address.
    pub fn nonce_of(&self, address: &str) -> u64 {
        match self.get_account(address) {
            Ok(Some(acc)) => acc.nonce, // Return the nonce.
            _ => 0, // The account does not exist; nonce is zero.
        }
    }

    // Applies a transaction to the state.
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
            .unwrap_or(Account { nonce: 0, balance: 0, immature_rewards: Vec::new() }); // Get the sender account or create a new one.

        if from_acc.nonce != nonce {
            // If the nonce does not match, return an error.
            return Err(format!(
                "invalid nonce: got {}, expected {}",
                nonce, from_acc.nonce
            ));
        }

        let total = amount
            .checked_add(fee)
            .ok_or_else(|| "amount + fee overflows u64".to_string())?; // Compute the total required.

        if from_acc.balance < total {
            // If the mature balance is insufficient, return an error.
            return Err(format!(
                "insufficient mature balance: required {}, available {}",
                total, from_acc.balance
            ));
        }

        let mut to_acc = self
            .get_account(to)?
            .unwrap_or(Account { nonce: 0, balance: 0, immature_rewards: Vec::new() }); // Get the recipient account or create a new one.

        from_acc.balance -= total; // Deduct the total from the sender.
        from_acc.nonce += 1; // Increment the sender nonce.
        to_acc.balance = to_acc
            .balance
            .checked_add(amount)
            .ok_or_else(|| "recipient balance overflow".to_string())?; // Add the amount to the recipient.

        self.update_account(from, &from_acc)?; // Update the sender account.
        self.update_account(to, &to_acc)?; // Update the recipient account.

        Ok(()) // Success.
    }

    // Credits a mining reward to an account.
    // The reward is added to the list of immature rewards. On each call,
    // any reward that has reached `COINBASE_MATURITY` (100 blocks) is moved
    // to the mature balance.
    pub fn credit_reward(&mut self, miner: &str, reward: u64, block_height: u64) -> Result<(), String> {
        let mut acc = self
            .get_account(miner)?
            .unwrap_or(Account { nonce: 0, balance: 0, immature_rewards: Vec::new() }); // Get the miner account or create a new one.

        let maturity = crate::consensus::rules::COINBASE_MATURITY; // Get the maturity window.

        // Mature any rewards that have reached the maturity window.
        let mut still_immature = Vec::new(); // Temporary list for still-immature rewards.
        for (height, amount) in acc.immature_rewards.drain(..) {
            // Iterate over the immature rewards.
            if block_height >= height.saturating_add(maturity) {
                // If the reward has matured...
                acc.balance = acc.balance.saturating_add(amount); // Move it to the mature balance.
            } else {
                still_immature.push((height, amount)); // Otherwise, keep it in the list.
            }
        }
        acc.immature_rewards = still_immature; // Replace the immature rewards list.

        // Add the new reward.
        acc.immature_rewards.push((block_height, reward)); // Add the new reward to the list.

        self.update_account(miner, &acc)?; // Update the miner account.
        Ok(()) // Success.
    }

    // Flushes the trie to disk. Currently a no-op because LevelDB flushes on every write.
    pub fn flush(&mut self) -> Result<(), String> {
        Ok(()) // Nothing to do.
    }
}

// Debug implementation for StateDB.
impl std::fmt::Debug for StateDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateDB").finish() // A minimal debug representation.
    }
}

// NOTE: `Clone` and `Default` are intentionally NOT implemented for StateDB.
// Cloning a StateDB is dangerous because it would create a second independent
// trie that shares the same LevelDB adapter, leading to data corruption.
// Default is also dangerous because it would create a StateDB without a path,
// which cannot be persisted. By omitting these traits, we force the compiler
// to catch any accidental misuse at compile time.
