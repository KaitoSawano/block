// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Blockchain state and consensus-related operations.

use std::collections::HashMap; // For the legacy balances/nonces maps.

use crate::block::Block; // The block structure.
use crate::consensus::{self, genesis, pow as cpow, rules, spec}; // Consensus modules.
use crate::pow; // Proof-of-work functions.
use crate::storage::{Account, StateDB}; // Storage layer.
use crate::transaction::Transaction; // The transaction structure.
use crate::validation; // Validation rules.

// ============================================================
// ADDRESS PREFIX PARAMETERS
// ============================================================

/// Legacy (Base58Check) prefix byte for Mainnet.
pub const LEGACY_PREFIX_MAIN: u8 = 0x30;
/// Legacy (Base58Check) prefix byte for Testnet.
pub const LEGACY_PREFIX_TEST: u8 = 0x6f;
/// Legacy (Base58Check) prefix byte for Regtest.
pub const LEGACY_PREFIX_REGTEST: u8 = 0x6f;
/// Bech32 Human-Readable Part (HRP) for Mainnet.
pub const BECH32_HRP_MAIN: &str = "zena";
/// Bech32 Human-Readable Part (HRP) for Testnet.
pub const BECH32_HRP_TEST: &str = "ztes";
/// Bech32 Human-Readable Part (HRP) for Regtest.
pub const BECH32_HRP_REGTEST: &str = "zreg";

/// Return the Legacy prefix byte for the active network.
pub fn legacy_prefix_byte() -> u8 {
    match spec::current_network() {
        spec::Network::Main => LEGACY_PREFIX_MAIN,
        spec::Network::Test => LEGACY_PREFIX_TEST,
        spec::Network::Regtest => LEGACY_PREFIX_REGTEST,
    }
}

/// Return the Bech32 HRP for the active network.
pub fn bech32_hrp() -> &'static str {
    match spec::current_network() {
        spec::Network::Main => BECH32_HRP_MAIN,
        spec::Network::Test => BECH32_HRP_TEST,
        spec::Network::Regtest => BECH32_HRP_REGTEST,
    }
}

/// Return the Legacy prefix byte for a specific network.
pub fn legacy_prefix_byte_for(network: spec::Network) -> u8 {
    match network {
        spec::Network::Main => LEGACY_PREFIX_MAIN,
        spec::Network::Test => LEGACY_PREFIX_TEST,
        spec::Network::Regtest => LEGACY_PREFIX_REGTEST,
    }
}

/// Return the Bech32 HRP for a specific network.
pub fn bech32_hrp_for(network: spec::Network) -> &'static str {
    match network {
        spec::Network::Main => BECH32_HRP_MAIN,
        spec::Network::Test => BECH32_HRP_TEST,
        spec::Network::Regtest => BECH32_HRP_REGTEST,
    }
}

// ============================================================
// SOFT FORK DEPLOYMENTS
// ============================================================

/// Parameters for a soft fork deployment.
#[derive(Debug, Clone, Copy)]
pub struct SoftForkDeployment {
    /// Bit position to select the particular bit in nVersion (0-28).
    pub bit: u8,
    /// Start MedianTime for version bits miner confirmation.
    /// Use `NEVER_ACTIVE` for not scheduled.
    pub start_time: i64,
    /// Timeout/expiry MedianTime for the deployment attempt.
    /// Use `NO_TIMEOUT` for no timeout.
    pub timeout: i64,
    /// If lock in occurs, delay activation until at least this block height.
    pub min_activation_height: u64,
    /// Period of blocks to check signalling in (usually retarget period).
    pub period: u32,
    /// Minimum blocks signaling in the period to lock in (e.g., 1916 for 95%).
    pub threshold: u32,
}

/// Constant for start_time: never active.
pub const NEVER_ACTIVE: i64 = -2;
/// Constant for start_time: always active (useful for testing).
pub const ALWAYS_ACTIVE: i64 = -1;
/// Constant for timeout: no timeout.
pub const NO_TIMEOUT: i64 = i64::MAX;

