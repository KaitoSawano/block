use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use zenachain::blockchain::Blockchain;
use zenachain::config::Config;
use zenachain::p2p::network::{NetworkConfig, spawn};
use zenachain::rpc::RpcServer;
use zenachain::storage;

#[tokio::main]
async fn main() {
    println!("ZenaChain server starting");
    println!("Gunakan `./zenachain stop` untuk menghentikan node.");

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

    let chain = Arc::new(Mutex::new(
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
    tokio::spawn(async move {
        let chain_guard = chain_for_rpc.lock().await.clone();
        let mut server = RpcServer::new(chain_guard, &cfg_for_rpc);
        server.listen();
    });

    let net_handle = spawn(net_config, chain.clone());

    tokio::select! {
        res = net_handle => {
            match res {
                Ok(Ok(())) => println!("[p2p] network berhenti"),
                Ok(Err(e)) => eprintln!("[p2p] network error: {}", e),
                Err(e) => eprintln!("[p2p] task error: {}", e),
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n[signal] Ctrl+C diterima. Menyimpan chain...");
            let guard = chain.lock().await;
            if let Err(e) = storage::save_chain(&guard) {
                eprintln!("[node] gagal simpan chain: {}", e);
            } else {
                println!("[node] chain disimpan");
            }
        }
    }

    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("[node] shutdown selesai");
}
