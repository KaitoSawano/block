// Placeholder mempool ZENAC.
// Untuk saat ini mempool disimpan langsung di Blockchain.mempool.

use crate::transaction::Transaction;

#[derive(Debug, Default)]
pub struct Mempool {
    pub txs: Vec<Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Mempool { txs: Vec::new() }
    }

    pub fn add(&mut self, tx: Transaction) {
        self.txs.push(tx);
    }

    pub fn drain(&mut self) -> Vec<Transaction> {
        std::mem::take(&mut self.txs)
    }
}
