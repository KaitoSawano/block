//! Modul validasi konsensus ZenaChain.
//!
//! Semua aturan konsensus berada di file ini. File lain hanya
//! memanggil fungsi di sini, tidak boleh berisi logika validasi.

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::pow;
use crate::secp256k1::{address, verify};
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use std::collections::HashSet;
use std::fmt;

// ============================================================
// KONSTANTA KONSENSUS
// ============================================================

pub const COINBASE_MATURITY: u64 = 100;
pub const ADDRESS_PREFIX: &str = "ZENA";
pub const ADDRESS_HEX_LEN: usize = 40;
pub const MAX_TX_PER_BLOCK: usize = 10_000;
pub const MAX_INPUTS_PER_TX: usize = 1_000;
pub const MAX_OUTPUTS_PER_TX: usize = 1_000;
pub const MAX_NOTE_LEN: usize = 256;
pub const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

// ============================================================
// STRUKTUR ERROR
// ============================================================

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

// ============================================================
// ATURAN FORMAT
// ============================================================

/// E001: alamat harus diawali "ZENA".
/// E002: bagian setelah prefix harus 40 karakter.
/// E003: bagian setelah prefix harus hex.
pub fn rule_address_format(addr: &str) -> Result<(), ValidationError> {
    if !addr.starts_with(ADDRESS_PREFIX) {
        return Err(ValidationError::new(
            "E001",
            format!("alamat tidak diawali {}: {}", ADDRESS_PREFIX, addr),
        ));
    }

    let hex_part = &addr[ADDRESS_PREFIX.len()..];

    if hex_part.len() != ADDRESS_HEX_LEN {
        return Err(ValidationError::new(
            "E002",
            format!(
                "panjang hex alamat harus {}, dapat {}",
                ADDRESS_HEX_LEN,
                hex_part.len()
            ),
        ));
    }

    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::new(
            "E003",
            format!("alamat mengandung karakter bukan hex: {}", addr),
        ));
    }

    Ok(())
}

/// E004: transaksi tidak boleh mengirim ke alamat sendiri.
pub fn rule_self_send(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.from == tx.to {
        return Err(ValidationError::new(
            "E004",
            "alamat pengirim dan tujuan sama",
        ));
    }
    Ok(())
}

/// E005: amount harus lebih dari nol.
pub fn rule_amount_nonzero(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.amount == 0 {
        return Err(ValidationError::new(
            "E005",
            "amount harus lebih dari nol",
        ));
    }
    Ok(())
}

/// E006: note tidak boleh melebihi MAX_NOTE_LEN byte.
pub fn rule_note_length(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.note.len() > MAX_NOTE_LEN {
        return Err(ValidationError::new(
            "E006",
            format!("note melebihi {} byte", MAX_NOTE_LEN),
        ));
    }
    Ok(())
}

/// E007: jumlah input tidak boleh melebihi MAX_INPUTS_PER_TX.
pub fn rule_input_count(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.inputs.len() > MAX_INPUTS_PER_TX {
        return Err(ValidationError::new(
            "E007",
            format!("input melebihi {}", MAX_INPUTS_PER_TX),
        ));
    }
    Ok(())
}

// ============================================================
// ATURAN SIGNATURE
// ============================================================

/// E010: signature harus valid.
pub fn rule_signature(tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        return Ok(());
    }

    if tx.public_key.is_empty() {
        return Err(ValidationError::new("E010", "public key kosong"));
    }
    if tx.signature.is_empty() {
        return Err(ValidationError::new("E010", "signature kosong"));
    }

    let pk = match hex::decode(&tx.public_key) {
        Ok(b) => b,
        Err(_) => {
            return Err(ValidationError::new(
                "E010",
                "public key bukan hex",
            ))
        }
    };
    let sig = match hex::decode(&tx.signature) {
        Ok(b) => b,
        Err(_) => {
            return Err(ValidationError::new(
                "E010",
                "signature bukan hex",
            ))
        }
    };
    let msg = match hex::decode(&tx.txid) {
        Ok(b) => b,
        Err(_) => {
            return Err(ValidationError::new(
                "E010",
                "txid bukan hex",
            ))
        }
    };

    if !verify::verify(&pk, &msg, &sig) {
        return Err(ValidationError::new(
            "E010",
            format!("signature tidak valid untuk txid {}", tx.txid),
        ));
    }

    Ok(())
}

