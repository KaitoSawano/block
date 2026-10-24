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
pub fn rule_address_format(addr: &str) -> Result<(), ValidationError> {
    let hrp = crate::blockchain::bech32_hrp(); // Get the Bech32 HRP for the active network.
    if addr.starts_with(&format!("{}1", hrp)) {
        // If the address starts with the Bech32 HRP followed by "1"...
        if !address::verify_bech32_address(addr) {
            // Validate the Bech32 address.
            return Err(ValidationError::new(
                "E001", // Error code: invalid Bech32 address.
                format!("invalid bech32 address: {}", addr),
            ));
        }
        return Ok(()); // The Bech32 address is valid.
    }

    // Otherwise, treat it as a Legacy (Base58Check) address.
    if !address::verify_legacy_address(addr) {
        // Validate the Legacy address.
        return Err(ValidationError::new(
            "E001", // Error code: invalid Legacy address.
            format!("invalid legacy address: {}", addr),
        ));
    }
    Ok(()) // The Legacy address is valid.
}

// Validates that the sender and recipient are not the same.
pub fn rule_self_send(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.from == tx.to {
        // If the sender equals the recipient, reject it.
        return Err(ValidationError::new(
            "E004", // Error code: self-send.
            "sender and recipient addresses are identical",
        ));
    }
    Ok(()) // Not a self-send.
}

// Validates that the transfer amount is greater than zero.
pub fn rule_amount_nonzero(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.amount == 0 {
        // If the amount is zero, reject it.
        return Err(ValidationError::new(
            "E005", // Error code: zero amount.
            "amount must be greater than zero",
        ));
    }
    Ok(()) // The amount is valid.
}

// Validates the length of the transaction note.
pub fn rule_note_length(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.note.len() > MAX_NOTE_LEN {
        // If the note exceeds the limit, reject it.
        return Err(ValidationError::new(
            "E006", // Error code: note too long.
            format!("note exceeds {} bytes", MAX_NOTE_LEN),
        ));
    }
    Ok(()) // The note is valid.
}

// Validates the transaction signature.
pub fn rule_signature(tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        // Special transactions (COINBASE, GENESIS, etc.) do not need a signature.
        return Ok(());
    }
    if tx.public_key.is_empty() {
        // An empty public key is invalid.
        return Err(ValidationError::new("E010", "public key is empty"));
    }
    if tx.signature.is_empty() {
        // An empty signature is invalid.
        return Err(ValidationError::new("E010", "signature is empty"));
    }
    let pk = hex::decode(&tx.public_key)
        // Decode the public key from hex; error out if it fails.
        .map_err(|_| ValidationError::new("E010", "public key is not hexadecimal"))?;
    let sig = hex::decode(&tx.signature)
        // Decode the signature from hex; error out if it fails.
        .map_err(|_| ValidationError::new("E010", "signature is not hexadecimal"))?;
    let msg = hex::decode(tx.signing_hash())
        // Decode the signing hash from hex; error out if it fails.
        .map_err(|_| ValidationError::new("E010", "signing hash is not hexadecimal"))?;
    if !verify::verify(&pk, &msg, &sig) {
        // Verify the ECDSA signature; reject if invalid.
        return Err(ValidationError::new(
            "E010", // Error code: invalid signature.
            format!("signature is invalid for txid {}", tx.txid),
        ));
    }
    Ok(()) // The signature is valid.
}

// Validates that the public key matches the sender address.
// Supports both Legacy (Base58Check) and Bech32 addresses.
pub fn rule_address_matches_pubkey(tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        // Special transactions do not need this check.
        return Ok(());
    }
    if tx.public_key.is_empty() {
        // An empty public key is invalid.
        return Err(ValidationError::new("E011", "public key is empty"));
    }
    let pk = hex::decode(&tx.public_key)
        // Decode the public key; error out if it fails.
        .map_err(|_| ValidationError::new("E011", "public key is not hexadecimal"))?;

    // Detect the address format based on the prefix.
    let hrp = crate::blockchain::bech32_hrp();
    let expected = if tx.from.starts_with(&format!("{}1", hrp)) {
        // Bech32 address.
        address::address_from_public_key_bech32(&pk).map_err(|_| {
            ValidationError::new("E011", "public key cannot be converted to a bech32 address")
        })?
    } else {
        // Legacy address.
        address::address_from_public_key(&pk).map_err(|_| {
            ValidationError::new("E011", "public key cannot be converted to a legacy address")
        })?
    };

    if expected != tx.from {
        // If the derived address does not match the sender, reject it.
        return Err(ValidationError::new(
            "E011", // Error code: address does not match pubkey.
            format!(
                "address {} does not match the public key {}",
                tx.from, expected
            ),
        ));
    }
    Ok(()) // The address matches the public key.
}

