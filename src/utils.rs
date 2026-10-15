use sha3::{Digest, Sha3_256};

pub fn sha3_256_hex(data: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn satoshi_to_zenac(satoshi: u64) -> String {
    let whole = satoshi / 100_000_000;
    let frac = satoshi % 100_000_000;
    format!("{}.{:08}", whole, frac)
}

pub fn zenac_to_satoshi(zenac: f64) -> u64 {
    (zenac * 100_000_000.0) as u64
}
