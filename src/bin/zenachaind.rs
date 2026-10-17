use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;

use zenachain::blockchain::Blockchain;
use zenachain::config::Config;
use zenachain::p2p::network::{NetworkConfig, spawn};
use zenachain::rpc::RpcServer;
use zenachain::storage;

#[tokio::main]
async fn main() {
    println!("ZenaChain server starting");
    println!("Gunakan `./zenachain stop` untuk menghentikan node.");
    println!("Ctrl+C, kill, SIGHUP tidak mematikan node (sama seperti Bitcoin).");

    let config = Config::load_or_default();
    println!("[config] network      : {}", config.network);
    println!("[config] rpc_addr     : {}", config.rpc_addr);
    println!("[config] listen_addr  : {}", config.listen_addr);
    println!("[config] bootstrap    : {:?}", config.bootstrap);
    println!("[config] mining_addr  : {}", config.mining_address);
    println!("[config] lokasi       : {}", Config::default_path());

    if !std::path::Path::new(&Config::default_path()).exists() {
        let _ = config.save_default();
        println!("[config] file config dibuat di {}", Config::default_path());
    }

    let chain = Arc::new(RwLock::new(
        storage::load_chain().unwrap_or_else(|_| Blockchain::new()),
    ));

    let listen_addr = match config.listen_addr.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("[p2p] alamat listen tidak valid: {}", e);
            return;
        }
    };

    let mut bootstrap = Vec::new();
    for b in &config.bootstrap {
        if let Ok(sa) = b.parse() {
            bootstrap.push(sa);
        } else {
            eprintln!("[p2p] bootstrap tidak valid: {}", b);
        }
    }

    let net_config = NetworkConfig {
        listen_addr,
        bootstrap,
        ..Default::default()
    };

    let chain_for_rpc = chain.clone();
    let cfg_for_rpc = config.clone();
    let rpc_shutdown = {
        let mut server = RpcServer::new(chain_for_rpc.clone(), &cfg_for_rpc);
        let flag = server.shutdown_flag();
        tokio::task::spawn_blocking(move || {
            server.listen();
        });
        flag
    };

    let net_handle = spawn(net_config, chain.clone());

    install_signal_handlers();

    loop {
        if rpc_shutdown.load(Ordering::SeqCst) {
            println!("[node] shutdown diminta via RPC");
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    let guard = chain.read().await;
    if let Err(e) = storage::save_chain(&guard) {
        eprintln!("[node] gagal simpan chain: {}", e);
    } else {
        println!("[node] chain disimpan");
    }
    drop(guard);

    net_handle.abort();

    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("[node] shutdown selesai");
}

fn install_signal_handlers() {
    use tokio::signal::unix::{signal, SignalKind};

    let kinds = [
        (SignalKind::interrupt(), "SIGINT (Ctrl+C)"),
        (SignalKind::terminate(), "SIGTERM (kill)"),
        (SignalKind::hangup(), "SIGHUP (terminal ditutup)"),
        (SignalKind::quit(), "SIGQUIT (Ctrl+\\)"),
    ];

    for (kind, name) in kinds {
        match signal(kind) {
            Ok(mut sig) => {
                tokio::spawn(async move {
                    loop {
                        if sig.recv().await.is_some() {
                            println!();
                            println!("[signal] {} diterima. Node tetap berjalan.", name);
                            println!("[signal] Gunakan `./zenachain stop` untuk mematikan node.");
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("[signal] gagal pasang handler {}: {}", name, e);
            }
        }
    }
}
