// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! UTXO-based transaction model, modeled after Bitcoin Core.
//!
//! A transaction consists of a list of inputs (`TxIn`) and a list of
//! outputs (`TxOut`). Each input references a previous output via an
//! `OutPoint` (txid and output index) and carries a `script_sig` that
//! satisfies the `script_pubkey` of the referenced output. The
//! signature commits to a `SignatureHash` (SIGHASH_ALL) computed from a
//! binary serialization of the transaction, not from a string format.

use serde::{Deserialize, Serialize};
use zenachain_crypto::sha3_256_hex; // Keep SHA3 for txid? Bitcoin uses double SHA256.
// For strict Bitcoin compatibility, replace with double SHA256.
// This example keeps SHA3 for the existing hash infrastructure but the
// sighash uses double SHA256 below.

/// SIGHASH_ALL: the default Bitcoin signature hash type.
/// The signature commits to all inputs and all outputs.
pub const SIGHASH_ALL: u32 = 0x00000001;

/// A reference to a previous transaction output.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutPoint {
    /// The transaction identifier of the previous transaction.
    pub txid: String,
    /// The index of the output in the previous transaction.
    pub vout: u32,
}

/// A transaction input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxIn {
    /// The previous output being spent.
    pub prevout: OutPoint,
    /// The script that satisfies the previous output's script_pubkey.
    pub script_sig: Vec<u8>,
    /// The sequence number. Set to 0xffffffff for final.
    pub sequence: u32,
}

/// A transaction output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOut {
    /// The amount in satoshis.
    pub value: u64,
    /// The script that locks this output.
    pub script_pubkey: Vec<u8>,
}

/// A UTXO-based transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction version.
    pub version: i32,
    /// List of inputs.
    pub vin: Vec<TxIn>,
    /// List of outputs.
    pub vout: Vec<TxOut>,
    /// Lock time.
    pub lock_time: u32,
    /// The transaction identifier. Computed as SHA3-256(serialized
    /// transaction without script_sig). In Bitcoin this is
    /// double-SHA256.
    pub txid: String,
}

impl Transaction {
    /// Create a new empty transaction.
    pub fn new() -> Self {
        Transaction {
            version: 1,
            vin: Vec::new(),
            vout: Vec::new(),
            lock_time: 0,
            txid: String::new(),
        }
    }

    /// Compute the transaction identifier.
    /// The identifier commits to the version, all inputs (with empty
    /// script_sig), all outputs, and the lock time. It does not commit
    /// to the script_sig, following Bitcoin's legacy txid definition.
    pub fn compute_txid(&self) -> String {
        let mut clone = self.clone();
        for input in &mut clone.vin {
            input.script_sig.clear();
        }
        let serialized = clone.serialize_for_txid();
        sha3_256_hex(&serialized)
    }

