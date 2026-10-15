use sha3::{Digest, Sha3_256};

/// Hash SHA3-256 dalam bentuk hex (64 karakter).
pub fn sha3_256_hex(data: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Waktu sekarang dalam milidetik Unix.
pub fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
