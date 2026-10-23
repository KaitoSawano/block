// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Legacy storage functions (blocks, mempool, state snapshot).

use std::collections::HashMap;
use std::io::Error;

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::datadir;
use crate::storage::Account;

pub const BLOCKS_PER_FILE: usize = 1000;

pub fn save_chain(chain: &Blockchain) -> std::io::Result<()> {
    datadir::ensure_dirs()?;
    save_blocks(chain)?;
    save_mempool(&chain.mempool)?;
    save_state_snapshot(chain)?;
    Ok(())
}

pub fn load_chain() -> std::io::Result<Blockchain> {
    let db_path = datadir::state_db_path();

    // Try to load blocks from disk.
    let blocks = load_blocks(0)?;
    if blocks.is_empty() {
        return Ok(Blockchain::new());
    }

    // Verify genesis.
    let expected_genesis = Block::genesis();
    if blocks[0].hash != expected_genesis.hash {
        return Err(Error::other(
            "consensus failure: the chain on disk does not match the current genesis block.",
        ));
    }

    let mempool = load_mempool().unwrap_or_default();

    // Open the state DB.
    let state = crate::storage::StateDB::open(&db_path)
        .map_err(|e| Error::other(format!("failed to open state db: {}", e)))?;

    let mut chain = Blockchain {
        chain: blocks,
        nbits: crate::consensus::spec::current_network().genesis_nbits(),
        mempool,
        balances: HashMap::new(),
        nonces: HashMap::new(),
        mempool_timestamps: HashMap::new(),
        addresses: Vec::new(),
        state,
    };

    // Load state snapshot from JSON if LevelDB is empty.
    load_state_snapshot(&mut chain)?;

    Ok(chain)
}

pub fn save_state_snapshot(chain: &Blockchain) -> std::io::Result<()> {
    let mut accounts: HashMap<String, Account> = HashMap::new();
    for addr in &chain.addresses {
        if let Ok(Some(acc)) = chain.state.get_account(addr) {
            accounts.insert(addr.clone(), acc);
        }
    }
    let json = serde_json::to_string_pretty(&accounts).map_err(Error::other)?;
    std::fs::write(datadir::chainstate_dir().join("state_snapshot.json"), json)
}

pub fn load_state_snapshot(chain: &mut Blockchain) -> std::io::Result<()> {
    let path = datadir::chainstate_dir().join("state_snapshot.json");
    if !path.exists() {
        return Ok(());
    }

    let data = std::fs::read_to_string(&path)?;
    // Attempt to parse the snapshot. If the format is incompatible
    // (e.g., from an older Account structure), silently ignore it.
    let accounts: HashMap<String, Account> = match serde_json::from_str(&data) {
        Ok(a) => a,
        Err(_) => {
            // Incompatible snapshot format; skip loading.
            return Ok(());
        }
    };

    for (addr, acc) in accounts {
        // Only insert if not already present in LevelDB.
        if chain.state.get_account(&addr).unwrap_or(None).is_none() {
            chain
                .state
                .update_account(&addr, &acc)
                .map_err(Error::other)?;
        }
    }

    Ok(())
}

pub fn save_blocks(chain: &Blockchain) -> std::io::Result<()> {
    let total = chain.chain.len();
    let mut file_index = 0u32;
    let mut written = 0usize;
    while written < total {
        let end = (written + BLOCKS_PER_FILE).min(total);
        let slice = &chain.chain[written..end];
        let path = datadir::block_file(file_index);
        let json = serde_json::to_string_pretty(slice).map_err(Error::other)?;
        std::fs::write(path, json)?;
        written = end;
        file_index += 1;
    }
    Ok(())
}

pub fn load_blocks(_total_blocks: u64) -> std::io::Result<Vec<Block>> {
    let mut all: Vec<Block> = Vec::new();
    let mut i = 0u32;
    loop {
        let path = datadir::block_file(i);
        if !path.exists() {
            break;
        }
        let data = std::fs::read_to_string(&path)?;
        let mut chunk: Vec<Block> = serde_json::from_str(&data).map_err(Error::other)?;
        all.append(&mut chunk);
        i += 1;
    }
    Ok(all)
}

pub fn save_mempool(mempool: &[crate::transaction::Transaction]) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(mempool).map_err(Error::other)?;
    std::fs::write(datadir::mempool_path(), json)
}

pub fn load_mempool() -> std::io::Result<Vec<crate::transaction::Transaction>> {
    let data = std::fs::read_to_string(datadir::mempool_path())?;
    let v: Vec<crate::transaction::Transaction> =
        serde_json::from_str(&data).map_err(Error::other)?;
    Ok(v)
}
