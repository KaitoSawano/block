// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Base58Check encoding and decoding for ZenaChain addresses.

use zenachain_crypto::sha3_256_hex; // For the checksum hash.

/// Encode a payload (prefix + hash) into a Base58Check string.
/// The payload is extended with a 4-byte checksum, then encoded as Base58.
pub fn encode(payload: &[u8]) -> String {
    let mut body = payload.to_vec(); // Copy the payload into a mutable buffer.
    let checksum = compute_checksum(payload); // Compute the 4-byte checksum.
    body.extend_from_slice(&checksum); // Append the checksum.
    bs58::encode(body).into_string() // Encode to Base58.
}

/// Decode a Base58Check string into a payload (prefix + hash).
/// Returns `None` if the string is not valid Base58 or the checksum is wrong.
pub fn decode(address: &str) -> Option<Vec<u8>> {
    let decoded = bs58::decode(address).into_vec().ok()?; // Decode Base58.
    if decoded.len() < 4 {
        return None; // Too short to contain a checksum.
    }
    let (payload, checksum) = decoded.split_at(decoded.len() - 4); // Split payload and checksum.
    let expected = compute_checksum(payload); // Compute the expected checksum.
    if checksum != expected {
        return None; // Checksum mismatch.
    }
    Some(payload.to_vec()) // Return the payload.
}

/// Compute the 4-byte checksum of a payload.
/// The checksum is the first 4 bytes of SHA3-256(SHA3-256(payload)).
fn compute_checksum(payload: &[u8]) -> [u8; 4] {
    let first_hash = sha3_256_hex(payload); // First SHA3-256.
    let first_bytes = hex::decode(&first_hash).expect("sha3 hex is always valid");
    let second_hash = sha3_256_hex(&first_bytes); // Second SHA3-256.
    let second_bytes = hex::decode(&second_hash).expect("sha3 hex is always valid");
    let mut checksum = [0u8; 4];
    checksum.copy_from_slice(&second_bytes[..4]); // Take the first 4 bytes.
    checksum
}
