// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! ZenaChain library crate.

pub mod block; // Block structure and related operations.
pub mod blockchain; // Blockchain state and consensus-related operations.
pub mod checksum; // Address checksum computation and verification.
pub mod config; // Configuration file parsing and persistence.
pub mod consensus; // Consensus parameters and rules.
pub mod datadir; // Data directory layout and path helpers.
pub mod mempool; // Mempool representation.
pub mod node; // Node state container.
pub mod p2p; // Peer-to-peer networking.
pub mod pow; // Proof-of-work primitives.
pub mod rpc; // RPC server and client.
pub mod secp256k1; // secp256k1 cryptographic primitives.
pub mod storage; // Storage layer.
pub mod storage_legacy; // Legacy storage functions.
pub mod transaction; // Transaction structure.
pub mod utils; // Utility functions.
pub mod validation; // Consensus validation rules.
pub mod wallet; // Wallet representation and persistence.
