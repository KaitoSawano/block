// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Consensus validation rules for ZenaChain.
//!
//! This module contains every validation rule used by the consensus layer.
//! Consensus constants are imported from `crate::consensus::rules`.

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::consensus::rules;
use crate::pow;
use crate::secp256k1::{address, verify};
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use std::collections::HashSet;
use std::fmt;

pub use rules::ADDRESS_HEX_LEN;
pub use rules::ADDRESS_PREFIX;
pub use rules::BLOCK_VERSION;
pub use rules::COINBASE_MATURITY;
pub use rules::MAX_INPUTS_PER_TX;
pub use rules::MAX_NOTE_LEN;
pub use rules::MAX_OUTPUTS_PER_TX;
pub use rules::MAX_TX_PER_BLOCK;
pub use rules::ZERO_HASH;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub code: &'static str,
    pub message: String,
}

impl ValidationError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        ValidationError {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

pub fn rule_address_format(addr: &str) -> Result<(), ValidationError> {
    if !addr.starts_with(ADDRESS_PREFIX) {
        return Err(ValidationError::new(
            "E001",
            format!("address does not start with {}: {}", ADDRESS_PREFIX, addr),
        ));
    }
    let hex_part = &addr[ADDRESS_PREFIX.len()..];
    if hex_part.len() != ADDRESS_HEX_LEN {
        return Err(ValidationError::new(
            "E002",
            format!(
                "address hex part must be {} characters, got {}",
                ADDRESS_HEX_LEN,
                hex_part.len()
            ),
        ));
    }
    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::new(
            "E003",
            format!("address contains a non-hexadecimal character: {}", addr),
        ));
    }
    Ok(())
}

pub fn rule_self_send(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.from == tx.to {
        return Err(ValidationError::new(
            "E004",
            "sender and recipient addresses are identical",
        ));
    }
    Ok(())
}

pub fn rule_amount_nonzero(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.amount == 0 {
        return Err(ValidationError::new(
            "E005",
            "amount must be greater than zero",
        ));
    }
    Ok(())
}

pub fn rule_note_length(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.note.len() > MAX_NOTE_LEN {
        return Err(ValidationError::new(
            "E006",
            format!("note exceeds {} bytes", MAX_NOTE_LEN),
        ));
    }
    Ok(())
}

pub fn rule_input_count(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.inputs.len() > MAX_INPUTS_PER_TX {
        return Err(ValidationError::new(
            "E007",
            format!("input count exceeds {}", MAX_INPUTS_PER_TX),
        ));
    }
    Ok(())
}

pub fn rule_signature(tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        return Ok(());
    }
    if tx.public_key.is_empty() {
        return Err(ValidationError::new("E010", "public key is empty"));
    }
    if tx.signature.is_empty() {
        return Err(ValidationError::new("E010", "signature is empty"));
    }
    let pk = hex::decode(&tx.public_key)
        .map_err(|_| ValidationError::new("E010", "public key is not hexadecimal"))?;
    let sig = hex::decode(&tx.signature)
        .map_err(|_| ValidationError::new("E010", "signature is not hexadecimal"))?;
    let msg = hex::decode(&tx.txid)
        .map_err(|_| ValidationError::new("E010", "txid is not hexadecimal"))?;
    if !verify::verify(&pk, &msg, &sig) {
        return Err(ValidationError::new(
            "E010",
            format!("signature is invalid for txid {}", tx.txid),
        ));
    }
    Ok(())
}

pub fn rule_address_matches_pubkey(tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        return Ok(());
    }
    if tx.public_key.is_empty() {
        return Err(ValidationError::new("E011", "public key is empty"));
    }
    let pk = hex::decode(&tx.public_key)
        .map_err(|_| ValidationError::new("E011", "public key is not hexadecimal"))?;
    let expected = address::address_from_public_key(&pk).map_err(|_| {
        ValidationError::new("E011", "public key cannot be converted to an address")
    })?;
    if expected != tx.from {
        return Err(ValidationError::new(
            "E011",
            format!(
                "address {} does not match the public key {}",
                tx.from, expected
            ),
        ));
    }
    Ok(())
}