/// The state of a soft fork deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftForkState {
    Defined,    // Before start_time.
    Started,    // Signaling period active.
    LockedIn,   // Threshold met, waiting for activation.
    Active,     // Rules enforced.
    Failed,     // Timeout without activation.
}

/// The registry of all soft fork deployments.
pub const SOFT_FORK_DEPLOYMENTS: &[(&str, SoftForkDeployment)] = &[
    (
        "disable_legacy_addresses",
        SoftForkDeployment {
            bit: 0,
            start_time: ALWAYS_ACTIVE, // Force active for testing.
            timeout: NO_TIMEOUT,
            min_activation_height: 10, // Activate after block 10.
            period: 2016,
            threshold: 1916, // 95%
        },
    ),
];

/// Returns the deployment for a given name, if it exists.
pub fn get_softfork_deployment(name: &str) -> Option<&'static SoftForkDeployment> {
    SOFT_FORK_DEPLOYMENTS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, d)| d)
}

/// Get the median time past of the last 11 blocks.
pub fn median_time_past(chain: &Blockchain) -> i64 {
    let mut timestamps: Vec<i64> = chain
        .chain
        .iter()
        .rev()
        .take(11)
        .map(|b| (b.timestamp / 1000) as i64) // Convert ms to seconds.
        .collect();
    if timestamps.is_empty() {
        return 0;
    }
    timestamps.sort();
    timestamps[timestamps.len() / 2]
}

/// Count how many blocks in the last `period` signaled for a given bit.
pub fn count_signals(chain: &Blockchain, bit: u8, period: u32) -> u32 {
    let start = chain.chain.len().saturating_sub(period as usize);
    let mut count = 0;
    for block in &chain.chain[start..] {
        // Check if the bit is set in nversion.
        if (block.nversion & (1u32 << bit)) != 0 {
            count += 1;
        }
    }
    count
}

/// Compute the soft fork state for a deployment at the current chain tip.
pub fn get_softfork_state(name: &str, chain: &Blockchain) -> SoftForkState {
    let deployment = match get_softfork_deployment(name) {
        Some(d) => d,
        None => return SoftForkState::Defined,
    };

    let height = chain.height();
    let mtp = median_time_past(chain);

    // If NEVER_ACTIVE, not scheduled.
    if deployment.start_time == NEVER_ACTIVE {
        return SoftForkState::Defined;
    }

    // If ALWAYS_ACTIVE, check min_activation_height.
    if deployment.start_time == ALWAYS_ACTIVE {
        if height >= deployment.min_activation_height {
            return SoftForkState::Active;
        } else {
            return SoftForkState::LockedIn;
        }
    }

    if mtp < deployment.start_time {
        return SoftForkState::Defined;
    }

    if mtp >= deployment.timeout {
        return SoftForkState::Failed;
    }

    let signals = count_signals(chain, deployment.bit, deployment.period);

    if signals >= deployment.threshold {
        if height >= deployment.min_activation_height {
            return SoftForkState::Active;
        } else {
            return SoftForkState::LockedIn;
        }
    }

    SoftForkState::Started
}

/// Returns `true` if the given soft fork deployment is active.
pub fn is_softfork_active(name: &str, chain: &Blockchain) -> bool {
    get_softfork_state(name, chain) == SoftForkState::Active
}

// ============================================================
// MEMPOOL CONSTANTS
// ============================================================

pub const MAX_MEMPOOL_SIZE_BYTES: usize = 300 * 1024 * 1024;
pub const MIN_RELAY_FEE_PER_BYTE: u64 = 1;
pub const MEMPOOL_EXPIRY_SECS: u64 = 336 * 3600;
pub const RBF_MIN_FEE_BUMP_PERCENT: u64 = 10;

