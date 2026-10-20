// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Storage layer for ZenaChain (LevelDB + Merkle Patricia Trie).

pub mod account;
pub mod adapter;
pub mod state;

pub use account::Account;
pub use state::StateDB;
