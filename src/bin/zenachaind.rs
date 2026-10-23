// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! ZenaChain node daemon.

use std::sync::atomic::{AtomicBool, Ordering}; // For atomic operations.
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

// A flag that indicates a full shutdown was requested (via RPC `stop` or SIGTERM).
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect(); // Collect command-line arguments.

    let mut selected_network = Network::Main; // Default to Mainnet.
    let mut is_daemon_child = false; // Whether this is the background child process.

    for arg in &args[1..] {
        match arg.as_str() {
            "-testnet" | "--testnet" => selected_network = Network::Test, // Select Testnet.
            "-regtest" | "--regtest" => selected_network = Network::Regtest, // Select Regtest.
            "--daemon-child" => is_daemon_child = true, // Internal flag: this is the background child.
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
    if is_daemon_child {
        println!("[daemon] running in the background."); // Log that this is the background child.
    } else {
        println!("Press Ctrl+C to detach the node into the background (the node will keep running)."); // Log the detach behavior.
        println!("Use `./zenachain stop` to stop the node completely."); // Log the stop command.
    }

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

    // Install signal handlers.
    install_signal_handlers(rpc_shutdown.clone(), is_daemon_child);

    // Main loop: wait for a full shutdown request.
    loop {
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) || rpc_shutdown.load(Ordering::SeqCst) {
            // Full shutdown requested (via RPC `stop` or SIGTERM).
            println!("[node] full shutdown requested");
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await; // Sleep for 200ms.
    }

    // Perform a graceful shutdown: save the chain and release the LevelDB lock.
    {
        let guard = chain.read().await; // Acquire a read lock on the chain.
        if let Err(e) = storage::save_chain(&guard) {
            eprintln!("[node] failed to save chain: {}", e); // Log the error.
        } else {
            println!("[node] chain saved"); // Log success.
        }
        drop(guard); // Release the lock.
    }

    // Wait for the RPC server to stop.
    let _ = cmd_tx_for_shutdown.send(NetworkCommand::Shutdown).await; // Send the shutdown command to P2P.
    let _ = tokio::time::timeout(Duration::from_secs(5), rpc_handle).await; // Wait for the RPC server to stop.
    tokio::time::sleep(Duration::from_millis(500)).await; // Give the P2P network time to stop.

    println!("[node] shutdown complete"); // Log completion.
}

// Installs signal handlers.
// If `is_daemon_child` is true, SIGINT and SIGHUP are ignored.
// Otherwise, SIGINT and SIGHUP request a detach (handover to background).
fn install_signal_handlers(rpc_shutdown: Arc<AtomicBool>, is_daemon_child: bool) {
    use tokio::signal::unix::{signal, SignalKind}; // For Unix signal handling.

    if is_daemon_child {
        // The daemon child ignores SIGINT and SIGHUP so it survives terminal closure.
        // SIGTERM triggers a graceful shutdown.
        match signal(SignalKind::interrupt()) {
            Ok(mut sig) => {
                tokio::spawn(async move {
                    loop {
                        if sig.recv().await.is_some() {
                            println!("[signal] SIGINT received (ignored in daemon mode).");
                        }
                    }
                });
            }
            Err(e) => eprintln!("[signal] failed to install SIGINT handler: {}", e),
        }

        match signal(SignalKind::hangup()) {
            Ok(mut sig) => {
                tokio::spawn(async move {
                    loop {
                        if sig.recv().await.is_some() {
                            println!("[signal] SIGHUP received (ignored in daemon mode).");
                        }
                    }
                });
            }
            Err(e) => eprintln!("[signal] failed to install SIGHUP handler: {}", e),
        }

        match signal(SignalKind::terminate()) {
            Ok(mut sig) => {
                let flag = rpc_shutdown.clone();
                tokio::spawn(async move {
                    if sig.recv().await.is_some() {
                        println!("[signal] SIGTERM received. Initiating graceful shutdown...");
                        flag.store(true, Ordering::SeqCst);
                    }
                });
            }
            Err(e) => eprintln!("[signal] failed to install SIGTERM handler: {}", e),
        }

        return; // Done.
    }

    // Foreground mode: SIGINT and SIGHUP request a detach (handover to background).
    match signal(SignalKind::interrupt()) {
        Ok(mut sig) => {
            tokio::spawn(async move {
                if sig.recv().await.is_some() {
                    println!(); // Print a newline.
                    println!("[signal] SIGINT (Ctrl+C) received. Detaching into the background...");
                    detach_to_background(); // Spawn a new background child and exit.
                }
            });
        }
        Err(e) => eprintln!("[signal] failed to install SIGINT handler: {}", e),
    }

    match signal(SignalKind::hangup()) {
        Ok(mut sig) => {
            tokio::spawn(async move {
                if sig.recv().await.is_some() {
                    println!(); // Print a newline.
                    println!("[signal] SIGHUP (terminal closed) received. Detaching into the background...");
                    detach_to_background(); // Spawn a new background child and exit.
                }
            });
        }
        Err(e) => eprintln!("[signal] failed to install SIGHUP handler: {}", e),
    }

    // SIGTERM triggers a full shutdown (without detach).
    match signal(SignalKind::terminate()) {
        Ok(mut sig) => {
            tokio::spawn(async move {
                if sig.recv().await.is_some() {
                    println!(); // Print a newline.
                    println!("[signal] SIGTERM (kill) received. Initiating full shutdown...");
                    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst); // Request full shutdown.
                }
            });
        }
        Err(e) => eprintln!("[signal] failed to install SIGTERM handler: {}", e),
    }

    // SIGQUIT is a debug signal.
    match signal(SignalKind::quit()) {
        Ok(mut sig) => {
            tokio::spawn(async move {
                loop {
                    if sig.recv().await.is_some() {
                        println!(); // Print a newline.
                        println!("[signal] SIGQUIT (Ctrl+\\) received. The node continues to run.");
                    }
                }
            });
        }
        Err(e) => eprintln!("[signal] failed to install SIGQUIT handler: {}", e),
    }
}

// Spawns a new `zenachaind --daemon-child` process in the background, then exits.
// The new process inherits the same network and runs independently.
fn detach_to_background() {
    use std::process::Command;

    // Get the current executable path and arguments.
    let exe = std::env::current_exe().expect("failed to get current exe");
    let args: Vec<String> = std::env::args().collect();

    // Build the command: same executable + original args + --daemon-child.
    let mut cmd = Command::new(exe);
    for arg in &args[1..] {
        // Skip the --daemon-child flag if it somehow exists.
        if arg != "--daemon-child" {
            cmd.arg(arg);
        }
    }
    cmd.arg("--daemon-child");

    // Redirect stdout/stderr to the debug log.
    let log_path = zenachain::datadir::debug_log_path();
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("failed to open debug log");

    let log_file_clone = log_file.try_clone().expect("failed to clone log file");

    cmd.stdout(log_file);
    cmd.stderr(log_file_clone);

    // Spawn the child.
    match cmd.spawn() {
        Ok(child) => {
            println!("[daemon] spawned background process (PID: {})", child.id());
            println!("[daemon] parent process exiting; node continues in the background.");
            // Exit the parent. The child continues independently.
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("[daemon] failed to spawn background process: {}", e);
            eprintln!("[daemon] continuing in the foreground.");
        }
    }
}