pub fn rule_txid_consistent(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.txid != tx.compute_txid() {
        return Err(ValidationError::new(
            "E012",
            format!("txid {} does not match the computed value", tx.txid),
        ));
    }
    Ok(())
}

pub fn spent_inputs(chain: &Blockchain) -> HashSet<String> {
    let mut spent = HashSet::new();
    for block in &chain.chain {
        for tx in &block.transactions {
            for input in &tx.inputs {
                spent.insert(input.clone());
            }
        }
    }
    spent
}

pub fn mempool_inputs(chain: &Blockchain) -> HashSet<String> {
    let mut used = HashSet::new();
    for tx in &chain.mempool {
        for input in &tx.inputs {
            used.insert(input.clone());
        }
    }
    used
}

pub fn rule_no_double_spend(chain: &Blockchain, tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        return Ok(());
    }
    let spent_chain = spent_inputs(chain);
    let spent_mempool = mempool_inputs(chain);
    let mut conflicts = Vec::new();
    for input in &tx.inputs {
        if spent_chain.contains(input) || spent_mempool.contains(input) {
            conflicts.push(input.clone());
        }
    }
    if !conflicts.is_empty() {
        return Err(ValidationError::new(
            "E020",
            format!("inputs already spent: {}", conflicts.join(",")),
        ));
    }
    Ok(())
}

pub fn mature_balance(chain: &Blockchain, addr: &str) -> u64 {
    let spent = spent_inputs(chain);
    let current_height = chain.height();
    let mut mature: u64 = 0;
    for block in &chain.chain {
        for tx in &block.transactions {
            if tx.to == addr {
                let outpoint = format!("{}:0", tx.txid);
                if !spent.contains(&outpoint) {
                    mature = mature.saturating_add(tx.amount);
                }
            }
        }
        if block.mined_for == addr && block.reward > 0 {
            let mature_at = block.index.saturating_add(COINBASE_MATURITY);
            if mature_at <= current_height {
                mature = mature.saturating_add(block.reward);
            }
        }
    }
    mature
}

pub fn immature_balance(chain: &Blockchain, addr: &str) -> u64 {
    let current_height = chain.height();
    let mut immature: u64 = 0;
    for block in &chain.chain {
        if block.mined_for == addr && block.reward > 0 {
            let mature_at = block.index.saturating_add(COINBASE_MATURITY);
            if mature_at > current_height {
                immature = immature.saturating_add(block.reward);
            }
        }
    }
    immature
}

pub fn rule_sufficient_balance(
    chain: &Blockchain,
    tx: &Transaction,
) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        return Ok(());
    }
    let balance = mature_balance(chain, &tx.from);
    let need = tx.amount.saturating_add(tx.fee);
    if balance < need {
        return Err(ValidationError::new(
            "E021",
            format!(
                "insufficient balance: required {} sat, available {} sat",
                need, balance
            ),
        ));
    }
    Ok(())
}

pub fn rule_block_version(block: &Block) -> Result<(), ValidationError> {
    if block.nversion != BLOCK_VERSION {
        return Err(ValidationError::new(
            "E028",
            format!(
                "unsupported block version {} at index {}: expected {}",
                block.nversion, block.index, BLOCK_VERSION
            ),
        ));
    }
    Ok(())
}

pub fn rule_block_tx_count(block: &Block) -> Result<(), ValidationError> {
    if block.transactions.len() > MAX_TX_PER_BLOCK {
        return Err(ValidationError::new(
            "E030",
            format!(
                "block contains {} transactions, exceeding the limit of {}",
                block.transactions.len(),
                MAX_TX_PER_BLOCK
            ),
        ));
    }
    Ok(())
}

pub fn rule_block_reward(block: &Block) -> Result<(), ValidationError> {
    let expected = Blockchain::calculate_reward(block.index);
    if block.reward != expected {
        return Err(ValidationError::new(
            "E031",
            format!(
                "reward for block {} does not match the era: got {}, expected {}",
                block.index, block.reward, expected
            ),
        ));
    }
    Ok(())
}

