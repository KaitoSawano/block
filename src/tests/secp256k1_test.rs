// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use zenachain::secp256k1::{address, keypair::KeyPair, sign, verify};
use zenachain_crypto::sha3_256_hex;

#[test]
fn generate_keypair_has_correct_lengths() {
    let kp = KeyPair::generate().expect("keypair generation failed");
    assert_eq!(kp.public_key.len(), 33);
    assert_eq!(kp.secret_key.len(), 32);
}

#[test]
fn sign_and_verify_roundtrip() {
    let kp = KeyPair::generate().expect("keypair generation failed");
    let msg_hex = sha3_256_hex(b"hello zenachain");
    let msg = hex::decode(&msg_hex).expect("hex decode failed");
    let sig = sign::sign(&kp.secret_key, &msg).expect("signing failed");
    assert!(verify::verify(&kp.public_key, &msg, &sig));
}

#[test]
fn verify_rejects_wrong_message() {
    let kp = KeyPair::generate().expect("keypair generation failed");
    let msg_hex = sha3_256_hex(b"hello zenachain");
    let msg = hex::decode(&msg_hex).expect("hex decode failed");
    let sig = sign::sign(&kp.secret_key, &msg).expect("signing failed");
    let other = hex::decode(&sha3_256_hex(b"other message")).expect("hex decode failed");
    assert!(!verify::verify(&kp.public_key, &other, &sig));
}

#[test]
fn address_is_deterministic() {
    let kp = KeyPair::generate().expect("keypair generation failed");
    let a1 = address::address_from_public_key(&kp.public_key).expect("address failed");
    let a2 = address::address_from_public_key(&kp.public_key).expect("address failed");
    assert_eq!(a1, a2);
}

#[test]
fn address_has_correct_format() {
    let kp = KeyPair::generate().expect("keypair generation failed");
    let addr = address::address_from_public_key(&kp.public_key).expect("address failed");
    assert!(addr.starts_with("ZENA"));
    assert_eq!(addr.len(), 44);
}
