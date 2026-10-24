// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Consensus validation rules for ZenaChain.

use crate::block::Block; // The block structure to validate.
use crate::blockchain::Blockchain; // The chain state for contextual validation.
use crate::consensus::rules; // Consensus rule constants.
use crate::consensus::spec::{self, Network}; // Network parameters (Main/Test/Regtest).
use crate::pow; // Proof-of-work functions.
use crate::secp256k1::{address, verify}; // secp256k1 cryptography utilities.
use crate::transaction::Transaction; // The transaction structure.
use std::fmt; // For implementing Display on ValidationError.

// Re-export constants that are network-agnostic.
pub use rules::BLOCK_VERSION; // Block version (1).
pub use rules::COINBASE_MATURITY; // Coinbase maturity (100).
pub use rules::MAX_INPUTS_PER_TX; // Max inputs per tx.
pub use rules::MAX_NOTE_LEN; // Max note length.
pub use rules::MAX_OUTPUTS_PER_TX; // Max outputs per tx.
pub use rules::MAX_TX_PER_BLOCK; // Max tx per block.
pub use rules::ZERO_HASH; // Zero hash for genesis.

/// A validation error with a code and a human-readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub code: &'static str, // The error code (e.g., "E001").
    pub message: String,    // The human-readable error message.
}

impl ValidationError {
    // Constructs a new ValidationError.
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        ValidationError {
            code, // Store the error code.
            message: message.into(), // Convert the message into a String.
        }
    }
}

impl fmt::Display for ValidationError {
    // Implements Display so errors can be printed as "[code] message".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message) // Format the output.
    }
}

// Validates the address format (Legacy Base58Check or Native SegWit Bech32).
// If `ACTIVATION_HEIGHT_BECH32_ONLY` is active, only Bech32 is accepted.
pub fn rule_address_format(addr: &str, current_height: u64) -> Result<(), ValidationError> {
    let hrp = crate::blockchain::bech32_hrp(); // Get the Bech32 HRP for the active network.
    let is_bech32 = addr.starts_with(&format!("{}1", hrp)); // Check if it's Bech32.

    // Check if Bech32-only mode is active.
    let bech32_only = crate::blockchain::is_activation_active(
        crate::blockchain::ACTIVATION_HEIGHT_BECH32_ONLY,
        current_height,
    );

    if bech32_only && !is_bech32 {
        // If Bech32-only mode is active and the address is not Bech32, reject it.
        return Err(ValidationError::new(
            "E008", // Error code: Legacy address rejected after Bech32 activation.
            format!(
                "legacy addresses are no longer accepted after height {}: {}",
                crate::blockchain::ACTIVATION_HEIGHT_BECH32_ONLY,
                addr
            ),
        ));
    }

    if is_bech32 {
        // Validate Bech32 address.
        if !address::verify_bech32_address(addr) {
            return Err(ValidationError::new(
                "E001",
                format!("invalid bech32 address: {}", addr),
            ));
        }
        return Ok(());
    }

    // Validate Legacy address (if Bech32-only is not active).
    if !address::verify_legacy_address(addr) {
        return Err(ValidationError::new(
            "E001",
            format!("invalid legacy address: {}", addr),
        ));
    }
    Ok(())
}

// Validates that the sender and recipient are not the same.
pub fn rule_self_send(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.from == tx.to {
        return Err(ValidationError::new(
            "E004",
            "sender and recipient addresses are identical",
        ));
    }
    Ok(())
}

// Validates that the transfer amount is greater than zero.
pub fn rule_amount_nonzero(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.amount == 0 {
        return Err(ValidationError::new(
            "E005",
            "amount must be greater than zero",
        ));
    }
    Ok(())
}

// Validates the length of the transaction note.
pub fn rule_note_length(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.note.len() > MAX_NOTE_LEN {
        return Err(ValidationError::new(
            "E006",
            format!("note exceeds {} bytes", MAX_NOTE_LEN),
        ));
    }
    Ok(())
}

