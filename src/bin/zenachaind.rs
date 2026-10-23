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
// In this case, the node stops completely and does NOT spawn a background child.
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

// A flag that indicates a detach (handover to background) was requested (via Ctrl+C).
// In this case, the node continues running in the background without saving state.
static DETACH_REQUESTED: AtomicBool = AtomicBool::new(false);

fn main() {
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

    // If this is the daemon child, we skip the signal-based detach logic.
    // The child runs the node normally.
    if !is_daemon_child {
        // Install a SIGINT handler that triggers a detach (fork) BEFORE starting Tokio.
        install_detach_signal_handler();
    }

    println!("ZenaChain server starting (network: {})", selected_network.name()); // Log the network.
    if is_daemon_child {
        println!("[daemon] running in the background."); // Log that this is the background child.
    } else {
        println!("Press Ctrl+C to detach the node into the background (the node will keep running)."); // Log the detach behavior.
        println!("Use `./zenachain stop` to stop the node completely."); // Log the stop command.
    }

    // Start the Tokio runtime.
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main(selected_network, is_daemon_child));
}

// The async main function, containing the node logic.
async fn async_main(selected_network: Network, is_daemon_child: bool) {
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

    // Install signal handlers for the async runtime (SIGTERM for shutdown).
    // SIGINT is handled by the pre-Tokio handler in `install_detach_signal_handler`.
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

// Installs a SIGINT handler that triggers a detach (fork) BEFORE starting Tokio.
// This is safe because it runs on the main thread, outside the Tokio runtime.
fn install_detach_signal_handler() {
    use nix::sys::signal::{self, SigHandler, Signal}; // For signal handling.

    extern "C" fn handle_sigint(_: i32) {
        // This handler runs on the main thread before Tokio starts.
        // It forks the process and lets the child continue.
        detach_to_background();
    }

    unsafe {
        // Install the SIGINT handler.
        signal::signal(Signal::SIGINT, SigHandler::Handler(handle_sigint))
            .expect("failed to install SIGINT handler");
    }
}

// Installs signal handlers for the async runtime.
// SIGTERM triggers a full shutdown (without detach).
fn install_signal_handlers(rpc_shutdown: Arc<AtomicBool>, is_daemon_child: bool) {
    use tokio::signal::unix::{signal, SignalKind}; // For Unix signal handling.

    if is_daemon_child {
        // The daemon child ignores SIGHUP so it survives terminal closure.
        // SIGTERM triggers a graceful shutdown.
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
}

// Detaches the current process from the terminal and continues in the background.
// This uses `fork()` + `setsid()` and does NOT save state.
// The parent exits; the child continues running the node.
fn detach_to_background() {
    use nix::unistd::{fork, ForkResult, setsid}; // For fork and setsid.

    match unsafe { fork() } {
        Ok(ForkResult::Parent { .. }) => {
            // Parent process: exit gracefully.
            println!();
            println!("[daemon] parent process exiting; node continues in the background.");
            std::process::exit(0); // Exit the parent.
        }
        Ok(ForkResult::Child) => {
            // Child process: continue running as the daemon.
            // Create a new session so the child is not tied to the terminal.
            let _ = setsid();
            println!();
            println!("[daemon] node is now running in the background.");
            // The child continues into the Tokio runtime.
        }
        Err(e) => {
            // Fork failed: log the error and continue in the foreground.
            eprintln!("[daemon] fork failed: {}. Continuing in the foreground.", e);
        }
    }
}