// Checkpoint module for hardcoded block hashes.
pub mod checkpoint {
    // A single checkpoint entry.
    pub struct Checkpoint {
        pub height: u64, // The block height.
        pub hash: &'static str, // The expected block hash.
    }

    // The list of known checkpoints.
    pub const CHECKPOINTS: &[Checkpoint] = &[Checkpoint {
        height: 0, // Genesis checkpoint.
        hash: "0000000000000000000000000000000000000000000000000000000000000000", // Zero hash.
    }];

    // Finds the expected hash for a given height, if any.
    pub fn find(height: u64) -> Option<&'static str> {
        CHECKPOINTS.iter().find(|c| c.height == height).map(|c| c.hash) // Search the list.
    }

    // Checks whether a given height is a checkpoint.
    pub fn is_checkpoint(height: u64) -> bool {
        CHECKPOINTS.iter().any(|c| c.height == height) // Check if any checkpoint matches.
    }

    // Returns the highest checkpoint height.
    pub fn highest() -> u64 {
        CHECKPOINTS.iter().map(|c| c.height).max().unwrap_or(0) // Find the maximum height.
    }

    // Verifies that a block hash matches the checkpoint (if one exists).
    pub fn verify(height: u64, hash: &str) -> bool {
        match find(height) {
            Some(expected) => expected == hash, // If a checkpoint exists, compare hashes.
            None => true, // If no checkpoint exists, accept the block.
        }
    }
}

// DNS seeds for peer discovery.
pub mod dns_seed {
    use crate::consensus::spec; // For network selection.

    // Mainnet DNS seeds (currently using Bitcoin seeds for testing).
    pub const MAINNET_SEEDS: &[&str] = &[
        "seed.bitcoin.sipa.be", // Bitcoin seed (for testing).
        "dnsseed.bluematt.me", // Bitcoin seed (for testing).
    ];

    // Testnet DNS seeds (currently using Bitcoin testnet seeds for testing).
    pub const TESTNET_SEEDS: &[&str] = &[
        "testnet-seed.bitcoin.jonasschnelli.ch", // Bitcoin testnet seed (for testing).
    ];

    // Regtest does not use DNS seeds.
    pub const REGTEST_SEEDS: &[&str] = &[];

    // Returns the DNS seeds for the currently active network.
    pub fn seeds_for_current_network() -> &'static [&'static str] {
        match spec::current_network() {
            spec::Network::Main => MAINNET_SEEDS, // Mainnet seeds.
            spec::Network::Test => TESTNET_SEEDS, // Testnet seeds.
            spec::Network::Regtest => REGTEST_SEEDS, // Regtest seeds (empty).
        }
    }
}

// Network-related constants and helpers.
pub mod network {
    use crate::consensus::spec; // For network selection.

    // The height after which pruning is allowed.
    pub const PRUNE_AFTER_HEIGHT: u64 = 100_000;
    // The assumed blockchain size in MB (for pruning).
    pub const ASSUMED_BLOCKCHAIN_SIZE: u64 = 496;
    // The assumed chain state size in MB (for pruning).
    pub const ASSUMED_CHAIN_STATE_SIZE: u64 = 6;

    // Returns the name of the active network.
    pub fn network_name() -> &'static str {
        spec::current_network().name() // Fetch the name from the spec.
    }

    // Returns the default P2P port for the active network.
    pub fn default_port() -> u16 {
        spec::current_network().default_port() // Fetch the port from the spec.
    }

    // Returns the magic bytes of the active network as a hex string.
    pub fn network_magic_hex() -> String {
        spec::current_network().magic_hex() // Fetch the magic bytes from the spec.
    }
}

