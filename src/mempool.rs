// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Mempool representation.

use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Mempool {
    pub txs: Vec<Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Mempool { txs: Vec::new() }
    }

    /// Add a transaction to the mempool if it is valid.
    pub fn add(&mut self, tx: Transaction) -> bool {
        if tx.is_valid() {
            self.txs.push(tx);
            true
        } else {
            false
        }
    }

    /// Remove and return all transactions from the mempool.
    pub fn drain(&mut self) -> Vec<Transaction> {
        std::mem::take(&mut self.txs)
    }

    pub fn len(&self) -> usize {
        self.txs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.txs.is_empty()
    }
}
