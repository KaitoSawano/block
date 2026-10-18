// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Transaction structure and signature operations.

use zenachain_crypto::sha3_256_hex;
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
    pub output_script: String,
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
            output_script: String::new(),
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
            output_script: String::new(),
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
            output_script: String::new(),
        };
        tx.txid = tx.compute_txid();
        tx
    }

    pub fn compute_txid(&self) -> String {
        let payload = format!(
            "{}|{}|{}|{}|{}|{}|{}",
            self.from,
            self.to,
            self.amount,
            self.fee,
            self.note,
            self.inputs.join(","),
            self.output_script
        );
        sha3_256_hex(payload.as_bytes())
    }

    pub fn sign(&mut self, secret_hex: &str) -> Result<(), String> {
        use crate::secp256k1::{keypair::KeyPair, sign};

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
        crate::validation::rule_signature(self).is_ok()
    }

    pub fn expected_address(&self) -> Option<String> {
        if self.public_key.is_empty() {
            return None;
        }
        let pk = hex::decode(&self.public_key).ok()?;
        crate::secp256k1::address::address_from_public_key(&pk).ok()
    }

    pub fn is_valid(&self) -> bool {
        crate::validation::rule_txid_consistent(self).is_ok()
            && crate::validation::rule_address_format(&self.from).is_ok()
            && crate::validation::rule_address_format(&self.to).is_ok()
            && crate::validation::rule_self_send(self).is_ok()
            && crate::validation::rule_amount_nonzero(self).is_ok()
            && crate::validation::rule_note_length(self).is_ok()
            && crate::validation::rule_input_count(self).is_ok()
            && crate::validation::rule_signature(self).is_ok()
            && crate::validation::rule_address_matches_pubkey(self).is_ok()
    }
}
