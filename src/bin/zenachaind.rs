use futures::StreamExt;
use libp2p::Multiaddr;
use tokio::signal::unix::{signal, SignalKind};
use zenachain::blockchain::Blockchain;
use zenachain::p2p;
use zenachain::rpc::RpcServer;

#[tokio::main]
async fn main() {
    let chain = Blockchain::new();

    println!("ZenaChain server starting");
    println!("Gunakan `./zenachain stop` untuk menghentikan node.");

    // Abaikan SIGINT dan SIGHUP supaya node tetap jalan.
    // Hanya perintah `./zenachain stop` yang boleh menghentikan.
    let mut sigint = signal(SignalKind::interrupt()).expect("gagal daftar SIGINT");
    let mut sighup = signal(SignalKind::hangup()).expect("gagal daftar SIGHUP");

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    println!("[signal] SIGINT diabaikan. Gunakan `./zenachain stop` untuk berhenti.");
                }
                _ = sighup.recv() => {
                    println!("[signal] SIGHUP diabaikan. Node tetap jalan.");
                }
            }
        }
    });

    let keypair = p2p::swarm::load_or_generate_keypair(".chain/peer_id");

    let mut swarm = match p2p::build_swarm(keypair).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[p2p] gagal bikin swarm: {}", e);
            return;
        }
    };

    let listen_addr: Multiaddr = match p2p::peers::LISTEN_ADDR.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("[p2p] alamat listen tidak valid: {}", e);
            return;
        }
    };

    if let Err(e) = swarm.listen_on(listen_addr) {
        eprintln!("[p2p] gagal listen {}: {}", p2p::peers::LISTEN_ADDR, e);
    }

    for addr in p2p::peers::BOOTSTRAP_PEERS {
        match addr.parse::<Multiaddr>() {
            Ok(ma) => {
                if let Err(e) = swarm.dial(ma) {
                    eprintln!("[p2p] gagal dial {}: {}", addr, e);
                }
            }
            Err(e) => eprintln!("[p2p] alamat bootstrap tidak valid {}: {}", addr, e),
        }
    }

    let chain_for_rpc = chain.clone();
    tokio::spawn(async move {
        let mut server = RpcServer::new(chain_for_rpc);
        server.listen();
    });

    while let Some(event) = swarm.next().await {
        let _ = p2p::events::classify(event);
    }
}
