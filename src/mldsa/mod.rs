//! Modul ML-DSA-65 (Dilithium3) untuk ZenaChain.
//!
//! Menggunakan crate `dcrypt`, implementasi pure Rust FIPS 204
//! dengan zero unsafe code dan constant-time execution.
//!
//! Parameter: ML-DSA-65, NIST Security Level 3.

pub mod address;
pub mod keypair;
pub mod sign;
pub mod verify;

/// Ukuran public key ML-DSA-65 dalam byte.
pub const PUBLIC_KEY_SIZE: usize = 1952;

/// Ukuran secret key ML-DSA-65 dalam byte.
pub const SECRET_KEY_SIZE: usize = 4032;

/// Ukuran signature ML-DSA-65 dalam byte.
pub const SIGNATURE_SIZE: usize = 3309;
