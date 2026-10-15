use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub note: String,
}

impl Transaction {
    pub fn new(from: &str, to: &str, amount: u64, fee: u64) -> Self {
        println!("[transaction] new: membuat transaksi biasa antar dua alamat.");
        Transaction {
            from: from.into(),
            to: to.into(),
            amount,
            fee,
            note: String::new(),
        }
    }

    pub fn coinbase(note: String) -> Self {
        println!("[transaction] coinbase: transaksi khusus penerbitan hadiah blok.");
        Transaction {
            from: "COINBASE".into(),
            to: "MINER".into(),
            amount: 0,
            fee: 0,
            note,
        }
    }

    pub fn secret(note: String) -> Self {
        println!("[transaction] secret: transaksi khusus penyimpanan pesan genesis.");
        Transaction {
            from: "GENESIS".into(),
            to: "ZENAC".into(),
            amount: 0,
            fee: 0,
            note,
        }
    }

    pub fn is_valid(&self) -> bool {
        println!("[transaction] is_valid: memeriksa kelengkapan field transaksi.");
        !self.to.is_empty() && !self.from.is_empty()
    }
}
