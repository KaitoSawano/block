use crate::blockchain::Blockchain;
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use std::collections::HashSet;

pub const COINBASE_MATURITY: u64 = 100;
pub const ADDRESS_PREFIX: &str = "ZENA";
pub const ADDRESS_HEX_LEN: usize = 40;

/// Cek format alamat ZENA.
/// Format: "ZENA" + 40 karakter hex.
pub fn is_valid_address(addr: &str) -> bool {
    if !addr.starts_with(ADDRESS_PREFIX) {
        return false;
    }
    let hex_part = &addr[ADDRESS_PREFIX.len()..];
    if hex_part.len() != ADDRESS_HEX_LEN {
        return false;
    }
    hex_part.chars().all(|c| c.is_ascii_hexdigit())
}

/// Kumpulkan semua input yang sudah dipakai di seluruh chain.
fn spent_inputs(chain: &Blockchain) -> HashSet<String> {
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
fn mempool_inputs(chain: &Blockchain) -> HashSet<String> {
    let mut used = HashSet::new();
    for tx in &chain.mempool {
        for input in &tx.inputs {
            used.insert(input.clone());
        }
    }
    used
}

/// Hitung saldo matang sebuah alamat.
/// Sumber: transaksi masuk yang belum dibelanjakan + reward blok yang sudah matang.
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

/// Cek apakah transaksi punya input yang sudah dipakai di chain atau mempool.
/// Return daftar input yang bentrok.
pub fn find_double_spend(chain: &Blockchain, tx: &Transaction) -> Vec<String> {
    let spent_chain = spent_inputs(chain);
    let spent_mempool = mempool_inputs(chain);
    let mut conflicts = Vec::new();

    for input in &tx.inputs {
        if spent_chain.contains(input) || spent_mempool.contains(input) {
            conflicts.push(input.clone());
        }
    }

    conflicts
}

/// Cek apakah pengirim punya saldo cukup untuk amount + fee.
pub fn has_sufficient_balance(chain: &Blockchain, tx: &Transaction) -> bool {
    let balance = mature_balance(chain, &tx.from);
    let need = tx.amount.saturating_add(tx.fee);
    balance >= need
}

/// Validasi lengkap satu transaksi terhadap chain.
/// Return Err dengan pesan bila tidak valid.
pub fn validate_transaction(chain: &Blockchain, tx: &Transaction) -> Result<(), String> {
    // 1. Format alamat.
    if !is_valid_address(&tx.from) {
        return Err(format!("alamat pengirim tidak valid: {}", tx.from));
    }
    if !is_valid_address(&tx.to) {
        return Err(format!("alamat tujuan tidak valid: {}", tx.to));
    }

    // 2. Tidak boleh kirim ke diri sendiri.
    if tx.from == tx.to {
        return Err("alamat pengirim dan tujuan sama".into());
    }

    // 3. Amount harus lebih dari nol.
    if tx.amount == 0 {
        return Err("amount harus lebih dari nol".into());
    }

    // 4. Signature.
    if !tx.verify_signature() {
        return Err("signature tidak valid".into());
    }

    // 5. Alamat harus cocok dengan public key.
    if let Some(expected) = tx.expected_address() {
        if expected != tx.from {
            return Err("alamat tidak cocok dengan public key".into());
        }
    } else {
        return Err("public key tidak dapat diturunkan ke alamat".into());
    }

    // 6. Double-spend.
    let conflicts = find_double_spend(chain, tx);
    if !conflicts.is_empty() {
        return Err(format!("input sudah dipakai: {}", conflicts.join(",")));
    }

    // 7. Saldo cukup.
    if !has_sufficient_balance(chain, tx) {
        let balance = mature_balance(chain, &tx.from);
        let need = tx.amount.saturating_add(tx.fee);
        return Err(format!(
            "saldo tidak cukup: butuh {} sat, tersedia {} sat",
            need, balance
        ));
    }

    Ok(())
}

/// Validasi semua transaksi non-khusus di dalam mempool.
/// Return daftar pesan error (kosong bila semua valid).
pub fn validate_mempool(chain: &Blockchain) -> Vec<String> {
    let mut errors = Vec::new();
    for tx in &chain.mempool {
        // Transaksi khusus (NETWORK, MEMPOOL, dst) dilewati.
        if tx.from == "NETWORK" || tx.from == "MEMPOOL" || tx.from == "COINBASE"
            || tx.from == "GENESIS"
        {
            continue;
        }

        if let Err(e) = validate_transaction(chain, tx) {
            errors.push(format!("{}: {}", tx.txid, e));
        }
    }
    errors
}

/// Validasi blok: pastikan semua transaksi di dalamnya valid.
pub fn validate_block_transactions(chain: &Blockchain, transactions: &[Transaction]) -> Result<(), String> {
    for tx in transactions {
        if tx.from == "COINBASE" || tx.from == "GENESIS" || tx.from == "NETWORK"
            || tx.from == "MEMPOOL"
        {
            continue;
        }

        if !tx.verify_signature() {
            return Err(format!("transaksi {} signature tidak valid", tx.txid));
        }

        if let Some(expected) = tx.expected_address() {
            if expected != tx.from {
                return Err(format!("transaksi {} alamat tidak cocok", tx.txid));
            }
        }

        // Double-spend terhadap chain yang ada saat ini (blok sebelumnya).
        let conflicts = find_double_spend(chain, tx);
        if !conflicts.is_empty() {
            return Err(format!(
                "transaksi {} double-spend: {}",
                tx.txid,
                conflicts.join(",")
            ));
        }
    }

    Ok(())
}

/// Cek apakah wallet saat ini punya saldo cukup untuk transaksi tertentu.
pub fn wallet_has_balance(wallet: &Wallet, chain: &Blockchain, amount_sat: u64, fee_sat: u64) -> bool {
    let balance = mature_balance(chain, &wallet.address);
    balance >= amount_sat.saturating_add(fee_sat)
}
