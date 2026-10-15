use crate::secp256k1::{address, keypair::KeyPair, sign, verify};
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
    pub public_key: String,
    pub signature: String,
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
            public_key: String::new(),
            signature: String::new(),
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
            public_key: String::new(),
            signature: String::new(),
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
            public_key: String::new(),
            signature: String::new(),
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

    pub fn sign(&mut self, secret_hex: &str) -> Result<(), String> {
        let sk = hex::decode(secret_hex).map_err(|e| e.to_string())?;
        let kp = KeyPair::from_secret(&sk)?;

        self.public_key = kp.public_key_hex();
        self.txid = self.compute_txid();

        let txid_bytes = hex::decode(&self.txid).map_err(|e| e.to_string())?;
        let sig = sign::sign(&kp.secret_key, &txid_bytes)?;
        self.signature = hex::encode(sig);

        Ok(())
    }

    pub fn verify_signature(&self) -> bool {
        if self.from == "COINBASE"
            || self.from == "GENESIS"
            || self.from == "NETWORK"
            || self.from == "MEMPOOL"
        {
            return true;
        }

        if self.public_key.is_empty() || self.signature.is_empty() {
            return false;
        }

        let pk = match hex::decode(&self.public_key) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let sig = match hex::decode(&self.signature) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let msg = match hex::decode(&self.txid) {
            Ok(b) => b,
            Err(_) => return false,
        };

        verify::verify(&pk, &msg, &sig)
    }

    pub fn expected_address(&self) -> Option<String> {
        if self.public_key.is_empty() {
            return None;
        }
        let pk = hex::decode(&self.public_key).ok()?;
        address::address_from_public_key(&pk).ok()
    }

    pub fn is_valid(&self) -> bool {
        if self.to.is_empty() || self.from.is_empty() {
            return false;
        }
        if !self.verify_signature() {
            return false;
        }
        if let Some(expected) = self.expected_address() {
            if expected != self.from {
                return false;
            }
        }
        true
    }
}