// Validates the transaction signature.
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
    let msg = hex::decode(tx.signing_hash())
        .map_err(|_| ValidationError::new("E010", "signing hash is not hexadecimal"))?;
    if !verify::verify(&pk, &msg, &sig) {
        return Err(ValidationError::new(
            "E010",
            format!("signature is invalid for txid {}", tx.txid),
        ));
    }
    Ok(())
}

// Validates that the public key matches the sender address.
// Supports both Legacy (Base58Check) and Bech32 addresses.
pub fn rule_address_matches_pubkey(tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        return Ok(());
    }
    if tx.public_key.is_empty() {
        return Err(ValidationError::new("E011", "public key is empty"));
    }
    let pk = hex::decode(&tx.public_key)
        .map_err(|_| ValidationError::new("E011", "public key is not hexadecimal"))?;

    // Detect the address format based on the prefix.
    let hrp = crate::blockchain::bech32_hrp();
    let expected = if tx.from.starts_with(&format!("{}1", hrp)) {
        address::address_from_public_key_bech32(&pk).map_err(|_| {
            ValidationError::new("E011", "public key cannot be converted to a bech32 address")
        })?
    } else {
        address::address_from_public_key(&pk).map_err(|_| {
            ValidationError::new("E011", "public key cannot be converted to a legacy address")
        })?
    };

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

// Validates that the txid is consistent with the transaction contents.
pub fn rule_txid_consistent(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.txid != tx.compute_txid() {
        return Err(ValidationError::new(
            "E012",
            format!("txid {} does not match the computed value", tx.txid),
        ));
    }
    Ok(())
}

// Validates the transaction nonce against the chain state and mempool.
pub fn rule_nonce(chain: &Blockchain, tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        return Ok(());
    }

    let mut expected = chain.nonce_of(&tx.from);

    loop {
        let has_mempool_tx = chain
            .mempool
            .iter()
            .any(|t| t.from == tx.from && t.nonce == expected);

        if !has_mempool_tx {
            break;
        }
        expected += 1;
    }

    if tx.nonce != expected {
        return Err(ValidationError::new(
            "E022",
            format!("invalid nonce: got {}, expected {}", tx.nonce, expected),
        ));
    }
    Ok(())
}

// Validates that the sender has enough balance for the amount plus fee.
pub fn rule_sufficient_balance(
    chain: &Blockchain,
    tx: &Transaction,
) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        return Ok(());
    }
    let balance = chain.spendable_balance_of(&tx.from);
    let need = tx.amount.saturating_add(tx.fee);
    if balance < need {
        return Err(ValidationError::new(
            "E021",
            format!(
                "insufficient spendable balance: required {} sat, available {} sat",
                need, balance
            ),
        ));
    }
    Ok(())
}

// Validates the block version.
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

// Validates the number of transactions in the block.
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

// Validates the block subsidy against the halving era.
pub fn rule_block_subsidy(block: &Block) -> Result<(), ValidationError> {
    let expected = Blockchain::calculate_reward(block.index);
    if block.subsidy != expected {
        return Err(ValidationError::new(
            "E031",
            format!(
                "subsidy for block {} does not match the era: got {}, expected {}",
                block.index, block.subsidy, expected
            ),
        ));
    }
    Ok(())
}

// Validates the total block reward (subsidy + total_fees).
pub fn rule_block_reward(block: &Block) -> Result<(), ValidationError> {
    let expected = block.subsidy.saturating_add(block.total_fees);
    if block.reward != expected {
        return Err(ValidationError::new(
            "E046",
            format!(
                "reward for block {} is invalid: got {}, expected {} (subsidy {} + fees {})",
                block.index, block.reward, expected, block.subsidy, block.total_fees
            ),
        ));
    }
    Ok(())
}

