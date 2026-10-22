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
    /// List of immature rewards: (block_height, amount).
    pub immature_rewards: Vec<(u64, u64)>,
}

impl Encodable for Account {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);
        s.append(&self.nonce);
        s.append(&self.balance);
        s.begin_list(self.immature_rewards.len());
        for (height, amount) in &self.immature_rewards {
            s.begin_list(2);
            s.append(height);
            s.append(amount);
        }
    }
}

impl Decodable for Account {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let nonce: u64 = rlp.val_at(0)?;
        let balance: u64 = rlp.val_at(1)?;
        let immature_list = rlp.at(2)?;
        let mut immature_rewards = Vec::new();
        for i in 0..immature_list.item_count()? {
            let item = immature_list.at(i)?;
            let height: u64 = item.val_at(0)?;
            let amount: u64 = item.val_at(1)?;
            immature_rewards.push((height, amount));
        }
        Ok(Account {
            nonce,
            balance,
            immature_rewards,
        })
    }
}