    /// Serialize the transaction for txid computation.
    /// This is a simplified binary serialization. For strict Bitcoin
    /// compatibility, use the exact Bitcoin serialization format.
    fn serialize_for_txid(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.version.to_le_bytes());
        // varint(vin.len())
        out.push(self.vin.len() as u8);
        for input in &self.vin {
            let txid_bytes = hex::decode(&input.prevout.txid).unwrap_or_else(|_| vec![0u8; 32]);
            out.extend_from_slice(&txid_bytes);
            out.extend_from_slice(&input.prevout.vout.to_le_bytes());
            // varint(script_sig.len()) — empty here
            out.push(0);
            out.extend_from_slice(&input.sequence.to_le_bytes());
        }
        // varint(vout.len())
        out.push(self.vout.len() as u8);
        for output in &self.vout {
            out.extend_from_slice(&output.value.to_le_bytes());
            out.push(output.script_pubkey.len() as u8);
            out.extend_from_slice(&output.script_pubkey);
        }
        out.extend_from_slice(&self.lock_time.to_le_bytes());
        out
    }

    /// Compute the SignatureHash (SIGHASH_ALL) for a given input index.
    ///
    /// The algorithm follows Bitcoin's legacy sighash:
    /// 1. Clone the transaction.
    /// 2. For the input being signed, set script_sig to the
    ///    script_pubkey of the previous output (scriptCode).
    /// 3. For all other inputs, clear script_sig.
    /// 4. Serialize the modified transaction.
    /// 5. Append the sighash type (4 bytes little-endian).
    /// 6. Return double-SHA256 of the result.
    pub fn signature_hash(&self, input_index: usize, prevout_script: &[u8]) -> Result<[u8; 32], String> {
        if input_index >= self.vin.len() {
            return Err("input index out of range".into());
        }

        let mut modified = self.clone();
        // Clear all script_sig fields.
        for input in &mut modified.vin {
            input.script_sig.clear();
        }
        // Set the scriptCode for the input being signed.
        modified.vin[input_index].script_sig = prevout_script.to_vec();

        let mut serialized = modified.serialize_for_sighash();
        // Append SIGHASH_ALL as a 4-byte little-endian integer.
        serialized.extend_from_slice(&SIGHASH_ALL.to_le_bytes());

        // Double SHA256, following Bitcoin.
        use sha2::{Digest, Sha256};
        let first = Sha256::digest(&serialized);
        let second = Sha256::digest(&first);
        let mut result = [0u8; 32];
        result.copy_from_slice(&second);
        Ok(result)
    }

    /// Serialize the transaction for sighash computation.
    fn serialize_for_sighash(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.version.to_le_bytes());
        out.push(self.vin.len() as u8);
        for input in &self.vin {
            let txid_bytes = hex::decode(&input.prevout.txid).unwrap_or_else(|_| vec![0u8; 32]);
            out.extend_from_slice(&txid_bytes);
            out.extend_from_slice(&input.prevout.vout.to_le_bytes());
            out.push(input.script_sig.len() as u8);
            out.extend_from_slice(&input.script_sig);
            out.extend_from_slice(&input.sequence.to_le_bytes());
        }
        out.push(self.vout.len() as u8);
        for output in &self.vout {
            out.extend_from_slice(&output.value.to_le_bytes());
            out.push(output.script_pubkey.len() as u8);
            out.extend_from_slice(&output.script_pubkey);
        }
        out.extend_from_slice(&self.lock_time.to_le_bytes());
        out
    }

    /// Sign a specific input with the given secret key and the
    /// script_pubkey of the previous output.
    pub fn sign_input(
        &mut self,
        input_index: usize,
        secret_key: &[u8],
        prevout_script: &[u8],
    ) -> Result<(), String> {
        use crate::secp256k1::{keypair::KeyPair, sign};

        let sighash = self.signature_hash(input_index, prevout_script)?;
        let kp = KeyPair::from_secret(secret_key)?;
        let sig = sign::sign(&kp.secret_key, &sighash)?;

        // In Bitcoin, the script_sig for P2PKH is <signature> <pubkey>.
        // For simplicity, we store the signature and the public key in
        // a simple concatenated format: 1-byte sig_len | sig | pubkey.
        let mut script_sig = Vec::new();
        script_sig.push(sig.len() as u8);
        script_sig.extend_from_slice(&sig);
        script_sig.extend_from_slice(&kp.public_key);
        self.vin[input_index].script_sig = script_sig;

        self.txid = self.compute_txid();
        Ok(())
    }

    /// Verify the signature of a specific input.
    pub fn verify_input(&self, input_index: usize, prevout_script: &[u8]) -> bool {
        use crate::secp256k1::verify;

        if input_index >= self.vin.len() {
            return false;
        }
        let script_sig = &self.vin[input_index].script_sig;
        if script_sig.is_empty() {
            return false;
        }
        let sig_len = script_sig[0] as usize;
        if script_sig.len() < 1 + sig_len + 33 {
            return false;
        }
        let sig = &script_sig[1..1 + sig_len];
        let pubkey = &script_sig[1 + sig_len..1 + sig_len + 33];

        let sighash = match self.signature_hash(input_index, prevout_script) {
            Ok(h) => h,
            Err(_) => return false,
        };
        verify::verify(pubkey, &sighash, sig)
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new()
    }
}