/// E011: alamat harus cocok dengan public key.
pub fn rule_address_matches_pubkey(tx: &Transaction) -> Result<(), ValidationError> {
    if is_special_tx(tx) {
        return Ok(());
    }

    if tx.public_key.is_empty() {
        return Err(ValidationError::new(
            "E011",
            "public key kosong",
        ));
    }

    let pk = match hex::decode(&tx.public_key) {
        Ok(b) => b,
        Err(_) => {
            return Err(ValidationError::new(
                "E011",
                "public key bukan hex",
            ))
        }
    };

    let expected = match address::address_from_public_key(&pk) {
        Ok(a) => a,
        Err(_) => {
            return Err(ValidationError::new(
                "E011",
                "public key tidak dapat diturunkan ke alamat",
            ))
        }
    };

    if expected != tx.from {
        return Err(ValidationError::new(
            "E011",
            format!("alamat {} tidak cocok dengan pubkey {}", tx.from, expected),
        ));
    }

    Ok(())
}

/// E012: txid harus konsisten dengan isi transaksi.
pub fn rule_txid_consistent(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.txid != tx.compute_txid() {
        return Err(ValidationError::new(
            "E012",
            format!("txid {} tidak cocok dengan hitungan", tx.txid),
        ));
    }
    Ok(())
}

// ============================================================
// ATURAN UTXO DAN SALDO
// ============================================================

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

/// E020: input tidak boleh sudah dipakai di chain atau mempool.
pub fn rule_no_double_spend(
    chain: &Blockchain,
    tx: &Transaction,
) -> Result<(), ValidationError> {
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
            format!("input sudah dipakai: {}", conflicts.join(",")),
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

/// E021: saldo pengirim harus cukup untuk amount + fee.
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
                "saldo tidak cukup: butuh {} sat, tersedia {} sat",
                need, balance
            ),
        ));
    }
    Ok(())
}

// ============================================================
// ATURAN BLOK
// ============================================================

/// E030: blok tidak boleh punya lebih dari MAX_TX_PER_BLOCK transaksi.
pub fn rule_block_tx_count(block: &Block) -> Result<(), ValidationError> {
    if block.transactions.len() > MAX_TX_PER_BLOCK {
        return Err(ValidationError::new(
            "E030",
            format!(
                "blok berisi {} transaksi, melebihi batas {}",
                block.transactions.len(),
                MAX_TX_PER_BLOCK
            ),
        ));
    }
    Ok(())
}

/// E031: coinbase reward harus sesuai era halving.
pub fn rule_block_reward(block: &Block) -> Result<(), ValidationError> {
    let expected = Blockchain::calculate_reward(block.index);
    if block.reward != expected {
        return Err(ValidationError::new(
            "E031",
            format!(
                "reward blok {} tidak sesuai era: dapat {}, seharusnya {}",
                block.index, block.reward, expected
            ),
        ));
    }
    Ok(())
}

/// E032: timestamp blok tidak boleh mundur dari blok sebelumnya.
pub fn rule_block_timestamp_monotonic(
    prev: &Block,
    cur: &Block,
) -> Result<(), ValidationError> {
    if cur.timestamp < prev.timestamp {
        return Err(ValidationError::new(
            "E032",
            format!(
                "timestamp blok {} mundur dari blok {}",
                cur.index, prev.index
            ),
        ));
    }
    Ok(())
}

/// E033: nBits blok harus sama dengan nBits chain saat ini.
pub fn rule_block_nbits(chain: &Blockchain, block: &Block) -> Result<(), ValidationError> {
    if block.nbits != chain.nbits {
        return Err(ValidationError::new(
            "E033",
            format!(
                "nbits blok {:#010x} tidak sesuai chain {:#010x}",
                block.nbits, chain.nbits
            ),
        ));
    }
    Ok(())
}

/// E034: hash blok harus konsisten dengan isi.
pub fn rule_block_hash_consistent(block: &Block) -> Result<(), ValidationError> {
    if block.hash != block.calculate_hash() {
        return Err(ValidationError::new(
            "E034",
            format!("hash blok {} tidak konsisten dengan isi", block.index),
        ));
    }
    Ok(())
}

/// E035: previous_hash blok harus cocok dengan hash blok sebelumnya.
pub fn rule_block_previous_hash(
    prev: &Block,
    cur: &Block,
) -> Result<(), ValidationError> {
    if cur.previous_hash != prev.hash {
        return Err(ValidationError::new(
            "E035",
            format!(
                "previous_hash blok {} tidak cocok dengan hash blok {}",
                cur.index, prev.index
            ),
        ));
    }
    Ok(())
}