// Re-export consensus functions for convenience.
pub use consensus::consensus_fingerprint; // Consensus fingerprint.
pub use consensus::consensus_fingerprint_for; // Consensus fingerprint for a network.
pub use consensus::genesis_coinbase_txid; // Genesis coinbase txid.
pub use consensus::genesis_coinbase_txid_for; // Genesis coinbase txid for a network.
pub use genesis::genesis_message; // Genesis message.
pub use genesis::genesis_output_script; // Genesis output script.
pub use genesis::genesis_secret; // Genesis secret.
pub use genesis::genesis_timestamp; // Genesis timestamp.
pub use spec::coin_name; // Coin name.
pub use spec::decimals; // Decimal places.
pub use spec::difficulty_adjust_interval; // Difficulty adjustment interval.
pub use spec::hash_algo; // Hash algorithm.
pub use spec::initial_reward; // Initial reward.
pub use spec::initial_reward_zenac; // Initial reward in ZENAC.
pub use spec::max_halvings; // Maximum halvings.
pub use spec::max_supply; // Maximum supply.
pub use spec::max_supply_zenac; // Maximum supply in ZENAC.
pub use spec::miner_confirmation_window; // Miner confirmation window.
pub use spec::pow_target_spacing_secs; // PoW target spacing.
pub use spec::pow_target_timespan_secs; // PoW target timespan.
pub use spec::protocol_version; // Protocol version.
pub use spec::satoshi; // Satoshi unit.
pub use spec::signature_algo; // Signature algorithm.
pub use spec::subsidy_halving_interval; // Subsidy halving interval.
pub use spec::symbol; // Coin symbol.
pub use spec::version; // Version string.

// Returns the maximum number of transactions per block.
pub fn max_tx_per_block() -> usize {
    rules::MAX_TX_PER_BLOCK // Return the constant.
}

// Returns the PoW limit as a hex string.
pub fn pow_limit() -> &'static str {
    cpow::pow_limit() // Return the limit.
}

// Returns the genesis nBits.
pub fn genesis_nbits() -> u32 {
    spec::current_network().genesis_nbits() // Fetch from the spec.
}

// The Blockchain struct, representing the full chain state.
#[derive(Debug)] // Only `Debug` is derived.
pub struct Blockchain {
    pub chain: Vec<Block>, // The list of blocks.
    pub nbits: u32, // The current difficulty target.
    pub mempool: Vec<Transaction>, // The mempool.
    pub balances: HashMap<String, u64>, // Legacy balance map (unused).
    pub nonces: HashMap<String, u64>, // Legacy nonce map (unused).
    pub mempool_timestamps: HashMap<String, u64>, // Mempool timestamps.
    pub addresses: Vec<String>, // Known addresses.
    pub state: StateDB, // The state database.
}

impl Blockchain {
    // Creates a new Blockchain instance with the genesis block.
    pub fn new() -> Self {
        let genesis = Block::genesis(); // Create the genesis block.
        let db_path = crate::datadir::state_db_path(); // Get the state DB path.
        let mut state = StateDB::open(&db_path).expect("failed to open state db"); // Open the state DB.

        let genesis_acc = state.get_account("GENESIS").unwrap_or(None); // Check if the genesis account exists.
        if genesis_acc.is_none() {
            // If it does not exist, create it.
            state
                .update_account(
                    "GENESIS", // Genesis account address.
                    &Account {
                        nonce: 0, // Initial nonce.
                        balance: genesis.reward, // Initial balance (genesis reward).
                        immature_rewards: Vec::new(), // No immature rewards.
                    },
                )
                .expect("failed to init genesis account"); // Panic on failure.
        }

        // Fill the genesis state_root after initialization and
        // recalculate the hash to include the new state_root.
        let mut genesis = genesis; // Make the genesis block mutable.
        if let Ok(root) = state.root_hash() {
            // If the root hash is available...
            genesis.state_root = hex::encode(root); // Set the state_root.
            genesis.hash = genesis.calculate_hash(); // Recalculate the hash.
        }

        Blockchain {
            chain: vec![genesis], // Start with the genesis block.
            nbits: spec::current_network().genesis_nbits(), // Set the initial nbits.
            mempool: Vec::new(), // Empty mempool.
            balances: HashMap::new(), // Empty legacy balances.
            nonces: HashMap::new(), // Empty legacy nonces.
            mempool_timestamps: HashMap::new(), // Empty mempool timestamps.
            addresses: vec!["GENESIS".to_string()], // Start with the genesis address.
            state, // The state DB.
        }
    }