// Validates the total_fees of the block against the sum of transaction fees.
pub fn rule_block_total_fees(block: &Block) -> Result<(), ValidationError> {
    let computed: u64 = block.transactions.iter().map(|tx| tx.fee).sum();
    if block.total_fees != computed {
        return Err(ValidationError::new(
            "E047",
            format!(
                "total_fees for block {} is invalid: got {}, expected {}",
                block.index, block.total_fees, computed
            ),
        ));
    }
    Ok(())
}

// Validates the consistency of the block's coinbase_txid.
pub fn rule_block_coinbase_consistent(block: &Block) -> Result<(), ValidationError> {
    if block.index == 0 {
        return Ok(());
    }
    let expected = zenachain_crypto::sha3_256_hex(
        format!("{}|{}|{}", block.index, block.mined_for, block.timestamp).as_bytes(),
    );
    if block.coinbase_txid != expected {
        return Err(ValidationError::new(
            "E048",
            format!(
                "coinbase_txid for block {} is inconsistent with index, mined_for, and timestamp",
                block.index
            ),
        ));
    }
    Ok(())
}

// Validates the mined_for address.
pub fn rule_mined_for_valid(block: &Block) -> Result<(), ValidationError> {
    if block.index == 0 {
        return Ok(());
    }
    if block.mined_for.is_empty() {
        return Err(ValidationError::new(
            "E049",
            format!("block {} has an empty mined_for address", block.index),
        ));
    }
    rule_address_format(&block.mined_for, block.index).map_err(|_| {
        ValidationError::new(
            "E049",
            format!(
                "mined_for address of block {} is invalid: {}",
                block.index, block.mined_for
            ),
        )
    })
}

// Validates that the block timestamp is monotonically increasing.
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

// Validates the block nbits against the chain's nbits.
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

// Validates that the block hash is consistent with its contents.
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

// Validates that the block previous_hash matches the hash of the previous block.
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

// Validates that the block index is sequential.
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

// Validates the block's proof-of-work.
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

// Validates the block against known checkpoints.
pub fn rule_block_checkpoint(block: &Block) -> Result<(), ValidationError> {
    if !crate::blockchain::checkpoint::verify(block.index, &block.hash) {
        return Err(ValidationError::new(
            "E038",
            format!("block {} violates a checkpoint", block.index),
        ));
    }
    Ok(())
}

// Validates the structure of the genesis block.
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

// Validates that the total supply does not exceed the maximum.
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

// Validates the genesis fingerprint for a specific network.
pub fn rule_genesis_fingerprint_for(
    block: &Block,
    network: Network,
) -> Result<(), ValidationError> {
    let expected_txid = crate::blockchain::genesis_coinbase_txid_for(network);
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

// Validates the genesis fingerprint for the active network.
pub fn rule_genesis_fingerprint(block: &Block) -> Result<(), ValidationError> {
    rule_genesis_fingerprint_for(block, spec::current_network())
}

// Validates the block's merkle root.
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

// Validates the genesis output script.
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

// Validates the genesis block's proof-of-work.
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

// Checks whether a transaction is a special transaction (not a regular user transaction).
pub fn is_special_tx(tx: &Transaction) -> bool {
    tx.from == "COINBASE"
        || tx.from == "GENESIS"
        || tx.from == "NETWORK"
        || tx.from == "MEMPOOL"
}

// Checks whether an address is valid for the active network.
pub fn is_valid_address(addr: &str, current_height: u64) -> bool {
    rule_address_format(addr, current_height).is_ok()
}

// Fully validates a single transaction.
pub fn validate_transaction(chain: &Blockchain, tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        return Ok(());
    }
    rule_txid_consistent(tx)?;
    rule_address_format(&tx.from, chain.height())?;
    rule_address_format(&tx.to, chain.height())?;
    rule_self_send(tx)?;
    rule_amount_nonzero(tx)?;
    rule_note_length(tx)?;
    rule_signature(tx)?;
    rule_address_matches_pubkey(tx)?;
    rule_nonce(chain, tx)?;
    rule_sufficient_balance(chain, tx)?;
    Ok(())
}

