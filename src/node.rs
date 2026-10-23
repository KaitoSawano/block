// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Node state container.

use crate::blockchain::Blockchain;
use crate::config::Config;
use crate::datadir;
use crate::storage;

pub struct Node {
    pub chain: Blockchain,
    pub config: Config,
}

impl Node {
    /// Create a new node by loading the configuration and chain from disk.
    pub fn new() -> Self {
        let _ = datadir::ensure_dirs();
        let config = Config::load_or_default();
        let chain = storage::load_chain().unwrap_or_else(|_| Blockchain::new());
        Node { chain, config }
    }

    /// Return the configured mining address.
    pub fn mining_address(&self) -> &str {
        &self.config.mining_address
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}