    // Returns the current chain height.
    pub fn height(&self) -> u64 {
        self.chain.last().unwrap().index // The index of the last block.
    }

    // Returns the circulating supply.
    pub fn circulating_supply(&self) -> u64 {
        self.chain
            .iter()
            .fold(0u64, |acc, b| acc.saturating_add(b.reward)) // Sum all block rewards.
    }

    // Calculates the reward for a given height.
    pub fn calculate_reward(height: u64) -> u64 {
        let halvings = (height / spec::SUBSIDY_HALVING_INTERVAL) as u32; // Compute the halving era.
        if halvings >= spec::MAX_HALVINGS {
            // If we have reached the maximum halvings...
            return 0; // The reward is zero.
        }
        spec::INITIAL_REWARD >> halvings // Otherwise, return the halved reward.
    }

    // Returns the current era (halving epoch).
    pub fn current_era(&self) -> u64 {
        self.height() / spec::SUBSIDY_HALVING_INTERVAL // Compute the era.
    }

    // Returns the number of blocks until the next halving.
    pub fn blocks_until_halving(&self) -> u64 {
        let next = (self.current_era() + 1) * spec::SUBSIDY_HALVING_INTERVAL; // Compute the next halving height.
        next.saturating_sub(self.height()) // Return the difference.
    }

    // Returns the total balance for an address.
    pub fn balance_of(&self, addr: &str) -> u64 {
        self.state.balance_of(addr) // Delegate to the state DB.
    }

    // Returns the spendable balance for an address.
    pub fn spendable_balance_of(&self, addr: &str) -> u64 {
        self.state.spendable_balance_of(addr) // Delegate to the state DB.
    }

    // Returns the nonce for an address.
    pub fn nonce_of(&self, addr: &str) -> u64 {
        self.state.nonce_of(addr) // Delegate to the state DB.
    }

    // Returns the next nonce for an address, accounting for the mempool.
    pub fn next_nonce_for(&self, addr: &str) -> u64 {
        let base = self.state.nonce_of(addr); // The base nonce from the state.
        let mut next = base; // Start with the base nonce.
        loop {
            let has = self
                .mempool
                .iter()
                .any(|tx| tx.from == addr && tx.nonce == next); // Check if the nonce is in the mempool.
            if !has {
                // If it is not in the mempool...
                return next; // This is the next nonce.
            }
            next = next.saturating_add(1); // Otherwise, increment and try again.
        }
    }

    // Applies a transaction to the state.
    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<(), String> {
        if !self.addresses.contains(&tx.from) {
            self.addresses.push(tx.from.clone()); // Track the sender address.
        }
        if !self.addresses.contains(&tx.to) {
            self.addresses.push(tx.to.clone()); // Track the recipient address.
        }
        self.state
            .apply_transaction(&tx.from, &tx.to, tx.amount, tx.fee, tx.nonce) // Delegate to the state DB.
    }

    // Credits a mining reward to an account.
    pub fn credit_reward(&mut self, miner: &str, reward: u64, block_height: u64) -> Result<(), String> {
        if !self.addresses.contains(&miner.to_string()) {
            self.addresses.push(miner.to_string()); // Track the miner address.
        }
        self.state.credit_reward(miner, reward, block_height) // Delegate to the state DB.
    }

    // Returns the total work of the current chain.
    pub fn total_chainwork(&self) -> u128 {
        self.chain.last().map(|b| b.chainwork).unwrap_or(0) // The chainwork of the last block.
    }