// Validates that the txid is consistent with the transaction contents.
pub fn rule_txid_consistent(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.txid != tx.compute_txid() {
        // If the txid does not match the recomputed hash, reject it.
        return Err(ValidationError::new(
            "E012", // Error code: inconsistent txid.
            format!("txid {} does not match the computed value", tx.txid),
        ));
    }
    Ok(()) // The txid is consistent.
}

// Validates the transaction nonce against the chain state and mempool.
pub fn rule_nonce(chain: &Blockchain, tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        // Special transactions do not need a nonce check.
        return Ok(());
    }

    let mut expected = chain.nonce_of(&tx.from); // The current nonce from the state.

    loop {
        // Check if there is a mempool transaction from the same sender with this nonce.
        let has_mempool_tx = chain
            .mempool
            .iter()
            .any(|t| t.from == tx.from && t.nonce == expected);

        if !has_mempool_tx {
            // If there is none in the mempool, this is the expected nonce.
            break;
        }
        expected += 1; // If there is one, increment the expected nonce.
    }

    if tx.nonce != expected {
        // If the transaction nonce does not match the expected nonce, reject it.
        return Err(ValidationError::new(
            "E022", // Error code: wrong nonce.
            format!("invalid nonce: got {}, expected {}", tx.nonce, expected),
        ));
    }
    Ok(()) // The nonce is valid.
}

// Validates that the sender has enough balance for the amount plus fee.
pub fn rule_sufficient_balance(
    chain: &Blockchain,
    tx: &Transaction,
) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        // Special transactions do not need a balance check.
        return Ok(());
    }
    // Use spendable_balance, which already accounts for Coinbase Maturity.
    let balance = chain.spendable_balance_of(&tx.from);
    let need = tx.amount.saturating_add(tx.fee); // The total amount required.
    if balance < need {
        // If the balance is insufficient, reject it.
        return Err(ValidationError::new(
            "E021", // Error code: insufficient balance.
            format!(
                "insufficient spendable balance: required {} sat, available {} sat",
                need, balance
            ),
        ));
    }
    Ok(()) // The balance is sufficient.
}

// Validates the block version.
pub fn rule_block_version(block: &Block) -> Result<(), ValidationError> {
    if block.nversion != BLOCK_VERSION {
        // If the block version is not supported, reject it.
        return Err(ValidationError::new(
            "E028", // Error code: wrong block version.
            format!(
                "unsupported block version {} at index {}: expected {}",
                block.nversion, block.index, BLOCK_VERSION
            ),
        ));
    }
    Ok(()) // The block version is valid.
}

// Validates the number of transactions in the block.
pub fn rule_block_tx_count(block: &Block) -> Result<(), ValidationError> {
    if block.transactions.len() > MAX_TX_PER_BLOCK {
        // If the number of transactions exceeds the limit, reject it.
        return Err(ValidationError::new(
            "E030", // Error code: too many transactions.
            format!(
                "block contains {} transactions, exceeding the limit of {}",
                block.transactions.len(),
                MAX_TX_PER_BLOCK
            ),
        ));
    }
    Ok(()) // The transaction count is valid.
}

// Validates the block subsidy against the halving era.
pub fn rule_block_subsidy(block: &Block) -> Result<(), ValidationError> {
    let expected = Blockchain::calculate_reward(block.index); // The expected reward.
    if block.subsidy != expected {
        // If the subsidy does not match, reject it.
        return Err(ValidationError::new(
            "E031", // Error code: wrong subsidy.
            format!(
                "subsidy for block {} does not match the era: got {}, expected {}",
                block.index, block.subsidy, expected
            ),
        ));
    }
    Ok(()) // The subsidy is valid.
}

// Validates the total block reward (subsidy + total_fees).
pub fn rule_block_reward(block: &Block) -> Result<(), ValidationError> {
    let expected = block.subsidy.saturating_add(block.total_fees); // The expected reward.
    if block.reward != expected {
        // If the reward does not match, reject it.
        return Err(ValidationError::new(
            "E046", // Error code: wrong reward.
            format!(
                "reward for block {} is invalid: got {}, expected {} (subsidy {} + fees {})",
                block.index, block.reward, expected, block.subsidy, block.total_fees
            ),
        ));
    }
    Ok(()) // The reward is valid.
}

