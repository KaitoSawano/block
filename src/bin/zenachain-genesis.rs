// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use zenachain::block::Block;
use zenachain::consensus::spec::{self, Network};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut use_testnet = false;
    for arg in &args[1..] {
        match arg.as_str() {
            "-testnet" | "--testnet" => use_testnet = true,
            "--help" | "-h" => {
                println!("Usage: zenachain-genesis [-testnet|--testnet]");
                return;
            }
            other => {
                eprintln!("error: unknown option: {}", other);
                std::process::exit(1);
            }
        }
    }

    let network = if use_testnet {
        Network::Test
    } else {
        Network::Main
    };
    spec::set_current_network(network);

    let b = Block::genesis_unchecked();

    println!();
    println!("============================================================");
    println!("  ZENACHAIN GENESIS ({})", network.name());
    println!("  (copy into src/consensus/genesis.rs)");
    println!("============================================================");
    println!();
    if network == Network::Main {
        println!("  pub const EXPECTED_GENESIS_HASH: &str =");
        println!("      \"{}\";", b.hash);
        println!();
        println!("  pub const EXPECTED_GENESIS_MERKLE_ROOT: &str =");
        println!("      \"{}\";", b.merkle_root);
        println!();
        println!(
            "  pub const EXPECTED_GENESIS_TIMESTAMP: u128 = {};",
            b.timestamp
        );
        println!();
        println!("  pub const EXPECTED_GENESIS_NONCE: u64 = {};", b.nonce);
    } else {
        println!("  pub const TESTNET_EXPECTED_GENESIS_HASH: &str =");
        println!("      \"{}\";", b.hash);
        println!();
        println!("  pub const TESTNET_EXPECTED_GENESIS_MERKLE_ROOT: &str =");
        println!("      \"{}\";", b.merkle_root);
        println!();
        println!(
            "  pub const TESTNET_EXPECTED_GENESIS_TIMESTAMP: u128 = {};",
            b.timestamp
        );
        println!();
        println!(
            "  pub const TESTNET_EXPECTED_GENESIS_NONCE: u64 = {};",
            b.nonce
        );
    }
    println!("============================================================");
    println!();
}