    // Returns the size of a transaction in bytes.
    pub fn tx_size_bytes(tx: &Transaction) -> usize {
        let base = 32 + tx.from.len() + tx.to.len() + 8 + 8 + 8 + tx.public_key.len() + tx.signature.len(); // Compute the base size.
        base + tx.note.len() // Add the note size.
    }

    // Returns the total size of the mempool in bytes.
    pub fn mempool_size_bytes(&self) -> usize {
        self.mempool.iter().map(|tx| Self::tx_size_bytes(tx)).sum() // Sum the sizes.
    }

    // Returns the fee rate of a transaction (fee per byte).
    pub fn fee_rate(tx: &Transaction) -> u64 {
        let size = Self::tx_size_bytes(tx).max(1) as u64; // Get the size (at least 1).
        tx.fee / size // Compute the fee rate.
    }

    // Evicts the lowest-fee transactions from the mempool to make room.
    fn evict_lowest_fee_transactions(&mut self, needed_space: usize) {
        self.mempool.sort_by(|a, b| {
            // Sort the mempool by fee rate.
            let fee_a = Self::fee_rate(a); // Fee rate of A.
            let fee_b = Self::fee_rate(b); // Fee rate of B.
            fee_a.cmp(&fee_b) // Compare.
        });

        let mut freed = 0; // Space freed so far.
        let mut to_remove = Vec::new(); // Indices to remove.

        for (i, tx) in self.mempool.iter().enumerate() {
            // Iterate over the mempool.
            if freed >= needed_space {
                break; // Stop if we have freed enough.
            }
            freed += Self::tx_size_bytes(tx); // Add the size of this tx.
            to_remove.push(i); // Mark it for removal.
        }

        for i in to_remove.into_iter().rev() {
            // Remove in reverse order to avoid index shifts.
            let removed = self.mempool.remove(i); // Remove the tx.
            self.mempool_timestamps.remove(&removed.txid); // Remove its timestamp.
            println!("[mempool] Evicted tx {} (fee rate {}) to make room", removed.txid, Self::fee_rate(&removed)); // Log the eviction.
        }
    }

    // Evicts expired transactions from the mempool.
    pub fn evict_expired_transactions(&mut self) {
        let now = (crate::utils::now_millis() / 1000) as u64; // Current time in seconds.
        let mut to_remove = Vec::new(); // Txids to remove.

        for tx in &self.mempool {
            // Iterate over the mempool.
            if let Some(&ts) = self.mempool_timestamps.get(&tx.txid) {
                // If the tx has a timestamp...
                if now.saturating_sub(ts) > MEMPOOL_EXPIRY_SECS {
                    // And it has expired...
                    to_remove.push(tx.txid.clone()); // Mark it for removal.
                }
            }
        }

        for txid in to_remove {
            // Remove the expired txs.
            self.mempool.retain(|tx| tx.txid != txid); // Remove from the mempool.
            self.mempool_timestamps.remove(&txid); // Remove its timestamp.
            println!("[mempool] Evicted expired tx {}", txid); // Log the eviction.
        }
    }

    // Adds a transaction to the mempool.
    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        if self.mempool.iter().any(|t| t.txid == tx.txid) {
            // If the tx is already in the mempool...
            return false; // Reject it.
        }

        let rbf_pos = self.mempool.iter().position(|t| t.from == tx.from && t.nonce == tx.nonce); // Find a tx with the same sender and nonce (RBF).

