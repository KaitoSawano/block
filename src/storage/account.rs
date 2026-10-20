// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Account structure for the Ethereum-style state trie.

use rlp::{RlpStream, Encodable, Decodable, DecoderError, Rlp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub nonce: u64,
    pub balance: u64,
    /// Balance received from mining that is not yet mature and cannot be spent.
    pub immature_balance: u64,
    /// The block height when the last reward was credited.
    pub last_reward_height: u64,
}

impl Encodable for Account {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(4);
        s.append(&self.nonce);
        s.append(&self.balance);
        s.append(&self.immature_balance);
        s.append(&self.last_reward_height);
    }
}

impl Decodable for Account {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(Account {
            nonce: rlp.val_at(0)?,
            balance: rlp.val_at(1)?,
            immature_balance: rlp.val_at(2).unwrap_or(0),
            last_reward_height: rlp.val_at(3).unwrap_or(0),
        })
    }
}
