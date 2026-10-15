use crate::blockchain::{
    GENESIS_MESSAGE, GENESIS_SECRET, GENESIS_TIMESTAMP, INITIAL_REWARD,
};
use crate::transaction::Transaction;
use crate::utils::{now_millis, sha3_256_hex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub reward: u64,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(
        index: u64,
        previous_hash: String,
        reward: u64,
        transactions: Vec<Transaction>,
    ) -> Self {
        let mut b = Block {
            index,
            timestamp: now_millis(),
            previous_hash,
            hash: String::new(),
            nonce: 0,
            reward,
            transactions,
        };
        b.hash = b.calculate_hash();
        b
    }

    /// Genesis ZENAC dengan pesan rahasia "ID Today 15/Oct/2026".
    pub fn genesis() -> Self {
        let mut b = Block {
            index: 0,
            timestamp: GENESIS_TIMESTAMP,
            previous_hash: "0".repeat(64),
            hash: String::new(),
            nonce: 0,
            reward: INITIAL_REWARD,
            transactions: vec![
                Transaction::coinbase(GENESIS_MESSAGE.into()),
                Transaction::secret(GENESIS_SECRET.into()),
            ],
        };
        b.hash = b.calculate_hash();
        b
    }

    /// Hitung hash blok dengan SHA3-256.
    /// Transaksi ikut di-hash sehingga pesan genesis terikat permanen.
    pub fn calculate_hash(&self) -> String {
        let mut payload = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.previous_hash, self.nonce, self.reward
        );
        for tx in &self.transactions {
            payload.push_str(&format!(
                "|{}|{}|{}|{}|{}",
                tx.from, tx.to, tx.amount, tx.fee, tx.note
            ));
        }
        sha3_256_hex(payload.as_bytes())
    }
}
