// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Cryptographic primitives for ZenaChain.

pub mod hash;
pub mod encryption;

pub use hash::sha3_256_hex;
pub use encryption::{aes_decrypt, aes_encrypt, derive_key};
