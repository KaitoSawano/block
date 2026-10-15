use crate::utils::sha3_256_hex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub txid: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub note: String,
    pub inputs: Vec<String>,
}

impl Transaction {
    pub fn new(from: &str, to: &str, amount: u64, fee: u64) -> Self {
        Self::new_with_inputs(from, to, amount, fee, Vec::new(), String::new())
    }

    pub fn new_with_inputs(
        from: &str,
        to: &str,
        amount: u64,
        fee: u64,
        inputs: Vec<String>,
        note: String,
    ) -> Self {
        let mut tx = Transaction {
            txid: String::new(),
            from: from.into(),
            to: to.into(),
            amount,
            fee,
            note,
            inputs,
        };
        tx.txid = tx.compute_txid();
        tx
    }

    pub fn coinbase(note: String) -> Self {
        let mut tx = Transaction {
            txid: String::new(),
            from: "COINBASE".into(),
            to: "MINER".into(),
            amount: 0,
            fee: 0,
            note,
            inputs: Vec::new(),
        };
        tx.txid = tx.compute_txid();
        tx
    }

    pub fn secret(note: String) -> Self {
        let mut tx = Transaction {
            txid: String::new(),
            from: "GENESIS".into(),
            to: "ZENAC".into(),
            amount: 0,
            fee: 0,
            note,
            inputs: Vec::new(),
        };
        tx.txid = tx.compute_txid();
        tx
    }

    pub fn compute_txid(&self) -> String {
        let payload = format!(
            "{}|{}|{}|{}|{}|{}",
            self.from,
            self.to,
            self.amount,
            self.fee,
            self.note,
            self.inputs.join(",")
        );
        sha3_256_hex(payload.as_bytes())
    }

    pub fn is_valid(&self) -> bool {
        !self.to.is_empty() && !self.from.is_empty()
    }
}