// Validates the total_fees of the block against the sum of transaction fees.
pub fn rule_block_total_fees(block: &Block) -> Result<(), ValidationError> {
    let computed: u64 = block.transactions.iter().map(|tx| tx.fee).sum(); // Compute total fees.
    if block.total_fees != computed {
        // If they do not match, reject it.
        return Err(ValidationError::new(
            "E047", // Error code: wrong total_fees.
            format!(
                "total_fees for block {} is invalid: got {}, expected {}",
                block.index, block.total_fees, computed
            ),
        ));
    }
    Ok(()) // The total fees are valid.
}

// Validates the consistency of the block's coinbase_txid.
pub fn rule_block_coinbase_consistent(block: &Block) -> Result<(), ValidationError> {
    if block.index == 0 {
        // Genesis is exempt.
        return Ok(());
    }
    let expected = zenachain_crypto::sha3_256_hex(
        // Recompute the coinbase_txid from index, mined_for, and timestamp.
        format!("{}|{}|{}", block.index, block.mined_for, block.timestamp).as_bytes(),
    );
    if block.coinbase_txid != expected {
        // If it does not match, reject it.
        return Err(ValidationError::new(
            "E048", // Error code: inconsistent coinbase.
            format!(
                "coinbase_txid for block {} is inconsistent with index, mined_for, and timestamp",
                block.index
            ),
        ));
    }
    Ok(()) // The coinbase is consistent.
}

