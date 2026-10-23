// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! ZenaChain node daemon.

use std::sync::atomic::Ordering; // For atomic operations.
use std::sync::{Arc, Mutex}; // For shared state.
use std::time::Duration; // For timeouts.

use tokio::sync::{mpsc, RwLock}; // For async synchronization.

use zenachain::block::Block; // The block structure.
use zenachain::blockchain::Blockchain; // The blockchain struct.
use zenachain::config::Config; // The configuration.
use zenachain::consensus::spec::{self, Network}; // Network selection.
use zenachain::p2p::network::{NetworkCommand, NetworkConfig, PeerRegistry, spawn}; // P2P networking.
use zenachain::rpc::RpcServer; // The RPC server.
use zenachain::storage; // The storage layer.

// Flag to indicate whether the node is running as a daemon (background process).
static IS_DAEMON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect(); // Collect command-line arguments.

    let mut selected_network = Network::Main; // Default to Mainnet.

    for arg in &args[1..] {
        match arg.as_str() {
            "-testnet" | "--testnet" => selected_network = Network::Test, // Select Testnet.
            "-regtest" | "--regtest" => selected_network = Network::Regtest, // Select Regtest.
            "--help" | "-h" => {
                println!("Usage: zenachaind [-testnet|--testnet] [-regtest|--regtest]"); // Print help.
                return;
            }
            "--version" => {
                println!("zenachaind {}", env!("CARGO_PKG_VERSION")); // Print version.
                return;
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown option: {}", other); // Unknown option.
                std::process::exit(1);
            }
            _ => {
                eprintln!("error: zenachaind does not accept positional arguments"); // Positional arguments are not allowed.
                eprintln!("use `zenachain <command>` for RPC commands");
                std::process::exit(1);
            }
        }
    }

    spec::set_current_network(selected_network); // Set the active network.

    println!("ZenaChain server starting (network: {})", selected_network.name()); // Log the network.
    println!("Press Ctrl+C to detach the node into the background."); // Log the detach behavior.
    println!("Use `./zenachain stop` to stop the node completely."); // Log the stop command.

    let config = Config::load_or_default(); // Load the configuration.
    println!("[config] network      : {}", config.network.name()); // Log the network.
    println!("[config] rpc_addr     : {}", config.rpc_addr); // Log the RPC address.
    println!("[config] listen_addr  : {}", config.listen_addr); // Log the P2P listen address.
    println!("[config] bootstrap    : {:?}", config.bootstrap); // Log the bootstrap peers.
    println!("[config] mining_addr  : {}", config.mining_address); // Log the mining address.
    println!("[config] location     : {}", Config::default_path()); // Log the config file location.

    if !std::path::Path::new(&Config::default_path()).exists() {
        let _ = config.save_default(); // Save the default config if it does not exist.
        println!(
            "[config] configuration file created at {}",
            Config::default_path()
        );
    }

    let _ = Block::genesis(); // Initialize the genesis block.

    let chain = Arc::new(RwLock::new(
        storage::load_chain().unwrap_or_else(|_| Blockchain::new()), // Load the chain or create a new one.
    ));

    let peer_registry: PeerRegistry = Arc::new(Mutex::new(Vec::new())); // Create the peer registry.

    let (cmd_tx, cmd_rx) = mpsc::channel::<NetworkCommand>(100); // Create the network command channel.
    let cmd_tx_for_shutdown = cmd_tx.clone(); // Clone the sender for shutdown.

    let listen_addr = match config.listen_addr.parse() {
        Ok(a) => a, // Parse the listen address.
        Err(e) => {
            eprintln!("[p2p] invalid listen address: {}", e); // Log the error.
            std::process::exit(1);
        }
    };

    let mut bootstrap = Vec::new(); // The list of bootstrap peers.
    if selected_network != Network::Regtest {
        // Regtest does not use bootstrap peers.
        for b in &config.bootstrap {
            if let Ok(sa) = b.parse() {
                bootstrap.push(sa); // Add the bootstrap peer.
            } else {
                eprintln!("[p2p] invalid bootstrap address: {}", b); // Log the error.
            }
        }
    }

    let net_config = NetworkConfig {
        listen_addr, // The listen address.
        bootstrap, // The bootstrap peers.
        ..Default::default() // Use defaults for the rest.
    };

    let chain_for_rpc = chain.clone(); // Clone the chain for the RPC server.
    let cfg_for_rpc = config.clone(); // Clone the config for the RPC server.
    let rpc_registry = peer_registry.clone(); // Clone the registry for the RPC server.
    let (rpc_shutdown, rpc_handle) = {
        let mut server = RpcServer::new(
            chain_for_rpc.clone(), // The chain.
            &cfg_for_rpc, // The config.
            rpc_registry, // The peer registry.
            cmd_tx, // The network command sender.
            selected_network, // The network.
        );
        let flag = server.shutdown_flag(); // Get the shutdown flag.
        let handle = tokio::task::spawn_blocking(move || server.listen()); // Spawn the RPC server.
        (flag, handle)
    };

    let net_handle = spawn(net_config, chain.clone(), peer_registry, cmd_rx); // Spawn the P2P network.

    tokio::spawn(async move {
        match net_handle.await {
            Ok(Ok(())) => println!("[p2p] network loop exited"), // Log normal exit.
            Ok(Err(e)) => {
                eprintln!("[p2p] fatal: {}", e); // Log fatal error.
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("[p2p] fatal: task panicked: {}", e); // Log panic.
                std::process::exit(1);
            }
        }
    });

    // Install signal handlers that trigger a graceful shutdown or daemonize.
    install_signal_handlers(rpc_shutdown.clone()); // Pass the shutdown flag.

    loop {
        if rpc_shutdown.load(Ordering::SeqCst) {
            // If the shutdown flag is set...
            println!("[node] shutdown requested"); // Log the shutdown.
            break; // Exit the loop.
        }
        tokio::time::sleep(Duration::from_millis(200)).await; // Sleep for 200ms.
    }

    let guard = chain.read().await; // Acquire a read lock on the chain.
    if let Err(e) = storage::save_chain(&guard) {
        eprintln!("[node] failed to save chain: {}", e); // Log the error.
    } else {
        println!("[node] chain saved"); // Log success.
    }
    drop(guard); // Release the lock.

    let _ = cmd_tx_for_shutdown.send(NetworkCommand::Shutdown).await; // Send the shutdown command to P2P.
    let _ = tokio::time::timeout(Duration::from_secs(5), rpc_handle).await; // Wait for the RPC server to stop.
    tokio::time::sleep(Duration::from_millis(500)).await; // Give the P2P network time to stop.

    println!("[node] shutdown complete"); // Log completion.
}