        if let Some(pos) = rbf_pos {
            // If an RBF candidate exists...
            let old_tx = &self.mempool[pos]; // The old tx.
            let old_fee_rate = Self::fee_rate(old_tx); // Its fee rate.
            let new_fee_rate = Self::fee_rate(&tx); // The new tx's fee rate.

            let min_new_fee_rate = old_fee_rate + (old_fee_rate * RBF_MIN_FEE_BUMP_PERCENT / 100).max(1); // Compute the minimum fee bump.

            if new_fee_rate >= min_new_fee_rate {
                // If the new fee rate is sufficient...
                println!(
                    "[mempool] RBF: Replacing tx {} (fee rate {}) with {} (fee rate {})",
                    old_tx.txid, old_fee_rate, tx.txid, new_fee_rate // Log the replacement.
                );
                self.mempool_timestamps.remove(&old_tx.txid); // Remove the old timestamp.
                self.mempool.remove(pos); // Remove the old tx.
            } else {
                // If the new fee rate is insufficient...
                println!(
                    "[mempool] Rejected RBF attempt: new fee rate {} < required {}",
                    new_fee_rate, min_new_fee_rate // Log the rejection.
                );
                return false; // Reject the new tx.
            }
        }

        if validation::validate_transaction(self, &tx).is_err() {
            // If the tx is invalid...
            println!("[mempool] Rejected tx {}: validation failed", tx.txid); // Log the rejection.
            return false; // Reject it.
        }

        let tx_size = Self::tx_size_bytes(&tx); // The tx size.
        let fee_rate = Self::fee_rate(&tx); // The tx fee rate.

        if fee_rate < MIN_RELAY_FEE_PER_BYTE {
            // If the fee rate is below the minimum...
            println!("[mempool] Rejected tx {}: fee rate {} < minRelayTxFee {}", tx.txid, fee_rate, MIN_RELAY_FEE_PER_BYTE); // Log the rejection.
            return false; // Reject it.
        }

        if self.mempool_size_bytes() + tx_size > MAX_MEMPOOL_SIZE_BYTES {
            // If the mempool is full...
            self.evict_lowest_fee_transactions(tx_size); // Evict low-fee txs.
        }

        if self.mempool_size_bytes() + tx_size > MAX_MEMPOOL_SIZE_BYTES {
            // If it is still full...
            return false; // Reject the tx.
        }

        if self.mempool.len() >= rules::MAX_TX_PER_BLOCK {
            // If the mempool has too many txs...
            return false; // Reject the tx.
        }