// Validates all transactions in a block (with an overlay state).
pub fn validate_block_transactions(
    chain: &Blockchain,
    transactions: &[Transaction],
    height: u64,
) -> Result<(), ValidationError> {
    use std::collections::HashMap;

    let mut overlay_balances: HashMap<String, u64> = HashMap::new();
    let mut overlay_nonces: HashMap<String, u64> = HashMap::new();

    for tx in transactions {
        if is_special_tx(tx) {
            continue;
        }

        let base_nonce = overlay_nonces
            .get(&tx.from)
            .copied()
            .unwrap_or_else(|| chain.nonce_of(&tx.from));
        if tx.nonce != base_nonce {
            return Err(ValidationError::new(
                "E022",
                format!(
                    "invalid nonce for {}: got {}, expected {}",
                    tx.from, tx.nonce, base_nonce
                ),
            ));
        }

        let base_balance = overlay_balances
            .get(&tx.from)
            .copied()
            .unwrap_or_else(|| chain.spendable_balance_of(&tx.from));
        let need = match tx.amount.checked_add(tx.fee) {
            Some(v) => v,
            None => {
                return Err(ValidationError::new("E023", "amount + fee overflows u64"));
            }
        };
        if base_balance < need {
            return Err(ValidationError::new(
                "E021",
                format!(
                    "insufficient spendable balance for {}: required {} sat, available {} sat",
                    tx.from, need, base_balance
                ),
            ));
        }

        let new_from = base_balance - need;
        overlay_balances.insert(tx.from.clone(), new_from);

        let to_balance = overlay_balances
            .get(&tx.to)
            .copied()
            .unwrap_or_else(|| chain.balance_of(&tx.to));
        let new_to = match to_balance.checked_add(tx.amount) {
            Some(v) => v,
            None => {
                return Err(ValidationError::new(
                    "E024",
                    "recipient balance overflows u64",
                ));
            }
        };
        overlay_balances.insert(tx.to.clone(), new_to);

        overlay_nonces.insert(tx.from.clone(), base_nonce + 1);
    }

    Ok(())
}

// Fully validates a single block.
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
    rule_block_subsidy(block)?;
    rule_block_total_fees(block)?;
    rule_block_reward(block)?;
    rule_block_coinbase_consistent(block)?;
    rule_mined_for_valid(block)?;
    rule_block_pow(block)?;
    rule_block_tx_count(block)?;
    rule_block_timestamp_monotonic(prev, block)?;
    rule_block_checkpoint(block)?;
    rule_block_nbits(chain, block)?;
    validate_block_transactions(chain, &block.transactions, block.index)?;
    Ok(())
}

// Validates the entire chain from genesis to the tip.
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

    if !genesis.transactions.is_empty() {
        validate_block_transactions(chain, &genesis.transactions, genesis.index)?;
    }

    for i in 1..chain.chain.len() {
        let prev = &chain.chain[i - 1];
        let cur = &chain.chain[i];
        rule_block_version(cur)?;
        rule_block_index_sequential(prev, cur)?;
        rule_block_hash_consistent(cur)?;
        rule_block_previous_hash(prev, cur)?;
        rule_block_merkle_root(cur)?;
        rule_block_subsidy(cur)?;
        rule_block_total_fees(cur)?;
        rule_block_reward(cur)?;
        rule_block_coinbase_consistent(cur)?;
        rule_mined_for_valid(cur)?;
        rule_block_pow(cur)?;
        rule_block_tx_count(cur)?;
        rule_block_timestamp_monotonic(prev, cur)?;
        rule_block_checkpoint(cur)?;
        rule_block_nbits(chain, cur)?;
        validate_block_transactions(chain, &cur.transactions, cur.index)?;
    }

    Ok(())
}

// Validates all transactions currently in the mempool.
pub fn validate_mempool(chain: &Blockchain) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    for tx in &chain.mempool {
        if let Err(e) = validate_transaction(chain, tx) {
            errors.push(e);
        }
    }
    errors
}