pub fn rule_block_timestamp_monotonic(prev: &Block, cur: &Block) -> Result<(), ValidationError> {
    if cur.timestamp < prev.timestamp {
        return Err(ValidationError::new(
            "E032",
            format!(
                "timestamp of block {} is earlier than block {}",
                cur.index, prev.index
            ),
        ));
    }
    Ok(())
}

pub fn rule_block_nbits(chain: &Blockchain, block: &Block) -> Result<(), ValidationError> {
    if block.nbits != chain.nbits {
        return Err(ValidationError::new(
            "E033",
            format!(
                "nbits of block {:#010x} does not match chain nbits {:#010x}",
                block.nbits, chain.nbits
            ),
        ));
    }
    Ok(())
}

pub fn rule_block_hash_consistent(block: &Block) -> Result<(), ValidationError> {
    if block.hash != block.calculate_hash() {
        return Err(ValidationError::new(
            "E034",
            format!(
                "hash of block {} is not consistent with its contents",
                block.index
            ),
        ));
    }
    Ok(())
}

pub fn rule_block_previous_hash(prev: &Block, cur: &Block) -> Result<(), ValidationError> {
    if cur.previous_hash != prev.hash {
        return Err(ValidationError::new(
            "E035",
            format!(
                "previous_hash of block {} does not match the hash of block {}",
                cur.index, prev.index
            ),
        ));
    }
    Ok(())
}

pub fn rule_block_index_sequential(prev: &Block, cur: &Block) -> Result<(), ValidationError> {
    if cur.index != prev.index + 1 {
        return Err(ValidationError::new(
            "E036",
            format!(
                "index of block {} is not sequential after block {}",
                cur.index, prev.index
            ),
        ));
    }
    Ok(())
}

pub fn rule_block_pow(block: &Block) -> Result<(), ValidationError> {
    if !pow::hash_meets_nbits(&block.hash, block.nbits) {
        return Err(ValidationError::new(
            "E037",
            format!(
                "hash of block {} does not meet the target nbits {:#010x}",
                block.index, block.nbits
            ),
        ));
    }
    Ok(())
}

pub fn rule_block_checkpoint(block: &Block) -> Result<(), ValidationError> {
    if !crate::blockchain::checkpoint::verify(block.index, &block.hash) {
        return Err(ValidationError::new(
            "E038",
            format!("block {} violates a checkpoint", block.index),
        ));
    }
    Ok(())
}

pub fn rule_genesis(block: &Block) -> Result<(), ValidationError> {
    if block.index != 0 {
        return Err(ValidationError::new(
            "E039",
            "the genesis block must have index 0",
        ));
    }
    if block.previous_hash != ZERO_HASH {
        return Err(ValidationError::new(
            "E039",
            "the genesis block previous_hash must be zero",
        ));
    }
    Ok(())
}

pub fn rule_total_supply(chain: &Blockchain) -> Result<(), ValidationError> {
    let total = chain.circulating_supply();
    let max = crate::blockchain::max_supply();
    if total > max {
        return Err(ValidationError::new(
            "E041",
            format!(
                "total supply {} sat exceeds the maximum of {} sat",
                total, max
            ),
        ));
    }
    Ok(())
}

pub fn rule_genesis_fingerprint(block: &Block) -> Result<(), ValidationError> {
    let expected_txid = crate::blockchain::genesis_coinbase_txid();
    if block.coinbase_txid != expected_txid {
        return Err(ValidationError::new(
            "E042",
            format!(
                "genesis coinbase_txid does not match the consensus fingerprint: got {}, expected {}",
                block.coinbase_txid, expected_txid
            ),
        ));
    }
    Ok(())
}

pub fn rule_block_merkle_root(block: &Block) -> Result<(), ValidationError> {
    let expected = Block::compute_merkle_root(&block.transactions);
    if block.merkle_root != expected {
        return Err(ValidationError::new(
            "E043",
            format!(
                "merkle root of block {} does not match: got {}, expected {}",
                block.index, block.merkle_root, expected
            ),
        ));
    }
    Ok(())
}