        let now = (crate::utils::now_millis() / 1000) as u64; // Current time.
        self.mempool_timestamps.insert(tx.txid.clone(), now); // Store the timestamp.
        self.mempool.push(tx); // Add the tx to the mempool.
        true // Accept the tx.
    }

    // Mines all pending transactions in the mempool.
    pub fn mine_pending(&mut self) -> Result<u64, String> {
        let prev = self.chain.last().unwrap().clone(); // The previous block.
        let next_index = prev.index + 1; // The next block index.
        let subsidy = Self::calculate_reward(next_index); // The block subsidy.
        let total_fees: u64 = self.mempool.iter().map(|tx| tx.fee).sum(); // The total fees.
        let reward = subsidy.saturating_add(total_fees); // The total reward.

        if self.circulating_supply().saturating_add(reward) > spec::MAX_SUPPLY {
            // If the max supply would be exceeded...
            return Err("max supply reached".into()); // Return an error.
        }

        let txs = std::mem::take(&mut self.mempool); // Take the mempool txs.
        self.mempool_timestamps.clear(); // Clear the timestamps.

        for tx in &txs {
            // Apply all txs to the state.
            self.apply_transaction(tx)?; // Error out on failure.
        }
        self.credit_reward(&prev.mined_for, reward, next_index)?; // Credit the reward.

        let state_root = hex::encode(self.state.root_hash().map_err(|e| e.to_string())?); // Compute the state root.

        let mut new_block = Block::new_for(
            next_index, // The new index.
            prev.hash.clone(), // The previous hash.
            subsidy, // The subsidy.
            total_fees, // The total fees.
            self.nbits, // The current nbits.
            txs.clone(), // The transactions.
            String::new(), // The mined_for address (empty for now).
            state_root, // The state root.
            prev.chainwork, // The previous chainwork.
        );
        if !pow::mine(&mut new_block, self.nbits) {
            // Mine the block.
            return Err("mining failed: nonce exhausted".into()); // Return an error on failure.
        }

        let nonce = new_block.nonce; // The block nonce.
        self.chain.push(new_block); // Add the block to the chain.
        self.adjust_difficulty(); // Adjust the difficulty.
        Ok(nonce) // Return the nonce.
    }

    // Adjusts the difficulty based on the last interval.
    pub fn adjust_difficulty(&mut self) {
        if spec::current_network() == spec::Network::Regtest {
            // Regtest does not adjust difficulty.
            return;
        }

        let height = self.height(); // The current height.
        if height == 0 || !height.is_multiple_of(spec::DIFFICULTY_ADJUST_INTERVAL) {
            // If we are not at an adjustment interval...
            return; // Do nothing.
        }

        let interval = spec::DIFFICULTY_ADJUST_INTERVAL as usize; // The adjustment interval.
        let last = &self.chain[height as usize]; // The last block.
        let first = &self.chain[height as usize - interval]; // The first block in the interval.

        let actual_time_ms = last.timestamp.saturating_sub(first.timestamp) as u64; // The actual time taken.
        let actual_time_secs = actual_time_ms / 1000; // Convert to seconds.
        let expected_time_secs = spec::POW_TARGET_TIMESPAN_SECS; // The expected time.

        if actual_time_secs == 0 {
            // If no time has passed...
            return; // Do nothing.
        }

        let current_target = pow::nbits_to_target(self.nbits); // The current target.
        let scale = (actual_time_secs as u128) * 256 / (expected_time_secs as u128); // The scale factor.
        let new_target = scale_target(&current_target, scale); // Scale the target.
        let new_nbits = target_to_nbits(&new_target); // Convert back to nbits.

        if new_nbits != self.nbits {
            // If the nbits has changed...
            self.nbits = new_nbits; // Update the nbits.
        }
    }

    // Validates the entire chain.
    pub fn is_valid(&self) -> bool {
        validation::validate_chain(self).is_ok() // Delegate to the validation module.
    }
}

impl Default for Blockchain {
    // Default implementation for Blockchain.
    fn default() -> Self {
        Self::new() // Create a new Blockchain.
    }
}

// Scales a 256-bit target by a 256-based factor.
fn scale_target(target: &[u8; 32], scale_256: u128) -> [u8; 32] {
    let mut result = [0u8; 32]; // The result array.
    let mut carry: u128 = 0; // The carry value.

    for i in (0..32).rev() {
        // Iterate from the least significant byte.
        let v = (target[i] as u128) * scale_256 + carry; // Multiply and add carry.
        result[i] = (v & 0xff) as u8; // Store the low byte.
        carry = v >> 8; // Store the carry.
    }
    result // Return the result.
}

// Converts a 256-bit target to nBits.
fn target_to_nbits(target: &[u8; 32]) -> u32 {
    let mut i = 0; // The index of the first non-zero byte.
    while i < 32 && target[i] == 0 {
        i += 1; // Skip leading zeros.
    }
    if i == 32 {
        // If the target is zero...
        return spec::current_network().genesis_nbits(); // Return the genesis nbits.
    }

    let size = 32 - i; // The size of the target.
    let mut mantissa: u32 = (target[i] as u32) << 16; // The mantissa (3 bytes).
    if i + 1 < 32 {
        mantissa |= (target[i + 1] as u32) << 8; // Add the second byte.
    }
    if i + 2 < 32 {
        mantissa |= target[i + 2] as u32; // Add the third byte.
    }

    if mantissa & 0x00800000 == 0 {
        // If the sign bit is not set...
        mantissa <<= 8; // Shift the mantissa.
        return (((size - 1) as u32) << 24) | (mantissa & 0x00ffffff); // Return the nbits.
    }

    ((size as u32) << 24) | (mantissa & 0x00ffffff) // Return the nbits.
}
