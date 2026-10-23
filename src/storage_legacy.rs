// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Legacy storage functions (blocks, mempool, state snapshot).

use std::collections::HashMap; // For the state snapshot.
use std::io::Error; // For error handling.

use crate::block::Block; // The block structure.
use crate::blockchain::Blockchain; // The blockchain struct.
use crate::datadir; // Data directory helpers.
use crate::storage::Account; // The account structure.

// The number of blocks per file on disk.
pub const BLOCKS_PER_FILE: usize = 1000;

// Saves the chain to disk (blocks, mempool, and state snapshot).
pub fn save_chain(chain: &Blockchain) -> std::io::Result<()> {
    datadir::ensure_dirs()?; // Ensure the data directory exists.
    save_blocks(chain)?; // Save the blocks.
    save_mempool(&chain.mempool)?; // Save the mempool.
    save_state_snapshot(chain)?; // Save the state snapshot.
    Ok(()) // Success.
}

// Loads the chain from disk.
pub fn load_chain() -> std::io::Result<Blockchain> {
    let db_path = datadir::state_db_path(); // Get the state DB path.

    // Try to load blocks from disk.
    let blocks = load_blocks(0)?; // Load all blocks.
    if blocks.is_empty() {
        // If there are no blocks...
        return Ok(Blockchain::new()); // Return a new chain.
    }

    // Verify the genesis block structure. We do NOT call `Block::genesis()`
    // here because that would re-mine the genesis block and could produce a
    // different hash if the state_root has changed. Instead, we rely on the
    // structural invariants of the genesis block (index 0, zero previous_hash).
    if blocks[0].index != 0 {
        // The genesis block must have index 0.
        return Err(Error::other(
            "consensus failure: the first block on disk is not the genesis block.",
        ));
    }
    if blocks[0].previous_hash != crate::consensus::rules::ZERO_HASH {
        // The genesis block must have a zero previous_hash.
        return Err(Error::other(
            "consensus failure: the genesis block previous_hash is not zero.",
        ));
    }

    let mempool = load_mempool().unwrap_or_default(); // Load the mempool.

    // Open the state DB.
    let state = crate::storage::StateDB::open(&db_path)
        .map_err(|e| Error::other(format!("failed to open state db: {}", e)))?; // Open the state DB.

    let mut chain = Blockchain {
        chain: blocks, // The loaded blocks.
        nbits: crate::consensus::spec::current_network().genesis_nbits(), // The initial nbits.
        mempool, // The loaded mempool.
        balances: HashMap::new(), // Empty legacy balances.
        nonces: HashMap::new(), // Empty legacy nonces.
        mempool_timestamps: HashMap::new(), // Empty mempool timestamps.
        addresses: Vec::new(), // Empty addresses.
        state, // The state DB.
    };

    // Load state snapshot from JSON if LevelDB is empty.
    load_state_snapshot(&mut chain)?; // Load the state snapshot.

    Ok(chain) // Return the loaded chain.
}

// Saves the state snapshot to a JSON file.
pub fn save_state_snapshot(chain: &Blockchain) -> std::io::Result<()> {
    let mut accounts: HashMap<String, Account> = HashMap::new(); // The accounts to save.
    for addr in &chain.addresses {
        // Iterate over the known addresses.
        if let Ok(Some(acc)) = chain.state.get_account(addr) {
            // If the account exists...
            accounts.insert(addr.clone(), acc); // Add it to the map.
        }
    }
    let json = serde_json::to_string_pretty(&accounts).map_err(Error::other)?; // Serialize to JSON.
    std::fs::write(datadir::chainstate_dir().join("state_snapshot.json"), json) // Write to disk.
}

// Loads the state snapshot from a JSON file.
pub fn load_state_snapshot(chain: &mut Blockchain) -> std::io::Result<()> {
    let path = datadir::chainstate_dir().join("state_snapshot.json"); // The snapshot path.
    if !path.exists() {
        // If the snapshot does not exist...
        return Ok(()); // Nothing to load.
    }

    let data = std::fs::read_to_string(&path)?; // Read the snapshot.
    // Attempt to parse the snapshot. If the format is incompatible
    // (e.g., from an older Account structure), silently ignore it.
    let accounts: HashMap<String, Account> = match serde_json::from_str(&data) {
        Ok(a) => a, // Parse the snapshot.
        Err(_) => {
            // Incompatible snapshot format; skip loading.
            return Ok(());
        }
    };

    for (addr, acc) in accounts {
        // Iterate over the loaded accounts.
        // Only insert if not already present in LevelDB.
        if chain.state.get_account(&addr).unwrap_or(None).is_none() {
            // If the account is not in LevelDB...
            chain
                .state
                .update_account(&addr, &acc)
                .map_err(Error::other)?; // Insert it.
        }
    }

    Ok(()) // Success.
}

// Saves the blocks to disk, splitting them into multiple files.
pub fn save_blocks(chain: &Blockchain) -> std::io::Result<()> {
    let total = chain.chain.len(); // The total number of blocks.
    let mut file_index = 0u32; // The current file index.
    let mut written = 0usize; // The number of blocks written so far.
    while written < total {
        // While there are blocks left to write...
        let end = (written + BLOCKS_PER_FILE).min(total); // Compute the end index.
        let slice = &chain.chain[written..end]; // Get the slice of blocks.
        let path = datadir::block_file(file_index); // Get the file path.
        let json = serde_json::to_string_pretty(slice).map_err(Error::other)?; // Serialize to JSON.
        std::fs::write(path, json)?; // Write to disk.
        written = end; // Update the written count.
        file_index += 1; // Increment the file index.
    }
    Ok(()) // Success.
}

// Loads all blocks from disk.
pub fn load_blocks(_total_blocks: u64) -> std::io::Result<Vec<Block>> {
    let mut all: Vec<Block> = Vec::new(); // The list of all blocks.
    let mut i = 0u32; // The file index.
    loop {
        let path = datadir::block_file(i); // Get the file path.
        if !path.exists() {
            // If the file does not exist...
            break; // Stop loading.
        }
        let data = std::fs::read_to_string(&path)?; // Read the file.
        let mut chunk: Vec<Block> = serde_json::from_str(&data).map_err(Error::other)?; // Parse the blocks.
        all.append(&mut chunk); // Append to the list.
        i += 1; // Increment the file index.
    }
    Ok(all) // Return all blocks.
}

// Saves the mempool to disk.
pub fn save_mempool(mempool: &[crate::transaction::Transaction]) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(mempool).map_err(Error::other)?; // Serialize to JSON.
    std::fs::write(datadir::mempool_path(), json) // Write to disk.
}

// Loads the mempool from disk.
pub fn load_mempool() -> std::io::Result<Vec<crate::transaction::Transaction>> {
    let data = std::fs::read_to_string(datadir::mempool_path())?; // Read the file.
    let v: Vec<crate::transaction::Transaction> =
        serde_json::from_str(&data).map_err(Error::other)?; // Parse the mempool.
    Ok(v) // Return the mempool.
}
