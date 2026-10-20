// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Storage layer for ZenaChain.

pub mod account;
pub mod adapter;
pub mod state;

pub use account::Account;
pub use state::StateDB;

// Re-export the legacy storage functions (blocks, mempool) from the parent module.
pub use crate::storage_legacy::*;