// Installs signal handlers that trigger a graceful shutdown or daemonize.
fn install_signal_handlers(shutdown_flag: Arc<std::sync::atomic::AtomicBool>) {
    use tokio::signal::unix::{signal, SignalKind}; // For Unix signal handling.

    // SIGINT (Ctrl+C) triggers a detach (daemonize).
    // SIGTERM (kill) triggers a graceful shutdown.
    // SIGHUP (terminal closed) triggers a detach.
    // SIGQUIT (Ctrl+\) is a debug signal that does nothing.

    // Install SIGINT handler for detaching.
    match signal(SignalKind::interrupt()) {
        Ok(mut sig) => {
            tokio::spawn(async move {
                if sig.recv().await.is_some() {
                    // If SIGINT is received...
                    println!(); // Print a newline.
                    println!("[signal] SIGINT (Ctrl+C) received. Detaching into the background...");
                    daemonize(); // Detach the process.
                    // The parent process exits here; the child continues.
                }
            });
        }
        Err(e) => {
            eprintln!("[signal] failed to install handler for SIGINT: {}", e);
        }
    }

    // Install SIGTERM handler for graceful shutdown.
    match signal(SignalKind::terminate()) {
        Ok(mut sig) => {
            let flag = shutdown_flag.clone(); // Clone the flag for the task.
            tokio::spawn(async move {
                if sig.recv().await.is_some() {
                    // If SIGTERM is received...
                    println!(); // Print a newline.
                    println!("[signal] SIGTERM (kill) received. Initiating graceful shutdown...");
                    flag.store(true, Ordering::SeqCst); // Set the shutdown flag.
                }
            });
        }
        Err(e) => {
            eprintln!("[signal] failed to install handler for SIGTERM: {}", e);
        }
    }

    // Install SIGHUP handler for detaching.
    match signal(SignalKind::hangup()) {
        Ok(mut sig) => {
            tokio::spawn(async move {
                if sig.recv().await.is_some() {
                    // If SIGHUP is received...
                    println!(); // Print a newline.
                    println!("[signal] SIGHUP (terminal closed) received. Detaching into the background...");
                    daemonize(); // Detach the process.
                }
            });
        }
        Err(e) => {
            eprintln!("[signal] failed to install handler for SIGHUP: {}", e);
        }
    }

    // Install SIGQUIT handler for debug (does nothing).
    match signal(SignalKind::quit()) {
        Ok(mut sig) => {
            tokio::spawn(async move {
                loop {
                    if sig.recv().await.is_some() {
                        // If SIGQUIT is received...
                        println!(); // Print a newline.
                        println!("[signal] SIGQUIT (Ctrl+\\) received. The node continues to run."); // Log the signal.
                    }
                }
            });
        }
        Err(e) => {
            eprintln!("[signal] failed to install handler for SIGQUIT: {}", e);
        }
    }
}

// Detaches the current process from the terminal and continues in the background.
// This is a simplified daemonization: the parent exits, and the child continues.
// Note: This does NOT close stdout/stderr, so the node will still print to the
// terminal until it is closed. For a full daemonization, you would also redirect
// stdout/stderr to a log file and call `setsid()`.
fn daemonize() {
    use nix::unistd::{fork, ForkResult}; // For fork().
    use std::process::exit; // For exit().

    if IS_DAEMON.load(Ordering::SeqCst) {
        // If already a daemon, do nothing.
        println!("[daemon] already running in the background.");
        return;
    }

    match unsafe { fork() } {
        Ok(ForkResult::Parent { .. }) => {
            // Parent process: exit gracefully.
            println!("[daemon] parent process exiting; node continues in the background.");
            exit(0); // Exit the parent.
        }
        Ok(ForkResult::Child) => {
            // Child process: continue running as the daemon.
            IS_DAEMON.store(true, Ordering::SeqCst); // Mark as daemon.
            println!("[daemon] node is now running in the background.");
            // Note: The child inherits the terminal's stdout/stderr. For a full
            // daemonization, you would redirect them to a log file here.
        }
        Err(e) => {
            // Fork failed: log the error and continue in the foreground.
            eprintln!("[daemon] fork failed: {}. Continuing in the foreground.", e);
        }
    }
}
