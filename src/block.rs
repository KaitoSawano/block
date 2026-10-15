use crate::blockchain::{self, Blockchain};
use crate::transaction::Transaction;
use crate::utils::{format_datetime, now_millis, sha3_256_hex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub mined_at: String,
    pub mined_for: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub reward: u64,
    pub coinbase_txid: String,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(
        index: u64,
        previous_hash: String,
        reward: u64,
        transactions: Vec<Transaction>,
    ) -> Self {
        Self::new_for(index, previous_hash, reward, transactions, String::new())
    }

    pub fn new_for(
        index: u64,
        previous_hash: String,
        reward: u64,
        transactions: Vec<Transaction>,
        mined_for: String,
    ) -> Self {
        let ts = now_millis();
        let coinbase_txid = sha3_256_hex(
            format!("{}|{}|{}", index, mined_for, ts).as_bytes(),
        );
        let mut b = Block {
            index,
            timestamp: ts,
            mined_at: format_datetime(ts),
            mined_for,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            reward,
            coinbase_txid,
            transactions,
        };
        b.hash = b.calculate_hash();
        b
    }

    pub fn genesis() -> Self {
        let ts = blockchain::genesis_timestamp();
        let coinbase_txid = sha3_256_hex(format!("genesis|{}", ts).as_bytes());
        let mut b = Block {
            index: 0,
            timestamp: ts,
            mined_at: format_datetime(ts),
            mined_for: "GENESIS".to_string(),
            previous_hash: "0".repeat(64),
            hash: String::new(),
            nonce: 0,
            reward: blockchain::initial_reward(),
            coinbase_txid,
            transactions: vec![
                Transaction::coinbase(blockchain::genesis_message().into()),
                Transaction::secret(blockchain::genesis_secret().into()),
            ],
        };
        b.hash = b.calculate_hash();
        b
    }

    pub fn calculate_hash(&self) -> String {
        let mut payload = format!(
            "{}{}{}{}{}{}{}{}",
            self.index,
            self.timestamp,
            self.mined_at,
            self.mined_for,
            self.previous_hash,
            self.nonce,
            self.reward,
            self.coinbase_txid
        );
        for tx in &self.transactions {
            payload.push_str(&format!("|{}", tx.txid));
        }
        sha3_256_hex(payload.as_bytes())
    }

    pub fn reward_at(height: u64) -> u64 {
        Blockchain::calculate_reward(height)
    }
}
