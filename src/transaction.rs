// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Account-based transaction model with per-address nonces.
//!
//! The transaction identifier (`txid`) commits to every field except
//! the signature, following the Bitcoin convention. The signature
//! commits to a `signing_hash` that includes the sender nonce, so that
//! a transaction cannot be replayed or reordered without invalidating
//! the signature.

use serde::{Deserialize, Serialize};
use zenachain_crypto::sha3_256_hex;

use crate::consensus::spec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub txid: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub note: String,
    pub public_key: String,
    pub signature: String,
    pub output_script: String,
}

impl Transaction {
    pub fn new(from: &str, to: &str, amount: u64, fee: u64, nonce: u64) -> Self {
        Self::new_with_note(from, to, amount, fee, nonce, String::new())
    }

    pub fn new_with_note(
        from: &str,
        to: &str,
        amount: u64,
        fee: u64,
        nonce: u64,
        note: String,
    ) -> Self {
        let mut tx = Transaction {
            txid: String::new(),
            from: from.into(),
            to: to.into(),
            amount,
            fee,
            nonce,
            note,
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
            nonce: 0,
            note,
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
            nonce: 0,
            note,
            public_key: String::new(),
            signature: String::new(),
            output_script: String::new(),
        };
        tx.txid = tx.compute_txid();
        tx
    }

    /// Compute the transaction identifier.
    ///
    /// The identifier commits to every field except the signature.
    pub fn compute_txid(&self) -> String {
        let payload = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            self.from,
            self.to,
            self.amount,
            self.fee,
            self.nonce,
            self.note,
            self.public_key,
            self.output_script
        );
        sha3_256_hex(payload.as_bytes())
    }

    /// Compute the signing hash.
    ///
    /// The signing hash includes the nonce so that the signature binds
    /// the transaction to a specific position in the sender's sequence.
    /// The signature field is excluded because a signature cannot commit
    /// to itself.
    ///
    /// IMPORTANT: The signing hash now includes the network's magic bytes
    /// (acting as a Chain ID) to prevent cross-network replay attacks.
    /// A transaction signed for Mainnet will be invalid on Testnet/Regtest.
    pub fn signing_hash(&self) -> String {
        let chain_id = spec::current_network().magic_hex();
        let payload = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}",
            chain_id, // Inject Chain ID here
            self.from,
            self.to,
            self.amount,
            self.fee,
            self.nonce,
            self.note,
            self.public_key,
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

        let msg = hex::decode(self.signing_hash()).map_err(|e| e.to_string())?;
        let sig = sign::sign(&kp.secret_key, &msg)?;
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
            && crate::validation::rule_signature(self).is_ok()
            && crate::validation::rule_address_matches_pubkey(self).is_ok()
    }
}