/// E036: index blok harus prev.index + 1.
pub fn rule_block_index_sequential(
    prev: &Block,
    cur: &Block,
) -> Result<(), ValidationError> {
    if cur.index != prev.index + 1 {
        return Err(ValidationError::new(
            "E036",
            format!(
                "index blok {} tidak berurutan setelah {}",
                cur.index, prev.index
            ),
        ));
    }
    Ok(())
}

/// E037: PoW blok harus memenuhi target nbits.
pub fn rule_block_pow(block: &Block) -> Result<(), ValidationError> {
    if !pow::hash_meets_nbits(&block.hash, block.nbits) {
        return Err(ValidationError::new(
            "E037",
            format!(
                "hash blok {} tidak memenuhi target nbits {:#010x}",
                block.index, block.nbits
            ),
        ));
    }
    Ok(())
}

/// E038: blok tidak boleh melanggar checkpoint.
pub fn rule_block_checkpoint(block: &Block) -> Result<(), ValidationError> {
    if !crate::blockchain::checkpoint::verify(block.index, &block.hash) {
        return Err(ValidationError::new(
            "E038",
            format!("blok {} melanggar checkpoint", block.index),
        ));
    }
    Ok(())
}

/// E039: genesis block harus index 0 dan previous_hash nol.
pub fn rule_genesis(block: &Block) -> Result<(), ValidationError> {
    if block.index != 0 {
        return Err(ValidationError::new(
            "E039",
            "genesis block harus index 0",
        ));
    }
    if block.previous_hash != ZERO_HASH {
        return Err(ValidationError::new(
            "E039",
            "genesis previous_hash harus nol",
        ));
    }
    Ok(())
}

// ============================================================
// PUNGGUNG VALIDASI
// ============================================================

pub fn is_special_tx(tx: &Transaction) -> bool {
    tx.from == "COINBASE"
        || tx.from == "GENESIS"
        || tx.from == "NETWORK"
        || tx.from == "MEMPOOL"
}

pub fn is_valid_address(addr: &str) -> bool {
    rule_address_format(addr).is_ok()
}

/// Validasi lengkap satu transaksi biasa.
pub fn validate_transaction(
    chain: &Blockchain,
    tx: &Transaction,
) -> Result<(), ValidationError> {
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

/// Validasi semua transaksi di dalam blok.
pub fn validate_block_transactions(
    chain: &Blockchain,
    transactions: &[Transaction],
) -> Result<(), ValidationError> {
    for tx in transactions {
        validate_transaction(chain, tx)?;
    }
    Ok(())
}

/// Validasi satu blok penuh terhadap blok sebelumnya.
pub fn validate_block(
    chain: &Blockchain,
    prev: &Block,
    block: &Block,
) -> Result<(), ValidationError> {
    rule_block_index_sequential(prev, block)?;
    rule_block_hash_consistent(block)?;
    rule_block_previous_hash(prev, block)?;
    rule_block_reward(block)?;
    rule_block_pow(block)?;
    rule_block_tx_count(block)?;
    rule_block_timestamp_monotonic(prev, block)?;
    rule_block_checkpoint(block)?;
    rule_block_nbits(chain, block)?;
    validate_block_transactions(chain, &block.transactions)?;
    Ok(())
}

/// Validasi seluruh chain.
pub fn validate_chain(chain: &Blockchain) -> Result<(), ValidationError> {
    if chain.chain.is_empty() {
        return Err(ValidationError::new("E040", "chain kosong"));
    }

    rule_genesis(&chain.chain[0])?;

    for i in 1..chain.chain.len() {
        let prev = &chain.chain[i - 1];
        let cur = &chain.chain[i];

        // Untuk validasi chain, chain yang dipakai adalah chain yang
        // sedang diperiksa minus blok setelah cur.
        rule_block_index_sequential(prev, cur)?;
        rule_block_hash_consistent(cur)?;
        rule_block_previous_hash(prev, cur)?;
        rule_block_reward(cur)?;
        rule_block_pow(cur)?;
        rule_block_tx_count(cur)?;
        rule_block_timestamp_monotonic(prev, cur)?;
        rule_block_checkpoint(cur)?;
    }

    Ok(())
}

/// Validasi seluruh isi mempool.
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
