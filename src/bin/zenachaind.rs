// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! ZenaChain node daemon.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::{mpsc, RwLock};

use zenachain::block::Block;
use zenachain::blockchain::Blockchain;
use zenachain::config::Config;
use zenachain::consensus::spec::{self, Network};
use zenachain::p2p::network::{NetworkCommand, NetworkConfig, PeerRegistry, spawn};
use zenachain::rpc::RpcServer;
use zenachain::storage;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut selected_network = Network::Main;

    for arg in &args[1..] {
        match arg.as_str() {
            "-testnet" | "--testnet" => selected_network = Network::Test,
            "-regtest" | "--regtest" => selected_network = Network::Regtest,
            "--help" | "-h" => {
                println!("Usage: zenachaind [-testnet|--testnet] [-regtest|--regtest]");
                return;
            }
            "--version" => {
                println!("zenachaind {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown option: {}", other);
                std::process::exit(1);
            }
            _ => {
                eprintln!("error: zenachaind does not accept positional arguments");
                eprintln!("use `zenachain <command>` for RPC commands");
                std::process::exit(1);
            }
        }
    }

    spec::set_current_network(selected_network);

    println!("ZenaChain server starting (network: {})", selected_network.name());
    println!("Use `./zenachain stop` to stop the node.");
    println!("Ctrl+C, kill, and SIGHUP do not stop the node (same as Bitcoin).");

    let config = Config::load_or_default();
    println!("[config] network      : {}", config.network.name());
    println!("[config] rpc_addr     : {}", config.rpc_addr);
    println!("[config] listen_addr  : {}", config.listen_addr);
    println!("[config] bootstrap    : {:?}", config.bootstrap);
    println!("[config] mining_addr  : {}", config.mining_address);
    println!("[config] location     : {}", Config::default_path());

    if !std::path::Path::new(&Config::default_path()).exists() {
        let _ = config.save_default();
        println!(
            "[config] configuration file created at {}",
            Config::default_path()
        );
    }

    let _ = Block::genesis();

    let chain = Arc::new(RwLock::new(
        storage::load_chain().unwrap_or_else(|_| Blockchain::new()),
    ));

    let peer_registry: PeerRegistry = Arc::new(Mutex::new(Vec::new()));

    let (cmd_tx, cmd_rx) = mpsc::channel::<NetworkCommand>(100);
    let cmd_tx_for_shutdown = cmd_tx.clone();

    let listen_addr = match config.listen_addr.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("[p2p] invalid listen address: {}", e);
            std::process::exit(1);
        }
    };

    let mut bootstrap = Vec::new();
    // Regtest does not require bootstrap peers.
    if selected_network != Network::Regtest {
        for b in &config.bootstrap {
            if let Ok(sa) = b.parse() {
                bootstrap.push(sa);
            } else {
                eprintln!("[p2p] invalid bootstrap address: {}", b);
            }
        }
    }

    let net_config = NetworkConfig {
        listen_addr,
        bootstrap,
        ..Default::default()
    };

    let chain_for_rpc = chain.clone();
    let cfg_for_rpc = config.clone();
    let rpc_registry = peer_registry.clone();
    let (rpc_shutdown, rpc_handle) = {
        let mut server = RpcServer::new(
            chain_for_rpc.clone(),
            &cfg_for_rpc,
            rpc_registry,
            cmd_tx,
            selected_network,
        );
        let flag = server.shutdown_flag();
        let handle = tokio::task::spawn_blocking(move || server.listen());
        (flag, handle)
    };

    let net_handle = spawn(net_config, chain.clone(), peer_registry, cmd_rx);

    tokio::spawn(async move {
        match net_handle.await {
            Ok(Ok(())) => println!("[p2p] network loop exited"),
            Ok(Err(e)) => {
                eprintln!("[p2p] fatal: {}", e);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("[p2p] fatal: task panicked: {}", e);
                std::process::exit(1);
            }
        }
    });

    install_signal_handlers();

    loop {
        if rpc_shutdown.load(Ordering::SeqCst) {
            println!("[node] shutdown requested via RPC");
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    let guard = chain.read().await;
    if let Err(e) = storage::save_chain(&guard) {
        eprintln!("[node] failed to save chain: {}", e);
    } else {
        println!("[node] chain saved");
    }
    drop(guard);

    let _ = cmd_tx_for_shutdown.send(NetworkCommand::Shutdown).await;
    let _ = tokio::time::timeout(Duration::from_secs(5), rpc_handle).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("[node] shutdown complete");
}

fn install_signal_handlers() {
    use tokio::signal::unix::{signal, SignalKind};

    let kinds = [
        (SignalKind::interrupt(), "SIGINT (Ctrl+C)"),
        (SignalKind::terminate(), "SIGTERM (kill)"),
        (SignalKind::hangup(), "SIGHUP (terminal closed)"),
        (SignalKind::quit(), "SIGQUIT (Ctrl+\\)"),
    ];

    for (kind, name) in kinds {
        match signal(kind) {
            Ok(mut sig) => {
                tokio::spawn(async move {
                    loop {
                        if sig.recv().await.is_some() {
                            println!();
                            println!(
                                "[signal] {} received. The node continues to run.",
                                name
                            );
                            println!("[signal] Use `./zenachain stop` to stop the node.");
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("[signal] failed to install handler for {}: {}", name, e);
            }
        }
    }
}
