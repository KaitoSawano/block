use crate::blockchain::{self, Blockchain};
use crate::datadir;
use crate::pow;
use crate::transaction::Transaction;
use crate::utils::{format_datetime, now_millis, sha3_256_hex};
use serde::{Deserialize, Serialize};

pub const BLOCK_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub nversion: u32,
    pub index: u64,
    pub timestamp: u128,
    pub mined_at: String,
    pub mined_for: String,
    pub previous_hash: String,
    pub merkle_root: String,
    pub hash: String,
    pub nonce: u64,
    pub nbits: u32,
    pub reward: u64,
    pub coinbase_txid: String,
    pub blockchain_size: u64,
    pub genesis_output_script: String,
    pub miner_pubkey: String,
    pub miner_signature: String,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn compute_merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return sha3_256_hex(b"empty");
        }
        let mut payload = String::new();
        for tx in transactions {
            payload.push_str(&tx.txid);
            payload.push('|');
        }
        sha3_256_hex(payload.as_bytes())
    }

    pub fn new(
        index: u64,
        previous_hash: String,
        reward: u64,
        nbits: u32,
        transactions: Vec<Transaction>,
    ) -> Self {
        Self::new_for(index, previous_hash, reward, nbits, transactions, String::new())
    }

    pub fn new_for(
        index: u64,
        previous_hash: String,
        reward: u64,
        nbits: u32,
        transactions: Vec<Transaction>,
        mined_for: String,
    ) -> Self {
        let ts = now_millis();
        let coinbase_txid =
            sha3_256_hex(format!("{}|{}|{}", index, mined_for, ts).as_bytes());
        let merkle_root = Self::compute_merkle_root(&transactions);

        // Perkiraan ukuran payload blok.
        let size = (previous_hash.len()
            + merkle_root.len()
            + coinbase_txid.len()
            + transactions.iter().map(|t| t.txid.len()).sum::<usize>()
            + 256) as u64;

        let mut b = Block {
            nversion: BLOCK_VERSION,
            index,
            timestamp: ts,
            mined_at: format_datetime(ts),
            mined_for,
            previous_hash,
            merkle_root,
            hash: String::new(),
            nonce: 0,
            nbits,
            reward,
            coinbase_txid,
            blockchain_size: size,
            genesis_output_script: String::new(),
            miner_pubkey: String::new(),
            miner_signature: String::new(),
            transactions,
        };
        b.hash = b.calculate_hash();
        b
    }

    /// Genesis block. Semua field terikat ke fingerprint konsensus.
    pub fn genesis() -> Self {
        let ts = blockchain::genesis_timestamp();
        let coinbase_txid = blockchain::genesis_coinbase_txid();
        let genesis_output_script = blockchain::genesis_output_script();

        let transactions = vec![
            Transaction::coinbase(blockchain::genesis_message().into()),
            Transaction::secret(blockchain::genesis_secret().into()),
        ];
        let merkle_root = Self::compute_merkle_root(&transactions);

        let size = (genesis_output_script.len()
            + merkle_root.len()
            + coinbase_txid.len()
            + transactions.iter().map(|t| t.txid.len()).sum::<usize>()
            + 256) as u64;

        let mut b = Block {
            nversion: BLOCK_VERSION,
            index: 0,
            timestamp: ts,
            mined_at: format_datetime(ts),
            mined_for: "GENESIS".to_string(),
            previous_hash: "0".repeat(64),
            merkle_root,
            hash: String::new(),
            nonce: 0,
            nbits: pow::GENESIS_NBITS,
            reward: blockchain::initial_reward(),
            coinbase_txid,
            blockchain_size: size,
            genesis_output_script,
            miner_pubkey: String::new(),
            miner_signature: String::new(),
            transactions,
        };
        b.hash = b.calculate_hash();
        b
    }

    /// Hash blok. Memasukkan semua field, termasuk semua field transaksi
    /// (from, to, amount, fee, note, inputs, public_key, signature).
    /// Perubahan sekecil apa pun di field ini akan mengubah hash.
    pub fn calculate_hash(&self) -> String {
        let mut payload = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.nversion,
            self.index,
            self.timestamp,
            self.mined_at,
            self.mined_for,
            self.previous_hash,
            self.merkle_root,
            self.nonce,
            self.nbits,
            self.reward,
            self.coinbase_txid,
            self.blockchain_size,
            self.genesis_output_script,
            self.miner_pubkey,
            self.miner_signature,
            self.transactions.len()
        );
        for tx in &self.transactions {
            payload.push_str(&format!(
                "\n{}|{}|{}|{}|{}|{}|{}|{}",
                tx.from,
                tx.to,
                tx.amount,
                tx.fee,
                tx.note,
                tx.inputs.join(","),
                tx.public_key,
                tx.signature
            ));
        }
        sha3_256_hex(payload.as_bytes())
    }

    pub fn reward_at(height: u64) -> u64 {
        Blockchain::calculate_reward(height)
    }

    pub fn log_add_to_wallet(&self) {
        let entry = format!(
            "AddToWallet\n  mined_at : {}\n  txid     : {}\n  hash     : {}",
            self.mined_at, self.coinbase_txid, self.hash
        );

        println!("{}", entry);
        datadir::append_debug_log(&entry);
    }
}
