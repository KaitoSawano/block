// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Persistent storage for the blockchain state.

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::datadir;
use crate::mempool::Mempool;
use std::io::Error;

pub const BLOCKS_PER_FILE: usize = 1000;

/// Save the blockchain state to disk.
pub fn save_chain(chain: &Blockchain) -> std::io::Result<()> {
    datadir::ensure_dirs()?;
    save_blocks(chain)?;
    save_chainstate(chain)?;
    save_mempool(&chain.mempool)?;
    Ok(())
}

/// Load the blockchain state from disk.
pub fn load_chain() -> std::io::Result<Blockchain> {
    if !datadir::chainstate_file().exists() {
        return Ok(Blockchain::new());
    }
    let state = load_chainstate()?;
    let blocks = load_blocks(state.total_blocks)?;
    if blocks.is_empty() {
        return Ok(Blockchain::new());
    }
    let mempool = load_mempool().unwrap_or_default();
    Ok(Blockchain {
        chain: blocks,
        nbits: state.nbits,
        mempool,
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChainState {
    pub total_blocks: u64,
    pub nbits: u32,
    pub updated_at: u128,
}

/// Save the chain state (block count and nbits) to disk.
pub fn save_chainstate(chain: &Blockchain) -> std::io::Result<()> {
    let state = ChainState {
        total_blocks: chain.chain.len() as u64,
        nbits: chain.nbits,
        updated_at: crate::utils::now_millis(),
    };
    let json = serde_json::to_string_pretty(&state).map_err(Error::other)?;
    std::fs::write(datadir::chainstate_file(), json)
}

/// Load the chain state from disk.
pub fn load_chainstate() -> std::io::Result<ChainState> {
    let data = std::fs::read_to_string(datadir::chainstate_file())?;
    let s: ChainState = serde_json::from_str(&data).map_err(Error::other)?;
    Ok(s)
}

/// Write the block files to disk.
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

/// Read all block files from disk.
pub fn load_blocks(total_blocks: u64) -> std::io::Result<Vec<Block>> {
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
    all.truncate(total_blocks as usize);
    Ok(all)
}

/// Write the mempool to disk.
pub fn save_mempool(mempool: &[crate::transaction::Transaction]) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(mempool).map_err(Error::other)?;
    std::fs::write(datadir::mempool_path(), json)
}

/// Read the mempool from disk.
pub fn load_mempool() -> std::io::Result<Vec<crate::transaction::Transaction>> {
    let data = std::fs::read_to_string(datadir::mempool_path())?;
    let v: Vec<crate::transaction::Transaction> =
        serde_json::from_str(&data).map_err(Error::other)?;
    Ok(v)
}

/// Write the mempool struct to disk.
pub fn save_mempool_struct(mempool: &Mempool) -> std::io::Result<()> {
    save_mempool(&mempool.txs)
}
