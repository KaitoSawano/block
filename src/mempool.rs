use crate::transaction::Transaction;

#[derive(Debug, Default)]
pub struct Mempool {
    pub txs: Vec<Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Mempool { txs: Vec::new() }
    }

    pub fn add(&mut self, tx: Transaction) -> bool {
        if tx.is_valid() {
            self.txs.push(tx);
            true
        } else {
            false
        }
    }

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