pub fn rule_genesis_output_script(block: &Block) -> Result<(), ValidationError> {
    if block.index != 0 {
        return Ok(());
    }
    let expected = crate::blockchain::genesis_output_script();
    if block.genesis_output_script != expected {
        return Err(ValidationError::new(
            "E044",
            format!(
                "genesis output script does not match: got {}, expected {}",
                block.genesis_output_script, expected
            ),
        ));
    }
    Ok(())
}

pub fn rule_genesis_pow(block: &Block) -> Result<(), ValidationError> {
    if !pow::hash_meets_nbits(&block.hash, block.nbits) {
        return Err(ValidationError::new(
            "E045",
            format!(
                "genesis hash does not meet the target nbits {:#010x}",
                block.nbits
            ),
        ));
    }
    Ok(())
}

pub fn is_special_tx(tx: &Transaction) -> bool {
    tx.from == "COINBASE"
        || tx.from == "GENESIS"
        || tx.from == "NETWORK"
        || tx.from == "MEMPOOL"
}

pub fn is_valid_address(addr: &str) -> bool {
    rule_address_format(addr).is_ok()
}

pub fn validate_transaction(chain: &Blockchain, tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        return Ok(());
    }
    rule_txid_consistent(tx)?;
    rule_address_format(&tx.from)?;
    rule_address_format(&tx.to)?;
    rule_self_send(tx)?;
    rule_amount_nonzero(tx)?;
    rule_note_length(tx)?;
    rule_input_count(tx)?;
    rule_signature(tx)?;
    rule_address_matches_pubkey(tx)?;
    rule_no_double_spend(chain, tx)?;
    rule_sufficient_balance(chain, tx)?;
    Ok(())
}

pub fn validate_block_transactions(
    chain: &Blockchain,
    transactions: &[Transaction],
) -> Result<(), ValidationError> {
    for tx in transactions {
        validate_transaction(chain, tx)?;
    }
    Ok(())
}

pub fn validate_block(
    chain: &Blockchain,
    prev: &Block,
    block: &Block,
) -> Result<(), ValidationError> {
    rule_block_version(block)?;
    rule_block_index_sequential(prev, block)?;
    rule_block_hash_consistent(block)?;
    rule_block_previous_hash(prev, block)?;
    rule_block_merkle_root(block)?;
    rule_block_reward(block)?;
    rule_block_pow(block)?;
    rule_block_tx_count(block)?;
    rule_block_timestamp_monotonic(prev, block)?;
    rule_block_checkpoint(block)?;
    rule_block_nbits(chain, block)?;
    validate_block_transactions(chain, &block.transactions)?;
    Ok(())
}

pub fn validate_chain(chain: &Blockchain) -> Result<(), ValidationError> {
    if chain.chain.is_empty() {
        return Err(ValidationError::new("E040", "chain is empty"));
    }

    let genesis = &chain.chain[0];
    rule_genesis(genesis)?;
    rule_genesis_fingerprint(genesis)?;
    rule_genesis_output_script(genesis)?;
    rule_genesis_pow(genesis)?;
    rule_block_hash_consistent(genesis)?;
    rule_block_merkle_root(genesis)?;
    rule_block_version(genesis)?;
    rule_total_supply(chain)?;

    for i in 1..chain.chain.len() {
        let prev = &chain.chain[i - 1];
        let cur = &chain.chain[i];
        rule_block_version(cur)?;
        rule_block_index_sequential(prev, cur)?;
        rule_block_hash_consistent(cur)?;
        rule_block_previous_hash(prev, cur)?;
        rule_block_merkle_root(cur)?;
        rule_block_reward(cur)?;
        rule_block_pow(cur)?;
        rule_block_tx_count(cur)?;
        rule_block_timestamp_monotonic(prev, cur)?;
        rule_block_checkpoint(cur)?;
    }

    Ok(())
}

pub fn validate_mempool(chain: &Blockchain) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    for tx in &chain.mempool {
        if let Err(e) = validate_transaction(chain, tx) {
            errors.push(e);
        }
    }
    errors
}

pub fn wallet_has_balance(
    wallet: &Wallet,
    chain: &Blockchain,
    amount_sat: u64,
    fee_sat: u64,
) -> bool {
    let balance = mature_balance(chain, &wallet.address);
    balance >= amount_sat.saturating_add(fee_sat)
}
