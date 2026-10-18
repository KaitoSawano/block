// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! ZenaChain standalone miner.
//!
//! This binary requests a block template from the node, mines the block,
//! and submits it back to the node through the RPC interface.

use zenachain::block::Block;
use zenachain::rpc;
use zenachain::transaction::Transaction;

fn main() {
    println!("[miner] zenachain-miner started");
    println!("[miner] connecting to the node at {}", rpc::DEFAULT_RPC_ADDR);

    loop {
        let template = match rpc::call("getblocktemplate", vec![]) {
            Ok(r) if r.ok => r.result,
            Ok(r) => {
                println!("[miner] node rejected the request: {}", r.error);
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
            Err(e) => {
                println!("[miner] cannot reach the node: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let height = template["height"].as_u64().unwrap_or(0);
        let previous_hash = template["previous_hash"].as_str().unwrap_or("").to_string();
        let nbits = template["nbits"].as_u64().unwrap_or(0) as u32;
        
        // Ambil data dari template RPC (yang sudah dihitung oleh node)
        let subsidy_satoshi = template["subsidy_satoshi"].as_u64().unwrap_or(0);
        let total_fees_satoshi = template["total_fees_satoshi"].as_u64().unwrap_or(0);
        let reward_satoshi = template["reward_satoshi"].as_u64().unwrap_or(0);

        let mut mempool: Vec<Transaction> =
            serde_json::from_value(template["mempool"].clone()).unwrap_or_default();

        let target_address =
            std::env::var("ZENAC_MINE_FOR").unwrap_or_else(|_| "UNKNOWN".to_string());

        if target_address == "UNKNOWN" {
            println!("[miner] ZENAC_MINE_FOR is not set. Set it to a valid ZENA address.");
            std::thread::sleep(std::time::Duration::from_secs(10));
            continue;
        }

        // Tambahkan transaksi MEMPOOL sebagai penanda (opsional, bisa dihapus)
        mempool.push(Transaction::new_with_inputs(
            "MEMPOOL",
            "CreateNewBlock",
            0,
            0,
            Vec::new(),
            format!("height={} target={}", height, target_address),
        ));

        // Panggil Block::new_for dengan 7 argumen (termasuk total_fees)
        let mut block = Block::new_for(
            height,
            previous_hash,
            subsidy_satoshi,
            total_fees_satoshi,
            nbits,
            mempool,
            target_address.clone(),
        );

        // Pastikan reward di blok sama dengan yang diharapkan (opsional)
        if block.reward != reward_satoshi {
            println!(
                "[miner] WARNING: reward mismatch. Block reward: {}, Template reward: {}",
                block.reward, reward_satoshi
            );
        }

        if !zenachain::pow::mine(&mut block, nbits) {
            println!("[miner] mining failed: nonce exhausted");
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }

        let block_json = serde_json::to_value(&block).unwrap();
        match rpc::call("submitblock", vec![block_json]) {
            Ok(r) if r.ok => {
                println!(
                    "[miner] block accepted: {}",
                    serde_json::to_string(&r.result).unwrap()
                );
            }
            Ok(r) => {
                println!("[miner] block rejected: {}", r.error);
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            Err(e) => {
                println!("[miner] submission failed: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
    }
}
