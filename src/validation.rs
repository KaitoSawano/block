//! Modul validasi konsensus ZenaChain.
//!
//! Semua aturan konsensus berada di file ini. Setiap aturan
//! ditulis sebagai fungsi terpisah dengan kode error sendiri,
//! supaya mudah diaudit dan diuji.
//!
//! Konvensi:
//! - Setiap fungsi validasi mengembalikan `Result<(), ValidationError>`.
//! - Setiap aturan punya kode error `E###` untuk identifikasi.
//! - Fungsi `validate_*` adalah penggabung dari aturan-aturan kecil.
//!
//! Bagian yang belum diimplementasikan ditandai dengan
//! `// TODO(rule): ...` supaya bisa dicari dengan mudah.

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use std::collections::HashSet;
use std::fmt;

// ============================================================
// KONSTANTA KONSENSUS
// ============================================================

/// Kedalaman minimum konfirmasi sebelum reward blok bisa dibelanjakan.
pub const COINBASE_MATURITY: u64 = 100;

/// Prefix alamat ZENA.
pub const ADDRESS_PREFIX: &str = "ZENA";

/// Panjang bagian hex alamat ZENA.
pub const ADDRESS_HEX_LEN: usize = 40;

/// Jumlah maksimum transaksi dalam satu blok.
pub const MAX_TX_PER_BLOCK: usize = 10_000;

/// Jumlah maksimum input dalam satu transaksi.
pub const MAX_INPUTS_PER_TX: usize = 1_000;

/// Jumlah maksimum output dalam satu transaksi (belum dipakai).
pub const MAX_OUTPUTS_PER_TX: usize = 1_000;

/// Jumlah maksimum catatan (note) dalam satu transaksi, dalam byte.
pub const MAX_NOTE_LEN: usize = 256;

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

/// E001: Alamat harus diawali "ZENA".
/// E002: Bagian setelah prefix harus 40 karakter.
/// E003: Bagian setelah prefix harus hex.
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

/// E004: Transaksi tidak boleh mengirim ke alamat sendiri.
pub fn rule_self_send(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.from == tx.to {
        return Err(ValidationError::new(
            "E004",
            "alamat pengirim dan tujuan sama",
        ));
    }
    Ok(())
}

/// E005: Amount harus lebih dari nol.
pub fn rule_amount_nonzero(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.amount == 0 {
        return Err(ValidationError::new(
            "E005",
            "amount harus lebih dari nol",
        ));
    }
    Ok(())
}

/// E006: Note tidak boleh melebihi MAX_NOTE_LEN byte.
pub fn rule_note_length(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.note.len() > MAX_NOTE_LEN {
        return Err(ValidationError::new(
            "E006",
            format!("note melebihi {} byte", MAX_NOTE_LEN),
        ));
    }
    Ok(())
}

/// E007: Jumlah input tidak boleh melebihi MAX_INPUTS_PER_TX.
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

/// E010: Signature harus valid.
pub fn rule_signature(tx: &Transaction) -> Result<(), ValidationError> {
    if !tx.verify_signature() {
        return Err(ValidationError::new(
            "E010",
            format!("signature tidak valid untuk txid {}", tx.txid),
        ));
    }
    Ok(())
}

/// E011: Alamat harus cocok dengan public key.
pub fn rule_address_matches_pubkey(tx: &Transaction) -> Result<(), ValidationError> {
    match tx.expected_address() {
        Some(expected) => {
            if expected != tx.from {
                return Err(ValidationError::new(
                    "E011",
                    format!("alamat {} tidak cocok dengan pubkey", tx.from),
                ));
            }
            Ok(())
        }
        None => Err(ValidationError::new(
            "E011",
            "public key tidak dapat diturunkan ke alamat",
        )),
    }
}

/// E012: Transaksi tidak boleh memakai signature kosong.
pub fn rule_signature_present(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.signature.is_empty() {
        return Err(ValidationError::new(
            "E012",
            "signature kosong",
        ));
    }
    if tx.public_key.is_empty() {
        return Err(ValidationError::new(
            "E012",
            "public key kosong",
        ));
    }
    Ok(())
}

// ============================================================
// ATURAN UTXO DAN SALDO
// ============================================================

/// Kumpulkan semua input yang sudah dipakai di seluruh chain.
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

/// Kumpulkan semua input yang dipakai di mempool.
pub fn mempool_inputs(chain: &Blockchain) -> HashSet<String> {
    let mut used = HashSet::new();
    for tx in &chain.mempool {
        for input in &tx.inputs {
            used.insert(input.clone());
        }
    }
    used
}