// Validates the mined_for address.
pub fn rule_mined_for_valid(block: &Block) -> Result<(), ValidationError> {
    if block.index == 0 {
        // Genesis is exempt.
        return Ok(());
    }
    if block.mined_for.is_empty() {
        // The mined_for address must not be empty.
        return Err(ValidationError::new(
            "E049", // Error code: empty mined_for.
            format!("block {} has an empty mined_for address", block.index),
        ));
    }
    rule_address_format(&block.mined_for).map_err(|_| {
        // Validate the format of the mined_for address.
        ValidationError::new(
            "E049", // Error code: invalid mined_for.
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
        // If the current block timestamp is earlier than the previous, reject it.
        return Err(ValidationError::new(
            "E032", // Error code: non-monotonic timestamp.
            format!(
                "timestamp of block {} is earlier than block {}",
                cur.index, prev.index
            ),
        ));
    }
    Ok(()) // The timestamp is valid.
}

// Validates the block nbits against the chain's nbits.
pub fn rule_block_nbits(chain: &Blockchain, block: &Block) -> Result<(), ValidationError> {
    if block.nbits != chain.nbits {
        // If the nbits does not match, reject it.
        return Err(ValidationError::new(
            "E033", // Error code: wrong nbits.
            format!(
                "nbits of block {:#010x} does not match chain nbits {:#010x}",
                block.nbits, chain.nbits
            ),
        ));
    }
    Ok(()) // The nbits is valid.
}

// Validates that the block hash is consistent with its contents.
pub fn rule_block_hash_consistent(block: &Block) -> Result<(), ValidationError> {
    if block.hash != block.calculate_hash() {
        // If the hash does not match, reject it.
        return Err(ValidationError::new(
            "E034", // Error code: inconsistent hash.
            format!(
                "hash of block {} is not consistent with its contents",
                block.index
            ),
        ));
    }
    Ok(()) // The hash is valid.
}

// Validates that the block previous_hash matches the hash of the previous block.
pub fn rule_block_previous_hash(prev: &Block, cur: &Block) -> Result<(), ValidationError> {
    if cur.previous_hash != prev.hash {
        // If the previous_hash does not match, reject it.
        return Err(ValidationError::new(
            "E035", // Error code: wrong previous_hash.
            format!(
                "previous_hash of block {} does not match the hash of block {}",
                cur.index, prev.index
            ),
        ));
    }
    Ok(()) // The previous hash is valid.
}

// Validates that the block index is sequential.
pub fn rule_block_index_sequential(prev: &Block, cur: &Block) -> Result<(), ValidationError> {
    if cur.index != prev.index + 1 {
        // If the index is not sequential, reject it.
        return Err(ValidationError::new(
            "E036", // Error code: non-sequential index.
            format!(
                "index of block {} is not sequential after block {}",
                cur.index, prev.index
            ),
        ));
    }
    Ok(()) // The index is valid.
}

// Validates the block's proof-of-work.
pub fn rule_block_pow(block: &Block) -> Result<(), ValidationError> {
    if !pow::hash_meets_nbits(&block.hash, block.nbits) {
        // If the hash does not meet the target, reject it.
        return Err(ValidationError::new(
            "E037", // Error code: invalid PoW.
            format!(
                "hash of block {} does not meet the target nbits {:#010x}",
                block.index, block.nbits
            ),
        ));
    }
    Ok(()) // The PoW is valid.
}

// Validates the block against known checkpoints.
pub fn rule_block_checkpoint(block: &Block) -> Result<(), ValidationError> {
    if !crate::blockchain::checkpoint::verify(block.index, &block.hash) {
        // If the block violates a checkpoint, reject it.
        return Err(ValidationError::new(
            "E038", // Error code: checkpoint violation.
            format!("block {} violates a checkpoint", block.index),
        ));
    }
    Ok(()) // The checkpoint is valid.
}

// Validates the structure of the genesis block.
pub fn rule_genesis(block: &Block) -> Result<(), ValidationError> {
    if block.index != 0 {
        // Genesis must have index 0.
        return Err(ValidationError::new(
            "E039", // Error code: wrong genesis index.
            "the genesis block must have index 0",
        ));
    }
    if block.previous_hash != ZERO_HASH {
        // Genesis must have a zero previous_hash.
        return Err(ValidationError::new(
            "E039", // Error code: wrong genesis previous_hash.
            "the genesis block previous_hash must be zero",
        ));
    }
    Ok(()) // The genesis block is valid.
}

// Validates that the total supply does not exceed the maximum.
pub fn rule_total_supply(chain: &Blockchain) -> Result<(), ValidationError> {
    let total = chain.circulating_supply(); // The current total supply.
    let max = crate::blockchain::max_supply(); // The maximum allowed supply.
    if total > max {
        // If it exceeds the maximum, reject it.
        return Err(ValidationError::new(
            "E041", // Error code: supply overflow.
            format!(
                "total supply {} sat exceeds the maximum of {} sat",
                total, max
            ),
        ));
    }
    Ok(()) // The supply is valid.
}

// Validates the genesis fingerprint for a specific network.
pub fn rule_genesis_fingerprint_for(
    block: &Block,
    network: Network,
) -> Result<(), ValidationError> {
    let expected_txid = crate::blockchain::genesis_coinbase_txid_for(network); // The expected genesis txid.
    if block.coinbase_txid != expected_txid {
        // If it does not match, reject it.
        return Err(ValidationError::new(
            "E042", // Error code: wrong genesis fingerprint.
            format!(
                "genesis coinbase_txid does not match the consensus fingerprint: got {}, expected {}",
                block.coinbase_txid, expected_txid
            ),
        ));
    }
    Ok(()) // The fingerprint is valid.
}

// Validates the genesis fingerprint for the active network.
pub fn rule_genesis_fingerprint(block: &Block) -> Result<(), ValidationError> {
    rule_genesis_fingerprint_for(block, spec::current_network()) // Use the active network.
}

// Validates the block's merkle root.
pub fn rule_block_merkle_root(block: &Block) -> Result<(), ValidationError> {
    let expected = Block::compute_merkle_root(&block.transactions); // The expected merkle root.
    if block.merkle_root != expected {
        // If it does not match, reject it.
        return Err(ValidationError::new(
            "E043", // Error code: wrong merkle root.
            format!(
                "merkle root of block {} does not match: got {}, expected {}",
                block.index, block.merkle_root, expected
            ),
        ));
    }
    Ok(()) // The merkle root is valid.
}

// Validates the genesis output script.
pub fn rule_genesis_output_script(block: &Block) -> Result<(), ValidationError> {
    if block.index != 0 {
        // Only applies to genesis.
        return Ok(());
    }
    let expected = crate::blockchain::genesis_output_script(); // The expected output script.
    if block.genesis_output_script != expected {
        // If it does not match, reject it.
        return Err(ValidationError::new(
            "E044", // Error code: wrong genesis output script.
            format!(
                "genesis output script does not match: got {}, expected {}",
                block.genesis_output_script, expected
            ),
        ));
    }
    Ok(()) // The output script is valid.
}

// Validates the genesis block's proof-of-work.
pub fn rule_genesis_pow(block: &Block) -> Result<(), ValidationError> {
    if !pow::hash_meets_nbits(&block.hash, block.nbits) {
        // If the genesis PoW is invalid, reject it.
        return Err(ValidationError::new(
            "E045", // Error code: invalid genesis PoW.
            format!(
                "genesis hash does not meet the target nbits {:#010x}",
                block.nbits
            ),
        ));
    }
    Ok(()) // The genesis PoW is valid.
}

// Checks whether a transaction is a special transaction (not a regular user transaction).
pub fn is_special_tx(tx: &Transaction) -> bool {
    tx.from == "COINBASE" // Coinbase transaction.
        || tx.from == "GENESIS" // Genesis transaction.
        || tx.from == "NETWORK" // Internal network transaction.
        || tx.from == "MEMPOOL" // Internal mempool transaction.
}

// Checks whether an address is valid for the active network.
pub fn is_valid_address(addr: &str) -> bool {
    rule_address_format(addr).is_ok() // Validate the address format.
}

// Fully validates a single transaction.
pub fn validate_transaction(chain: &Blockchain, tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        // Skip special transactions.
        return Ok(());
    }
    rule_txid_consistent(tx)?; // Check the txid.
    rule_address_format(&tx.from)?; // Check the sender address.
    rule_address_format(&tx.to)?; // Check the recipient address.
    rule_self_send(tx)?; // Check for self-send.
    rule_amount_nonzero(tx)?; // Check the amount.
    rule_note_length(tx)?; // Check the note length.
    rule_signature(tx)?; // Check the signature.
    rule_address_matches_pubkey(tx)?; // Check address/pubkey match.
    rule_nonce(chain, tx)?; // Check the nonce.
    rule_sufficient_balance(chain, tx)?; // Check the balance.
    Ok(()) // The transaction is valid.
}

// Validates all transactions in a block (with an overlay state).
pub fn validate_block_transactions(
    chain: &Blockchain,
    transactions: &[Transaction],
) -> Result<(), ValidationError> {
    use std::collections::HashMap; // For the balance/nonce overlay.

    let mut overlay_balances: HashMap<String, u64> = HashMap::new(); // Temporary balances.
    let mut overlay_nonces: HashMap<String, u64> = HashMap::new(); // Temporary nonces.

    for tx in transactions {
        if is_special_tx(tx) {
            // Skip special transactions.
            continue;
        }

        let base_nonce = overlay_nonces
            .get(&tx.from)
            .copied()
            .unwrap_or_else(|| chain.nonce_of(&tx.from)); // Base nonce (overlay or state).
        if tx.nonce != base_nonce {
            // If the nonce does not match, reject it.
            return Err(ValidationError::new(
                "E022", // Error code: wrong nonce.
                format!(
                    "invalid nonce for {}: got {}, expected {}",
                    tx.from, tx.nonce, base_nonce
                ),
            ));
        }

        // Use spendable_balance, which accounts for Coinbase Maturity.
        let base_balance = overlay_balances
            .get(&tx.from)
            .copied()
            .unwrap_or_else(|| chain.spendable_balance_of(&tx.from)); // Base balance.
        let need = match tx.amount.checked_add(tx.fee) {
            // Compute the total required.
            Some(v) => v, // If no overflow, use it.
            None => {
                // If overflow, reject it.
                return Err(ValidationError::new("E023", "amount + fee overflows u64"));
            }
        };
        if base_balance < need {
            // If the balance is insufficient, reject it.
            return Err(ValidationError::new(
                "E021", // Error code: insufficient balance.
                format!(
                    "insufficient spendable balance for {}: required {} sat, available {} sat",
                    tx.from, need, base_balance
                ),
            ));
        }

        let new_from = base_balance - need; // The new sender balance.
        overlay_balances.insert(tx.from.clone(), new_from); // Store it in the overlay.

        let to_balance = overlay_balances
            .get(&tx.to)
            .copied()
            .unwrap_or_else(|| chain.balance_of(&tx.to)); // The recipient balance.
        let new_to = match to_balance.checked_add(tx.amount) {
            // Add the amount to the recipient.
            Some(v) => v, // If no overflow, use it.
            None => {
                // If overflow, reject it.
                return Err(ValidationError::new(
                    "E024", // Error code: recipient balance overflow.
                    "recipient balance overflows u64",
                ));
            }
        };
        overlay_balances.insert(tx.to.clone(), new_to); // Store it in the overlay.

        overlay_nonces.insert(tx.from.clone(), base_nonce + 1); // Increment the sender nonce.
    }

    Ok(()) // All transactions are valid.
}

// Fully validates a single block.
pub fn validate_block(
    chain: &Blockchain,
    prev: &Block,
    block: &Block,
) -> Result<(), ValidationError> {
    rule_block_version(block)?; // Check the version.
    rule_block_index_sequential(prev, block)?; // Check the index.
    rule_block_hash_consistent(block)?; // Check the hash.
    rule_block_previous_hash(prev, block)?; // Check the previous_hash.
    rule_block_merkle_root(block)?; // Check the merkle root.
    rule_block_subsidy(block)?; // Check the subsidy.
    rule_block_total_fees(block)?; // Check the total fees.
    rule_block_reward(block)?; // Check the reward.
    rule_block_coinbase_consistent(block)?; // Check the coinbase.
    rule_mined_for_valid(block)?; // Check the mined_for address.
    rule_block_pow(block)?; // Check the PoW.
    rule_block_tx_count(block)?; // Check the transaction count.
    rule_block_timestamp_monotonic(prev, block)?; // Check the timestamp.
    rule_block_checkpoint(block)?; // Check the checkpoint.
    rule_block_nbits(chain, block)?; // Check the nbits.
    validate_block_transactions(chain, &block.transactions)?; // Check all transactions.
    Ok(()) // The block is valid.
}

// Validates the entire chain from genesis to the tip.
pub fn validate_chain(chain: &Blockchain) -> Result<(), ValidationError> {
    if chain.chain.is_empty() {
        // The chain must not be empty.
        return Err(ValidationError::new("E040", "chain is empty"));
    }

    let genesis = &chain.chain[0]; // The genesis block.
    rule_genesis(genesis)?; // Check the genesis structure.
    rule_genesis_fingerprint(genesis)?; // Check the genesis fingerprint.
    rule_genesis_output_script(genesis)?; // Check the genesis output script.
    rule_genesis_pow(genesis)?; // Check the genesis PoW.
    rule_block_hash_consistent(genesis)?; // Check the genesis hash.
    rule_block_merkle_root(genesis)?; // Check the genesis merkle root.
    rule_block_version(genesis)?; // Check the genesis version.
    rule_total_supply(chain)?; // Check the total supply.

    if !genesis.transactions.is_empty() {
        // Validate the genesis transactions (if any).
        validate_block_transactions(chain, &genesis.transactions)?;
    }

    for i in 1..chain.chain.len() {
        // Iterate over all blocks after genesis.
        let prev = &chain.chain[i - 1]; // The previous block.
        let cur = &chain.chain[i]; // The current block.
        rule_block_version(cur)?; // Check the version.
        rule_block_index_sequential(prev, cur)?; // Check the index.
        rule_block_hash_consistent(cur)?; // Check the hash.
        rule_block_previous_hash(prev, cur)?; // Check the previous_hash.
        rule_block_merkle_root(cur)?; // Check the merkle root.
        rule_block_subsidy(cur)?; // Check the subsidy.
        rule_block_total_fees(cur)?; // Check the total fees.
        rule_block_reward(cur)?; // Check the reward.
        rule_block_coinbase_consistent(cur)?; // Check the coinbase.
        rule_mined_for_valid(cur)?; // Check the mined_for address.
        rule_block_pow(cur)?; // Check the PoW.
        rule_block_tx_count(cur)?; // Check the transaction count.
        rule_block_timestamp_monotonic(prev, cur)?; // Check the timestamp.
        rule_block_checkpoint(cur)?; // Check the checkpoint.
        rule_block_nbits(chain, cur)?; // Check the nbits.
        validate_block_transactions(chain, &cur.transactions)?; // Check all transactions.
    }

    Ok(()) // The entire chain is valid.
}

// Validates all transactions currently in the mempool.
pub fn validate_mempool(chain: &Blockchain) -> Vec<ValidationError> {
    let mut errors = Vec::new(); // Collect all errors.
    for tx in &chain.mempool {
        // Iterate over each mempool transaction.
        if let Err(e) = validate_transaction(chain, tx) {
            // Validate the transaction.
            errors.push(e); // If invalid, collect the error.
        }
    }
    errors // Return the list of errors.
}
