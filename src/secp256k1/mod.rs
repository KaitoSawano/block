// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! secp256k1 cryptographic primitives for ZenaChain.
//!
//! This module uses the `secp256k1` crate (rust-bitcoin), which is a
//! wrapper around libsecp256k1, the same library used by Bitcoin Core.
//!
//! Parameters: ECDSA on the secp256k1 curve.

pub mod address;
pub mod keypair;
pub mod sign;
pub mod verify;

/// Size of a secp256k1 secret key in bytes.
pub const SECRET_KEY_SIZE: usize = 32;

/// Size of a compressed secp256k1 public key in bytes.
pub const PUBLIC_KEY_SIZE: usize = 33;

/// Minimum size of an ECDSA DER-encoded signature.
pub const SIGNATURE_MIN_SIZE: usize = 70;

/// Maximum size of an ECDSA DER-encoded signature.
pub const SIGNATURE_MAX_SIZE: usize = 72;
