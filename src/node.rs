// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use crate::blockchain::Blockchain;
use crate::config::Config;
use crate::datadir;
use crate::storage;

pub struct Node {
    pub chain: Blockchain,
    pub config: Config,
}

impl Node {
    pub fn new() -> Self {
        let _ = datadir::ensure_dirs();
        let config = Config::load_or_default();
        let chain = storage::load_chain().unwrap_or_else(|_| Blockchain::new());
        Node { chain, config }
    }

    pub fn mining_address(&self) -> &str {
        &self.config.mining_address
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}