/// E020: Input tidak boleh sudah dipakai di chain atau mempool.
pub fn rule_no_double_spend(
    chain: &Blockchain,
    tx: &Transaction,
) -> Result<(), ValidationError> {
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

/// Hitung saldo matang sebuah alamat.
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

/// E021: Saldo pengirim harus cukup untuk amount + fee.
pub fn rule_sufficient_balance(
    chain: &Blockchain,
    tx: &Transaction,
) -> Result<(), ValidationError> {
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

/// E030: Blok tidak boleh punya lebih dari MAX_TX_PER_BLOCK transaksi.
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

/// E031: Coinbase reward harus sesuai era halving.
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

/// E032: Timestamp blok tidak boleh mundur dari blok sebelumnya.
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

// ============================================================
// PUNGGUNG VALIDASI
// ============================================================

/// Cek apakah alamat valid.
pub fn is_valid_address(addr: &str) -> bool {
    rule_address_format(addr).is_ok()
}

/// Validasi lengkap satu transaksi biasa (bukan coinbase).
/// Menggabungkan semua aturan non-UTXO dan UTXO.
pub fn validate_transaction(
    chain: &Blockchain,
    tx: &Transaction,
) -> Result<(), ValidationError> {
    // Transaksi khusus tidak divalidasi dengan aturan biasa.
    if is_special_tx(tx) {
        return Ok(());
    }

    rule_address_format(&tx.from)?;
    rule_address_format(&tx.to)?;
    rule_self_send(tx)?;
    rule_amount_nonzero(tx)?;
    rule_note_length(tx)?;
    rule_input_count(tx)?;
    rule_signature_present(tx)?;
    rule_signature(tx)?;
    rule_address_matches_pubkey(tx)?;
    rule_no_double_spend(chain, tx)?;
    rule_sufficient_balance(chain, tx)?;

    // TODO(rule): cek scriptPubKey.
    // TODO(rule): cek locktime.
    // TODO(rule): cek versi transaksi.
    // TODO(rule): cek dust limit.

    Ok(())
}

/// Validasi semua transaksi di dalam blok terhadap chain.
/// Chain yang dipakai adalah chain sebelum blok ini ditambahkan.
pub fn validate_block_transactions(
    chain: &Blockchain,
    transactions: &[Transaction],
) -> Result<(), ValidationError> {
    for tx in transactions {
        validate_transaction(chain, tx)?;
    }
    Ok(())
}

/// Validasi seluruh blok, termasuk aturan header.
pub fn validate_block(
    chain: &Blockchain,
    block: &Block,
    prev: &Block,
) -> Result<(), ValidationError> {
    rule_block_tx_count(block)?;
    rule_block_reward(block)?;
    rule_block_timestamp_monotonic(prev, block)?;
    rule_block_nbits(chain, block)?;
    validate_block_transactions(chain, &block.transactions)?;

    // TODO(rule): cek merkle root.
    // TODO(rule): cek previous_hash.
    // TODO(rule): cek PoW terhadap target.
    // TODO(rule): cek checkpoint.

    Ok(())
}

/// Validasi seluruh isi mempool.
/// Mengembalikan daftar error (kosong bila semua valid).
pub fn validate_mempool(chain: &Blockchain) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    for tx in &chain.mempool {
        if let Err(e) = validate_transaction(chain, tx) {
            errors.push(e);
        }
    }
    errors
}

/// Cek apakah transaksi termasuk kategori khusus.
pub fn is_special_tx(tx: &Transaction) -> bool {
    tx.from == "COINBASE"
        || tx.from == "GENESIS"
        || tx.from == "NETWORK"
        || tx.from == "MEMPOOL"
}

/// Cek apakah wallet punya saldo cukup untuk transaksi tertentu.
pub fn wallet_has_balance(
    wallet: &Wallet,
    chain: &Blockchain,
    amount_sat: u64,
    fee_sat: u64,
) -> bool {
    let balance = mature_balance(chain, &wallet.address);
    balance >= amount_sat.saturating_add(fee_sat)
}

// ============================================================
// TEMPAT ATURAN TAMBAHAN
// ============================================================
//
// Tambahkan fungsi `rule_*` baru di atas, lalu panggil dari
// `validate_transaction` atau `validate_block`.
//
// Kode error yang sudah dipakai:
//   E001-E003  format alamat
//   E004       self-send
//   E005       amount nol
//   E006       note terlalu panjang
//   E007       input terlalu banyak
//   E010-E012  signature
//   E020-E021  UTXO dan saldo
//   E030-E033  header blok
//
// Kode berikutnya: E040, E041, dst.
