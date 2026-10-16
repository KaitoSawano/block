use crate::blockchain::{self, Blockchain};
use crate::consensus::{genesis, pow as cpow, rules};
use crate::datadir;
use crate::pow;
use crate::transaction::Transaction;
use crate::utils::{format_datetime, now_millis, sha3_256_hex};
use serde::{Deserialize, Serialize};

pub const BLOCK_VERSION: u32 = rules::BLOCK_VERSION;

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

        let mut layer: Vec<[u8; 32]> = transactions
            .iter()
            .filter_map(|tx| {
                let bytes = hex::decode(&tx.txid).ok()?;
                if bytes.len() != 32 {
                    return None;
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Some(arr)
            })
            .collect();

        if layer.is_empty() {
            return sha3_256_hex(b"empty");
        }

        while layer.len() > 1 {
            if layer.len() % 2 == 1 {
                let last = *layer.last().unwrap();
                layer.push(last);
            }

            let mut next: Vec<[u8; 32]> = Vec::with_capacity(layer.len() / 2);
            for pair in layer.chunks(2) {
                let mut buf = [0u8; 64];
                buf[..32].copy_from_slice(&pair[0]);
                buf[32..].copy_from_slice(&pair[1]);
                let hash = sha3_256_hex(&buf);
                let bytes = hex::decode(&hash).unwrap();
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                next.push(arr);
            }
            layer = next;
        }

        hex::encode(layer[0])
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

        let size = (previous_hash.len()
            + merkle_root.len()
            + coinbase_txid.len()
            + transactions.iter().map(|t| t.txid.len()).sum::<usize>()
            + 256) as u64;

        let mut b = Block {
            nversion: rules::BLOCK_VERSION,
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

    /// Genesis block. Menambang genesis, lalu `assert!` terhadap konstanta
    /// `EXPECTED_GENESIS_*` di `src/consensus/genesis.rs`.
    ///
    /// Tidak mencetak apa pun. Untuk melihat nilai genesis, jalankan
    /// binary `zenachain-genesis`.
    pub fn genesis() -> Self {
        let ts = genesis::GENESIS_TIMESTAMP;
        let coinbase_txid = blockchain::genesis_coinbase_txid();
        let genesis_output_script = genesis::GENESIS_OUTPUT_SCRIPT.to_string();
        let nbits = cpow::GENESIS_NBITS;

        let transactions = vec![
            Transaction::coinbase(genesis::GENESIS_MESSAGE.into()),
            Transaction::secret(genesis::GENESIS_SECRET.into()),
        ];
        let merkle_root = Self::compute_merkle_root(&transactions);

        let size = (genesis_output_script.len()
            + merkle_root.len()
            + coinbase_txid.len()
            + transactions.iter().map(|t| t.txid.len()).sum::<usize>()
            + 256) as u64;

        let mut b = Block {
            nversion: rules::BLOCK_VERSION,
            index: 0,
            timestamp: ts,
            mined_at: format_datetime(ts),
            mined_for: "GENESIS".to_string(),
            previous_hash: "0".repeat(64),
            merkle_root,
            hash: String::new(),
            nonce: 0,
            nbits,
            reward: crate::consensus::spec::INITIAL_REWARD,
            coinbase_txid,
            blockchain_size: size,
            genesis_output_script,
            miner_pubkey: String::new(),
            miner_signature: String::new(),
            transactions,
        };

        if !pow::mine(&mut b, nbits) {
            panic!("genesis mining gagal: nonce habis");
        }

        assert!(
            b.hash == genesis::EXPECTED_GENESIS_HASH,
            "\n\
             ============================================================\n\
             GENESIS HASH MISMATCH\n\
             ============================================================\n\
             hash genesis     : {}\n\
             EXPECTED_HASH    : {}\n\
             \n\
             Jalankan `./zenachain-genesis` untuk melihat nilai genesis\n\
             baru, lalu salin ke src/consensus/genesis.rs.\n\
             ============================================================\n",
            b.hash,
            genesis::EXPECTED_GENESIS_HASH
        );

        assert!(
            b.merkle_root == genesis::EXPECTED_GENESIS_MERKLE_ROOT,
            "\n\
             ============================================================\n\
             GENESIS MERKLE ROOT MISMATCH\n\
             ============================================================\n\
             merkle_root genesis : {}\n\
             EXPECTED_MERKLE     : {}\n\
             ============================================================\n",
            b.merkle_root,
            genesis::EXPECTED_GENESIS_MERKLE_ROOT
        );

        assert!(
            b.timestamp == genesis::EXPECTED_GENESIS_TIMESTAMP,
            "\n\
             ============================================================\n\
             GENESIS TIMESTAMP MISMATCH\n\
             ============================================================\n\
             timestamp genesis : {}\n\
             EXPECTED_TIMESTAMP: {}\n\
             ============================================================\n",
            b.timestamp,
            genesis::EXPECTED_GENESIS_TIMESTAMP
        );

        assert!(
            b.nonce == genesis::EXPECTED_GENESIS_NONCE,
            "\n\
             ============================================================\n\
             GENESIS NONCE MISMATCH\n\
             ============================================================\n\
             nonce genesis : {}\n\
             EXPECTED_NONCE: {}\n\
             ============================================================\n",
            b.nonce,
            genesis::EXPECTED_GENESIS_NONCE
        );

        b
    }

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
