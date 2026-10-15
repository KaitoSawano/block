//! Modul secp256k1 untuk ZenaChain.
//!
//! Menggunakan crate `secp256k1` (rust-bitcoin), wrapper atas
//! libsecp256k1 yang dipakai Bitcoin Core.
//!
//! Parameter: ECDSA di kurva secp256k1.

pub mod address;
pub mod keypair;
pub mod sign;
pub mod verify;

/// Ukuran secret key secp256k1 dalam byte.
pub const SECRET_KEY_SIZE: usize = 32;

/// Ukuran compressed public key secp256k1 dalam byte.
pub const PUBLIC_KEY_SIZE: usize = 33;

/// Ukuran minimum signature ECDSA DER encoding.
pub const SIGNATURE_MIN_SIZE: usize = 70;

/// Ukuran maksimum signature ECDSA DER encoding.
pub const SIGNATURE_MAX_SIZE: usize = 72;
